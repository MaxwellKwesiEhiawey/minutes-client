mod audio;
mod autostart;
mod call_detect;
mod commands;
mod db;
mod device;
mod docx_export;
mod error;
mod local_transcribe;
mod locking;
mod markdown;
mod models;
mod pdf_export;
mod permissions;
mod prompt_window;
mod recorder;
mod remote_stream;
mod remote_transcribe;
mod secrets;
mod settings;
mod share;
mod state;
mod summary;
mod telemetry;
mod tray;
mod vault_export;

use state::AppState;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Manager;

/// Holds the log appender's flush guard so it can be dropped deliberately
/// before a `std::process::exit`. See [`init_logging`].
static LOG_GUARD: OnceLock<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    OnceLock::new();

/// Flush buffered log lines. Required before any `std::process::exit`, which
/// runs no destructors and would otherwise discard them.
fn flush_logs() {
    if let Some(guard) = LOG_GUARD.get() {
        if let Ok(mut guard) = guard.lock() {
            drop(guard.take());
        }
    }
}

/// Current UTC timestamp as RFC3339 (used for all stored times).
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Load `.env` from the project root. Tauri dev often runs with cwd = `src-tauri`,
/// so we also try the parent of `CARGO_MANIFEST_DIR` (the repo root where `.env` lives).
fn load_env_file() {
    // Load `.env` (cwd-and-up search) at startup; settings::reload_env_keys()
    // re-reads it later so edits are picked up without a restart.
    let _ = dotenvy::dotenv();
    settings::reload_env_keys();
}

/// Initialize the global `tracing` subscriber: human-readable, non-ANSI
/// output written to a daily-rotating log file under `<data_dir>/logs/`.
///
/// The filter defaults to [`DEFAULT_LOG_FILTER`], which silences the chatty
/// native backends (whisper.cpp routes its C-level logs through tracing events,
/// and ONNX Runtime is worse) while keeping our own code at INFO.
/// Rotated files past [`LOG_RETENTION`] are deleted here, since the appender
/// rotates but never prunes.
/// Override with the `DESKSEC_LOG` env var (standard `tracing-subscriber`
/// EnvFilter syntax, e.g. `DESKSEC_LOG=debug` or `DESKSEC_LOG=desksec=debug`).
///
/// The non-blocking writer's `WorkerGuard` must stay alive for the process
/// lifetime to flush buffered log lines on drop, so it is parked in
/// [`LOG_GUARD`] rather than leaked: `report_fatal_startup_error` exits via
/// `std::process::exit`, which runs no destructors, and a leaked guard would
/// mean the very error the dialog tells the user to look up never reaches the
/// log file.
fn init_logging(data_dir: &std::path::Path) {
    let log_dir = data_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("[Minutes] could not create log directory {log_dir:?}: {e}");
        return;
    }

    // Before the appender opens today's file, so a machine that has been
    // accumulating logs for months is bounded from the first launch after this
    // ships rather than only going forward.
    let pruned = prune_old_logs(&log_dir, LOG_RETENTION);
    let file_appender = tracing_appender::rolling::daily(&log_dir, "desksec.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(Mutex::new(Some(guard)));

    let filter = tracing_subscriber::EnvFilter::try_from_env("DESKSEC_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(DEFAULT_LOG_FILTER));

    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init()
    {
        eprintln!("[Minutes] failed to initialize tracing subscriber: {e}");
        return;
    }

    if pruned > 0 {
        tracing::info!(
            files = pruned,
            "removed log files past the retention window"
        );
    }
}

/// How long a rotated log file is kept.
///
/// `tracing_appender::rolling::daily` rotates but **never deletes**, so without
/// this the log directory grows for the life of the install. Two weeks is enough
/// to investigate a report that arrives days later, and short enough that stale
/// diagnostics do not sit on disk indefinitely.
const LOG_RETENTION: std::time::Duration = std::time::Duration::from_secs(14 * 24 * 60 * 60);

/// The default log filter.
///
/// `ort` (ONNX Runtime, behind speaker diarization) is *extremely* chatty at
/// INFO: on this developer's machine a single day of use produced 1.58 million
/// `ort::logging` lines and a 187 MB log file — 99% of everything written. It is
/// pinned to `warn` for the same reason `whisper_rs` and `ggml` already are:
/// per-tensor graph-optimizer chatter is not diagnostics anyone acts on, and it
/// buries the lines that are. Raise it deliberately with
/// `DESKSEC_LOG=info,ort=info` when debugging diarization itself.
const DEFAULT_LOG_FILTER: &str = "info,whisper_rs=warn,ggml=warn,ort=warn";

/// Delete rotated log files older than `keep`. Returns how many were removed.
fn prune_old_logs(log_dir: &std::path::Path, keep: std::time::Duration) -> usize {
    match std::time::SystemTime::now().checked_sub(keep) {
        Some(cutoff) => prune_logs_before(log_dir, cutoff),
        None => 0,
    }
}

/// Age-based prune against an explicit cutoff, so the rule is testable without
/// waiting two weeks or backdating a file.
fn prune_logs_before(log_dir: &std::path::Path, cutoff: std::time::SystemTime) -> usize {
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        // Only ever our own rotated logs. Anything else in this directory belongs
        // to someone else and is not ours to delete.
        if !name.starts_with("desksec.log") {
            continue;
        }
        let modified = entry.metadata().and_then(|m| m.modified());
        let Ok(modified) = modified else { continue };
        if modified < cutoff && std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Install the process-level rustls crypto provider.
///
/// reqwest enables rustls's `aws-lc-rs` provider feature and tokio-tungstenite
/// enables `ring`; Cargo unifies both onto the single rustls 0.23 instance, so
/// rustls sees two candidates, refuses to choose, and every TLS handshake
/// panics with "Could not automatically determine the process-level
/// CryptoProvider" — which takes down live transcription (WSS), AI summaries and
/// model downloads at once. Installing one explicitly settles the choice, and is
/// the reason `rustls` is a direct dependency (see Cargo.toml).
///
/// Idempotent and cheap, so every path that is about to do TLS calls it rather
/// than trusting a single call in `run()`: that single call was silently lost in
/// the develop merge 99fdf9d, which resolved lib.rs in favour of a lineage that
/// never had it, and the whole app's networking went with it.
pub(crate) fn install_tls_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // `Err` only means a provider was already installed, which is fine.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Earliest point we control, so `app_startup_duration_bucket` measures as
    // much of startup as the app can see. Stamping a monotonic instant costs
    // nothing and touches no telemetry state.
    telemetry::mark_process_start();

    // Before any TLS is attempted — see `install_tls_provider`.
    install_tls_provider();

    load_env_file();

    // Route whisper.cpp / ggml C-level logs through Rust tracing so they don't
    // flood the terminal during a recording.
    minutes_core::install_whisper_logging_hooks();

    match tauri::Builder::default()
        // First, and deliberately so: the callback must be registered before
        // anything in `setup` touches the database, since the point is to stop a
        // second process reaching those paths at all. A second launch focuses the
        // window that is already running instead of opening `desksec.db` twice.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tracing::info!("second instance attempted; focusing the running window");
            prompt_window::focus_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![tray::HIDDEN_FLAG]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let handle = app.handle();

            // Startup failures are returned, not panicked on, so `run()` below
            // can report them to the user. Tauri itself does not do that: an
            // `Err` from this hook comes back out of `Builder::build()`, and
            // `.expect()`-ing it there is what made the app vanish on launch
            // with nothing on screen (issue #5).
            let data_dir = handle
                .path()
                .app_data_dir()
                .map_err(|e| format!("could not resolve app data dir: {e}"))?;
            std::fs::create_dir_all(&data_dir).ok();

            // Needs the resolved data dir (for the log file path), so this is
            // the earliest point in startup a tracing subscriber can exist.
            init_logging(&data_dir);

            let db_path = data_dir.join("desksec.db");
            let legacy_db_path = data_dir.join("parley.db");
            if !db_path.exists() && legacy_db_path.exists() {
                // Sidecars included: the database runs in WAL mode, so renaming
                // the main file alone would strand `parley.db-wal` and lose any
                // transactions it still held — see `db::rename_with_sidecars`.
                if let Err(e) = db::rename_with_sidecars(&legacy_db_path, &db_path) {
                    tracing::warn!("failed to migrate parley.db → desksec.db: {e}");
                }
            }

            let config_dir = handle
                .path()
                .app_config_dir()
                .map_err(|e| format!("could not resolve app config dir: {e}"))?;

            // Files staged for a share are meant to be momentary. Clear them at
            // startup so nothing — least of all a transcript — survives from a
            // previous session, whatever happened to it.
            if let Ok(cache_dir) = handle.path().app_cache_dir() {
                share::purge(&cache_dir);
            }

            // `{e:#}`, not `{e}`: anyhow only prints the cause chain in the
            // alternate form, and the causes are where the actual reason lives
            // (the OS error behind a failed rename, SQLCipher's complaint).
            let opened =
                db::open(&db_path).map_err(|e| format!("failed to open database: {e:#}"))?;
            if let Some(quarantined) = &opened.quarantined {
                report_quarantined_database(handle, &db_path, quarantined);
            }
            let conn = opened.conn;
            // Recover any recording that didn't shut down cleanly last time.
            let recovered = db::recover_interrupted(&conn).unwrap_or(0);
            if recovered > 0 {
                tracing::info!("recovered {recovered} interrupted meeting(s)");
            }

            let loaded_settings = settings::load(&config_dir);

            // Anonymous usage/error telemetry (see telemetry.rs and
            // docs/TELEMETRY.md). Inert unless a runtime or CI-embedded
            // exporter credential is available; failures never affect startup.
            telemetry::init(
                &config_dir,
                &app.package_info().version.to_string(),
                loaded_settings.telemetry_enabled,
            );
            telemetry::event(
                "app_started",
                &[
                    ("schema_version", telemetry::SCHEMA_VERSION.into()),
                    (
                        "engine",
                        loaded_settings.transcription_engine.as_str().into(),
                    ),
                    ("diarization", loaded_settings.diarization_enabled.into()),
                    (
                        "capture_microphone",
                        loaded_settings.capture_microphone.into(),
                    ),
                    (
                        "capture_system_audio",
                        loaded_settings.capture_system_audio.into(),
                    ),
                    (
                        "call_detection_enabled",
                        loaded_settings.call_detection_enabled.into(),
                    ),
                ],
            );
            if recovered > 0 {
                // Proxy for crash / unclean shutdown during a recording.
                telemetry::event(
                    "unclean_shutdown_detected",
                    &[("recovered_count", (recovered as i64).into())],
                );
            }

            install_tls_provider();
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| format!("failed to build HTTP client: {e}"))?;

            app.manage(AppState {
                db: Arc::new(Mutex::new(conn)),
                http,
                config_dir,
                settings: Mutex::new(loaded_settings.clone()),
                session: Mutex::new(None),
                starting: std::sync::atomic::AtomicBool::new(false),
                prompt: prompt_window::PromptState::default(),
            });

            // Preload whisper in the background when using on-device transcription.
            if settings::is_whisper_engine(&loaded_settings.transcription_engine) {
                let preload_cfg = local_transcribe::build_config(
                    &loaded_settings.whisper_model,
                    &loaded_settings.transcription_language,
                    loaded_settings.diarization_enabled,
                );
                tauri::async_runtime::spawn(async move {
                    let _ = tauri::async_runtime::spawn_blocking(move || {
                        local_transcribe::preload_whisper(&preload_cfg);
                    })
                    .await;
                });
            }

            autostart::reconcile(handle, loaded_settings.start_at_login);

            call_detect::CallDetector::spawn(handle.clone());

            // Detection is only useful if the process outlives its window, so
            // the app now lives in the menu bar. Built after the detector so a
            // tray failure cannot stop detection from starting.
            if let Err(e) = tray::install(handle) {
                tracing::warn!("could not create the menu bar icon: {e}");
            }

            // A login start should not throw a window in the user's face; the
            // point is that Minutes is already there when a call begins.
            if tray::started_hidden() {
                tracing::info!("started by the login item; staying in the menu bar");
            } else {
                tray::show_main_window(handle);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::recording_state,
            commands::list_audio_devices,
            commands::list_meetings,
            commands::search_meetings,
            commands::get_meeting,
            commands::delete_meeting,
            commands::rename_meeting,
            commands::generate_summary,
            commands::get_settings,
            commands::check_server,
            commands::save_settings,
            commands::export_markdown,
            commands::write_text_file,
            commands::export_docx,
            commands::export_pdf,
            commands::share_meeting,
            commands::transcription_status,
            commands::download_models,
            commands::list_installed_models,
            commands::delete_installed_model,
            commands::permission_status,
            commands::request_microphone,
            commands::request_browser_automation,
            commands::open_privacy_settings,
            commands::complete_onboarding,
            commands::reset_onboarding,
            prompt_window::show_new_meeting_prompt,
            prompt_window::get_meeting_prompt,
            prompt_window::close_meeting_prompt,
            prompt_window::start_recording_from_prompt,
            prompt_window::dismiss_meeting_prompt,
        ])
        .on_window_event(|window, event| {
            // Closing the main window hides it rather than ending the process:
            // the call detector runs on a thread of its own and dies with the
            // process, so quitting on close is what made detection stop the
            // moment the window was dismissed.
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                    tray::apply_activation_policy(window.app_handle());
                }
            }
        })
        .build(tauri::generate_context!())
    {
        Ok(app) => app.run(|app, event| {
            // Windows and Linux end the process when the last window goes, and
            // on macOS this also covers Cmd-Q. Quitting is deliberate only via
            // the tray, which sets the flag before asking to exit.
            match event {
                // Windows and Linux end the process when the last window goes,
                // and on macOS this also covers Cmd-Q. Quitting is deliberate
                // only via the tray, which sets the flag before asking to exit.
                tauri::RunEvent::ExitRequested { api, .. } => {
                    if !tray::quit_requested() {
                        api.prevent_exit();
                        tray::apply_activation_policy(app);
                    }
                }
                // Windows are only actually on screen by now, so this is the
                // earliest point the Dock decision can be made correctly.
                tauri::RunEvent::Ready => tray::apply_activation_policy(app),
                _ => {}
            }
        }),
        Err(e) => report_fatal_startup_error(&e),
    }
}

/// Tell the user their stored meetings are gone.
///
/// This runs when the database could not be decrypted and was moved aside.
/// A log line is not enough: from the user's side the app simply opens with an
/// empty history and no explanation, which is indistinguishable from the app
/// having silently deleted their data.
///
/// Uses `tauri_plugin_dialog`'s callback form rather than `rfd`, and that is
/// load-bearing: this is called from inside the setup hook, and a blocking
/// dialog there stalls the hook so the main window never loads at all — the
/// app hangs behind a modal instead of starting. `show()` returns immediately
/// and the dialog resolves on its own.
fn report_quarantined_database<R: tauri::Runtime>(
    handle: &tauri::AppHandle<R>,
    db_path: &std::path::Path,
    quarantined: &std::path::Path,
) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    handle
        .dialog()
        .message(format!(
            "The encryption key for your Minutes database is no longer in the system credential \
             store, so the database could not be opened. Minutes has started with an empty \
             history.\n\nThe old database has not been deleted. It is kept at:\n{}\n\nIt can only \
             be read again if the original key is restored (for example from a keyring backup). \
             New meetings are stored in {}.",
            quarantined.display(),
            db_path.display(),
        ))
        .title("Minutes — previous meetings could not be opened")
        .kind(MessageDialogKind::Warning)
        .show(|_| {});
}

/// Show a startup failure to the user and exit non-zero.
///
/// Without this the process dies on a panic with the reason only on stderr,
/// which nobody sees when the app is launched from a desktop icon — the
/// window appears for an instant and disappears, giving the user nothing to
/// report but "it doesn't open" (issue #5).
///
/// `rfd` is used directly rather than `tauri_plugin_dialog` because there is
/// no `AppHandle` to hang a plugin dialog off: the app failed to build.
fn report_fatal_startup_error(e: &tauri::Error) -> ! {
    let message = format!(
        "Minutes could not start.\n\n{e}\n\nDetails are in the log directory under the Minutes \
         application data folder."
    );
    tracing::error!("fatal startup error: {e}");
    eprintln!("[Minutes] fatal startup error: {e}");
    flush_logs();

    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Minutes")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();

    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression cover for the develop merge that dropped the provider install
    /// (99fdf9d): without a process-level provider, rustls refuses to pick one
    /// and every TLS handshake panics on a worker thread — live transcription,
    /// summaries and model downloads all fail with nothing useful on screen.
    #[test]
    fn installs_a_process_level_tls_provider() {
        install_tls_provider();
        assert!(
            rustls::crypto::CryptoProvider::get_default().is_some(),
            "no process-level rustls CryptoProvider: every TLS handshake will panic"
        );
    }

    #[test]
    fn log_prune_removes_only_stale_desksec_logs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let write = |name: &str| {
            let p = dir.path().join(name);
            std::fs::write(&p, b"x").expect("write");
            p
        };
        write("desksec.log.2026-01-01");
        write("desksec.log.2026-01-02");
        write("desksec.log"); // the file currently being written
                              // Not ours. Deleting a neighbour's file because it shared a directory
                              // would be a far worse bug than an oversized log.
        write("telemetry-spool.jsonl");
        write("notes.txt");

        // Cutoff in the future: everything on disk counts as stale.
        let future = std::time::SystemTime::now() + std::time::Duration::from_secs(3600);
        assert_eq!(prune_logs_before(dir.path(), future), 3);

        assert!(dir.path().join("telemetry-spool.jsonl").exists());
        assert!(dir.path().join("notes.txt").exists());
        assert!(!dir.path().join("desksec.log.2026-01-01").exists());
    }

    #[test]
    fn log_prune_keeps_files_inside_the_retention_window() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("desksec.log.2026-08-13"), b"x").expect("write");

        // Cutoff in the past: a file just written is not stale.
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        assert_eq!(prune_logs_before(dir.path(), past), 0);
        assert!(dir.path().join("desksec.log.2026-08-13").exists());

        // And the real entry point must not blow up on a directory it cannot read.
        assert_eq!(
            prune_old_logs(std::path::Path::new("/nope/missing"), LOG_RETENTION),
            0
        );
    }

    /// ONNX Runtime wrote 187 MB in a single day at INFO before this was pinned.
    #[test]
    fn default_log_filter_silences_the_chatty_native_backends() {
        for target in ["whisper_rs=warn", "ggml=warn", "ort=warn"] {
            assert!(
                DEFAULT_LOG_FILTER.contains(target),
                "{target} missing from the default filter: {DEFAULT_LOG_FILTER}"
            );
        }
        // Still info for our own code, or the log would be useless.
        assert!(DEFAULT_LOG_FILTER.starts_with("info"));
    }

    /// Called from several TLS entry points, so it must tolerate repeat calls
    /// (and a provider another crate installed first) without panicking.
    #[test]
    fn installing_the_tls_provider_is_idempotent() {
        install_tls_provider();
        install_tls_provider();
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}
