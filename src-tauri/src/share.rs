//! Hand an exported meeting to another application via the OS share picker.
//!
//! Two halves:
//!
//! * A **managed temporary file** ([`stage_path`], [`create_staged_file`],
//!   [`purge`]). The path is chosen entirely here — never supplied by the
//!   webview — so this adds no arbitrary-write surface, unlike the export
//!   commands, which accept any absolute path with an allowed extension.
//!
//! * The **native picker** ([`present`]), behind `#[cfg]` per platform:
//!   `NSSharingServicePicker` on macOS, `IDataTransferManagerInterop` on
//!   Windows, and an honest error elsewhere. The UI does not offer the action
//!   where it does not exist — see `SettingsView::share_supported` — so that
//!   error is a backstop, not a user-facing path.
//!
//! Cleanup is by sweeping, not by callback. Knowing when the user dismissed the
//! picker means implementing an `NSSharingServicePickerDelegate` from Rust, which
//! is a lot of machinery to trigger an unlink; instead the whole directory is
//! wiped at startup and stale files are dropped before each share, so a shared
//! transcript never outlives the session.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Subdirectory of the app cache dir that staged shares live in.
const STAGE_DIR: &str = "shares";

/// How long a staged file may linger before the next share sweeps it away.
const STALE_AFTER: Duration = Duration::from_secs(60 * 60);

/// The file format handed to the picker. Deserialized from the webview with no
/// `#[serde(default)]`, so a missing or unrecognized value is rejected before
/// anything is written — the UI's "pick a format first" gate is enforced here
/// too, not merely presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareFormat {
    Pdf,
    Docx,
    Md,
}

impl ShareFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ShareFormat::Pdf => "pdf",
            ShareFormat::Docx => "docx",
            ShareFormat::Md => "md",
        }
    }

    /// Label for telemetry. Same strings the export events already use.
    pub fn as_str(self) -> &'static str {
        self.extension()
    }
}

/// Turn a meeting title into a safe file stem.
///
/// Mirrors `sanitizeFilename` in `src/utils/format.ts`: keep word characters and
/// hyphens, cap the length, and fall back when the title collapses to nothing
/// (an emoji-only title would otherwise yield a file called `.pdf`).
pub fn safe_stem(title: &str, fallback: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut last_was_sep = false;
    for c in title.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
        if out.chars().count() >= 60 {
            break;
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

/// The directory staged shares live in, created if absent.
fn stage_dir(cache_dir: &Path) -> Result<PathBuf> {
    let dir = cache_dir.join(STAGE_DIR);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("could not create the share staging directory {dir:?}"))?;
    Ok(dir)
}

/// Where a share of `title` in `format` should be written.
pub fn stage_path(cache_dir: &Path, title: &str, format: ShareFormat) -> Result<PathBuf> {
    let dir = stage_dir(cache_dir)?;
    Ok(dir.join(format!(
        "{}.{}",
        safe_stem(title, "meeting"),
        format.extension()
    )))
}

/// Create (or truncate) a staged file with owner-only permissions.
///
/// The cache directory is per-user already, but a meeting export can contain the
/// verbatim transcript, so the file itself is `0600` rather than relying on the
/// process umask.
pub fn create_staged_file(path: &Path) -> Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .with_context(|| format!("could not create the staged share {path:?}"))
}

/// Delete staged files older than [`STALE_AFTER`]. Best-effort: a file that
/// cannot be removed (open in another app, say) must not fail the share.
pub fn purge_stale(cache_dir: &Path) {
    let Ok(dir) = stage_dir(cache_dir) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .and_then(|m| {
                now.duration_since(m)
                    .map_err(|_| std::io::ErrorKind::Other.into())
            })
            .map(|age| age > STALE_AFTER)
            // A file with no readable timestamp is treated as stale: leaving a
            // transcript behind is the worse failure.
            .unwrap_or(true);
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Remove the whole staging directory. Called at startup so nothing survives a
/// restart, whatever happened in the previous session.
pub fn purge(cache_dir: &Path) {
    let dir = cache_dir.join(STAGE_DIR);
    if dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            tracing::warn!("could not clear staged shares at {dir:?}: {e}");
        }
    }
}

/// Whether this platform has a share picker wired up. Mirrored to the UI as
/// `SettingsView::share_supported`, which is what hides the action elsewhere.
///
/// macOS and Windows. Linux has no desktop-wide share-picker API to call.
pub const fn supported() -> bool {
    cfg!(any(target_os = "macos", target_os = "windows"))
}

/// Present the OS share picker for an already-staged file, anchored to `window`.
///
/// Everything that can fail for a reason worth reporting is checked here, before
/// hopping to the main thread: the presentation itself has to happen there (AppKit
/// and the Windows share flyout both require it), and a closure handed to
/// `run_on_main_thread` cannot return a value. Only genuinely unexpected failures
/// are left to be logged from inside it.
pub fn present_for_window<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    path: PathBuf,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Read the pointer out here: a raw pointer is not `Send`, so it crosses
        // to the main thread as an integer.
        let view = window.ns_view().map_err(|e| {
            anyhow::anyhow!("could not find the window to anchor the share sheet to: {e}")
        })? as usize;
        window
            .run_on_main_thread(move || {
                if let Err(e) = unsafe { platform::present(&path, view as *mut std::ffi::c_void) } {
                    tracing::error!("could not present the share sheet: {e:#}");
                }
            })
            .map_err(|e| anyhow::anyhow!("could not reach the main thread to share: {e}"))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        // Same reason as the macOS branch: a raw handle is not `Send`, so it
        // crosses to the main thread as an integer. Tauri's `hwnd()` hands back
        // an `HWND` from the `windows` version *its* webview stack resolved
        // (0.61.x), which is a distinct type from ours even though the layout
        // is identical — going through the pointer avoids the mismatch instead
        // of transmuting between two copies of the crate.
        let hwnd = window
            .hwnd()
            .map_err(|e| {
                anyhow::anyhow!("could not find the window to anchor the share flyout to: {e}")
            })?
            .0 as isize;
        window
            .run_on_main_thread(move || {
                if let Err(e) = unsafe { platform::present(&path, hwnd) } {
                    tracing::error!("could not present the share flyout: {e:#}");
                }
            })
            .map_err(|e| anyhow::anyhow!("could not reach the main thread to share: {e}"))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        platform::present(&path, ())
    }
}

/* ============================== macOS ============================== */

#[cfg(target_os = "macos")]
mod platform {
    use anyhow::{anyhow, Result};
    use objc2::rc::Retained;
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::{NSSharingServicePicker, NSView};
    use objc2_foundation::{NSArray, NSRectEdge, NSString, NSURL};
    use std::cell::RefCell;
    use std::path::Path;

    thread_local! {
        /// The picker must outlive the call that presents it — AppKit shows it
        /// asynchronously and reads back from the object while it is on screen,
        /// so a dropped picker is a sheet that never appears.
        ///
        /// Thread-local rather than a `static Mutex`: AppKit objects are neither
        /// `Send` nor `Sync`, and presenting only ever happens on the main
        /// thread, so the main thread is exactly the right owner.
        static LIVE_PICKER: RefCell<Option<Retained<NSSharingServicePicker>>> =
            const { RefCell::new(None) };
    }

    /// Present the share sheet for `path`, anchored to `ns_view`.
    ///
    /// # Safety
    ///
    /// `ns_view` must be a valid `NSView*` for a live window, and this must run
    /// on the main thread — both guaranteed by the caller, which obtains the
    /// pointer from `WebviewWindow::ns_view()` inside `run_on_main_thread`.
    pub unsafe fn present(path: &Path, ns_view: *mut std::ffi::c_void) -> Result<()> {
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| anyhow!("the share sheet must be presented on the main thread"))?;
        let _ = mtm;

        let view = ns_view as *mut NSView;
        let view = unsafe { view.as_ref() }.ok_or_else(|| {
            anyhow!("the window has no content view to anchor the share sheet to")
        })?;

        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("the staged share path is not valid UTF-8"))?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path_str));
        // `initWithItems:` takes an untyped NSArray, so step the URL up to
        // `AnyObject` rather than handing over an `NSArray<NSURL>`.
        let item = Retained::into_super(Retained::into_super(url));
        let items = NSArray::from_retained_slice(&[item]);

        let picker = unsafe {
            NSSharingServicePicker::initWithItems(NSSharingServicePicker::alloc(), &items)
        };
        picker.showRelativeToRect_ofView_preferredEdge(view.bounds(), view, NSRectEdge::MinY);

        // Replacing the previous picker releases it, which is the right moment:
        // a new share means the old sheet is gone.
        LIVE_PICKER.with(|slot| {
            *slot.borrow_mut() = Some(picker);
        });
        Ok(())
    }
}

/* ============================= Windows ============================= */

#[cfg(target_os = "windows")]
mod platform {
    use anyhow::{anyhow, Context, Result};
    use std::cell::RefCell;
    use std::path::Path;
    use windows::core::{factory, Interface, Ref, HSTRING};
    use windows::ApplicationModel::DataTransfer::{DataRequestedEventArgs, DataTransferManager};
    use windows::Foundation::TypedEventHandler;
    use windows::Storage::{IStorageItem, StorageFile};
    use windows::Win32::Foundation::{E_UNEXPECTED, HWND};
    use windows::Win32::UI::Shell::IDataTransferManagerInterop;
    use windows_collections::IIterable;

    /// How long to wait for `GetFileFromPathAsync` before giving up.
    ///
    /// Opening a file that was written moments ago in our own cache directory
    /// takes microseconds, so this never elapses in practice. It exists because
    /// the wait happens on the UI thread: if the completion ever failed to
    /// arrive, an unbounded wait would be a hung window, whereas this is a
    /// logged error and a flyout that does not open.
    const RESOLVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    thread_local! {
        /// The state a pending share needs to survive the call that started it.
        ///
        /// `ShowShareUIForWindow` returns immediately and `DataRequested` fires
        /// afterwards, so the manager, the event registration, and the file
        /// being offered all have to outlive [`present`] — the same reason the
        /// macOS side holds onto its picker.
        ///
        /// Thread-local rather than a `static Mutex` because `StorageFile` is
        /// not `Send` (it carries no `unsafe impl Send` in the bindings, unlike
        /// `DataPackage`), and because both the presentation and the handler run
        /// on the main thread, which makes it the right owner.
        static LIVE_SHARE: RefCell<Option<Share>> = const { RefCell::new(None) };
    }

    /// One in-flight share: what the `DataRequested` handler reads, plus the
    /// registration that will call it.
    struct Share {
        manager: DataTransferManager,
        token: i64,
        file: StorageFile,
        /// The flyout shows a title, and refuses the package outright if the
        /// `DataPackage` has none.
        title: HSTRING,
    }

    impl Drop for Share {
        /// Unhook the handler when this share is replaced or dropped. Without
        /// this, every share would leave another live registration on a manager
        /// for the same window, and one request would be answered N times.
        fn drop(&mut self) {
            if let Err(e) = self.manager.RemoveDataRequested(self.token) {
                tracing::warn!("could not unhook the previous share handler: {e}");
            }
        }
    }

    /// Present the Windows share flyout for `path`, anchored to `hwnd`.
    ///
    /// # Safety
    ///
    /// `hwnd` must be a valid window handle, and this must run on the thread
    /// that owns it — both guaranteed by the caller, which reads the handle from
    /// `WebviewWindow::hwnd()` and calls this inside `run_on_main_thread`.
    pub unsafe fn present(path: &Path, hwnd: isize) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("the staged share path is not valid UTF-8"))?;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Meeting");

        // Resolve the file *before* showing the flyout rather than inside the
        // handler. The handler cannot report a failure anywhere the user will
        // see it — it would just produce an empty share — whereas a failure here
        // is returned and logged, and the flyout never opens on nothing.
        let file = resolve(path_str)
            .with_context(|| format!("could not hand {path:?} to the share flyout"))?;

        let hwnd = HWND(hwnd as *mut std::ffi::c_void);
        let interop = factory::<DataTransferManager, IDataTransferManagerInterop>()
            .context("the Windows share API is unavailable")?;
        // Per-window, not per-process: `GetForWindow` is the desktop-app
        // substitute for `DataTransferManager::GetForCurrentView`, which needs a
        // CoreWindow we do not have.
        let manager: DataTransferManager = unsafe { interop.GetForWindow(hwnd) }
            .context("could not attach the share flyout to this window")?;

        // The handler captures nothing: `TypedEventHandler::new` requires `Send`
        // and `StorageFile` is not, so the file travels through `LIVE_SHARE`
        // instead. Both sides are the main thread, so it is the same object.
        let token = manager
            .DataRequested(&TypedEventHandler::new(
                |_: Ref<DataTransferManager>, args: Ref<DataRequestedEventArgs>| {
                    let data = args.ok()?.Request()?.Data()?;
                    LIVE_SHARE.with(|slot| {
                        let slot = slot.borrow();
                        let share = slot
                            .as_ref()
                            .ok_or_else(|| windows::core::Error::from_hresult(E_UNEXPECTED))?;
                        data.Properties()?.SetTitle(&share.title)?;
                        // `SetStorageItems` wants an `IIterable<IStorageItem>`;
                        // the stock implementation is built from a `Vec` of the
                        // interface's `Default` type, which for an interface is
                        // `Option<Interface>`.
                        let item: IStorageItem = share.file.cast()?;
                        let items: IIterable<IStorageItem> = vec![Some(item)].into();
                        // Read-only: the receiving app gets a copy to read, and
                        // must not write back into our cache directory, which is
                        // swept out from under it.
                        data.SetStorageItems(&items, true)
                    })
                },
            ))
            .context("could not register the share handler")?;

        // Store before showing: the handler reads this, and `ShowShareUIForWindow`
        // can dispatch it before returning. Assigning here also drops the
        // previous share, unhooking its handler — the right moment, since a new
        // share means the old flyout is gone.
        LIVE_SHARE.with(|slot| {
            *slot.borrow_mut() = Some(Share {
                manager,
                token,
                file,
                title: HSTRING::from(title),
            });
        });

        unsafe { interop.ShowShareUIForWindow(hwnd) }.context("could not open the share flyout")?;
        Ok(())
    }

    /// Get a `StorageFile` for an absolute path.
    ///
    /// `GetFileFromPathAsync` is the only route to the `IStorageItem` the share
    /// contract wants, and it is asynchronous, so this waits. That is safe on the
    /// UI thread because the completion is delivered on a thread pool thread —
    /// `windows-future`'s completion delegate is agile, so nothing is marshalled
    /// back to the apartment doing the waiting — and bounded regardless by
    /// [`RESOLVE_TIMEOUT`].
    fn resolve(path: &str) -> Result<StorageFile> {
        let operation = StorageFile::GetFileFromPathAsync(&HSTRING::from(path))
            .context("could not open the staged share")?;
        wait_for(operation)?.context("could not open the staged share")
    }

    /// Block the current thread on `future` until it completes, or fail once
    /// [`RESOLVE_TIMEOUT`] has elapsed.
    ///
    /// Hand-rolled rather than reaching for an executor: the app's tokio runtime
    /// is not on this thread, and one bounded wait does not justify a dependency
    /// on a block-on crate.
    fn wait_for<F: std::future::IntoFuture>(future: F) -> Result<F::Output> {
        use std::future::Future;
        use std::sync::Arc;
        use std::task::{Context, Poll, Wake, Waker};

        struct Unpark(std::thread::Thread);
        impl Wake for Unpark {
            fn wake(self: Arc<Self>) {
                self.0.unpark();
            }
            fn wake_by_ref(self: &Arc<Self>) {
                self.0.unpark();
            }
        }

        let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
        let mut cx = Context::from_waker(&waker);
        let mut future = std::pin::pin!(future.into_future());
        let deadline = std::time::Instant::now() + RESOLVE_TIMEOUT;
        loop {
            if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
                return Ok(value);
            }
            // `park` may return spuriously; the loop polls again either way, and
            // the deadline is what actually ends it.
            match deadline.checked_duration_since(std::time::Instant::now()) {
                Some(left) => std::thread::park_timeout(left),
                None => {
                    return Err(anyhow!(
                        "the file took longer than {RESOLVE_TIMEOUT:?} to open"
                    ))
                }
            }
        }
    }
}

/* ===================== Every other platform ===================== */

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use anyhow::{anyhow, Result};
    use std::path::Path;

    /// Linux has no desktop-wide share-picker API to call. The UI hides the
    /// action because `share_supported` is false, so this is only reachable by a
    /// caller that ignored that.
    pub fn present(_path: &Path, _anchor: ()) -> Result<()> {
        Err(anyhow!(
            "Sharing to another app isn't available on this platform — save the file instead."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "minutes-share-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn staged_files_stay_inside_the_cache_directory() {
        let cache = tmp();
        // A title that tries to climb out must not be able to.
        let path = stage_path(&cache, "../../etc/passwd", ShareFormat::Pdf).unwrap();
        assert!(
            path.starts_with(cache.join(STAGE_DIR)),
            "escaped the staging directory: {path:?}"
        );
        assert_eq!(path.extension().unwrap(), "pdf");
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn the_extension_follows_the_format() {
        let cache = tmp();
        for (format, ext) in [
            (ShareFormat::Pdf, "pdf"),
            (ShareFormat::Docx, "docx"),
            (ShareFormat::Md, "md"),
        ] {
            let path = stage_path(&cache, "Weekly sync", format).unwrap();
            assert_eq!(path.extension().unwrap(), ext);
        }
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn a_title_with_nothing_usable_still_produces_a_named_file() {
        let cache = tmp();
        let path = stage_path(&cache, "🎉🎉", ShareFormat::Pdf).unwrap();
        assert_eq!(path.file_name().unwrap(), "meeting.pdf");
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn safe_stem_matches_the_frontends_rules() {
        assert_eq!(safe_stem("Weekly sync", "meeting"), "Weekly_sync");
        assert_eq!(
            safe_stem("Q3: planning / notes", "meeting"),
            "Q3_planning_notes"
        );
        assert_eq!(
            safe_stem("keep-hyphens_and_underscores", "meeting"),
            "keep-hyphens_and_underscores"
        );
        assert_eq!(safe_stem("", "meeting"), "meeting");
        assert_eq!(safe_stem("///", "meeting"), "meeting");
        assert!(safe_stem(&"x".repeat(200), "meeting").chars().count() <= 60);
    }

    #[cfg(unix)]
    #[test]
    fn staged_files_are_readable_only_by_their_owner() {
        use std::os::unix::fs::PermissionsExt;
        let cache = tmp();
        let path = stage_path(&cache, "Weekly sync", ShareFormat::Md).unwrap();
        create_staged_file(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "a staged transcript must not be world-readable"
        );
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn purging_removes_stale_files_and_keeps_fresh_ones() {
        let cache = tmp();
        let fresh = stage_path(&cache, "fresh", ShareFormat::Md).unwrap();
        let stale = stage_path(&cache, "stale", ShareFormat::Md).unwrap();
        create_staged_file(&fresh).unwrap();
        create_staged_file(&stale).unwrap();

        // Backdate past the threshold.
        let old = SystemTime::now() - STALE_AFTER - Duration::from_secs(60);
        filetime_set(&stale, old);

        purge_stale(&cache);
        assert!(fresh.exists(), "a file staged just now was swept away");
        assert!(!stale.exists(), "a stale share was left on disk");
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn purge_clears_everything() {
        let cache = tmp();
        let path = stage_path(&cache, "leftover", ShareFormat::Pdf).unwrap();
        create_staged_file(&path).unwrap();
        purge(&cache);
        assert!(
            !cache.join(STAGE_DIR).exists(),
            "staging directory survived"
        );
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn an_unknown_format_is_refused_rather_than_defaulted() {
        assert!(serde_json::from_str::<ShareFormat>("\"pdf\"").is_ok());
        assert!(serde_json::from_str::<ShareFormat>("\"docx\"").is_ok());
        assert!(serde_json::from_str::<ShareFormat>("\"md\"").is_ok());
        // The UI gates on picking a format; this is the same rule enforced on
        // the way in, so a frontend bug cannot share an unnamed format.
        assert!(serde_json::from_str::<ShareFormat>("\"exe\"").is_err());
        assert!(serde_json::from_str::<ShareFormat>("\"\"").is_err());
        assert!(serde_json::from_str::<ShareFormat>("null").is_err());
    }

    /// Set a file's mtime without pulling in a crate for it.
    fn filetime_set(path: &Path, when: SystemTime) {
        let f = std::fs::File::options().write(true).open(path).unwrap();
        f.set_modified(when).unwrap();
    }
}
