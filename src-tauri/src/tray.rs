//! Menu bar presence, so the app can outlive its window.
//!
//! Meeting auto-detection runs on a thread spawned in `setup` and never touches
//! a window — but it dies with the process, and the process used to end the
//! moment the last window closed. Detection was therefore only ever active
//! while someone had the app deliberately open, which is the opposite of what
//! it is for.
//!
//! Closing the window now hides it and the app stays in the menu bar. That
//! needs an affordance to get back, and a deliberate way out, which is what
//! this module provides.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

/// Passed by the login item so a boot-time launch stays in the menu bar
/// instead of opening a window nobody asked for.
pub const HIDDEN_FLAG: &str = "--hidden";

/// Whether this process was started by the login item.
pub fn started_hidden() -> bool {
    std::env::args().any(|a| a == HIDDEN_FLAG)
}

/// Set only by the tray's Quit item. `RunEvent::ExitRequested` refuses every
/// other exit, so without this the app could not be closed at all.
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Whether the user asked to quit through the tray.
pub fn quit_requested() -> bool {
    QUIT_REQUESTED.load(Ordering::SeqCst)
}

/// Monochrome menu bar glyph. macOS renders a *template* image as a mask,
/// inverting it for light and dark menu bars; the full-colour app icon in
/// `icons/` would look wrong in both.
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-Template@2x.png");

/// Build the menu bar icon and its menu.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Minutes", true, None::<&str>)?;
    let new_meeting = MenuItem::with_id(app, "new_meeting", "Start a meeting", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Minutes", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &new_meeting, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(Image::from_bytes(TRAY_ICON)?)
        // Tells macOS to treat the glyph as a mask. Ignored elsewhere.
        .icon_as_template(true)
        .tooltip("Minutes")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "new_meeting" => {
                show_main_window(app);
                if let Err(e) = crate::prompt_window::show_new_meeting_prompt(app.clone()) {
                    tracing::warn!("could not open the new-meeting prompt: {e}");
                }
            }
            "quit" => quit_app(app),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Reveal and focus the main window, restoring the Dock icon with it.
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    apply_activation_policy(app);
}

/// Quit for real: stop any recording first, then let `ExitRequested` through.
///
/// Exiting with a recording in flight would lose whatever had not been written,
/// so this takes the same path the Stop button does rather than a faster one.
fn quit_app(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let recording = app
            .try_state::<crate::state::AppState>()
            .is_some_and(|state| state.is_recording());
        if recording {
            tracing::info!("quit requested while recording; stopping first");
            if let Err(e) = crate::recorder::stop(&app).await {
                tracing::warn!("could not stop the recording cleanly before quitting: {e}");
            }
        }
        QUIT_REQUESTED.store(true, Ordering::SeqCst);
        app.exit(0);
    });
}

/// Whether the app should occupy the Dock, given how many windows are visible.
///
/// Split out from [`apply_activation_policy`] because it is the only part of
/// this module that can be tested: everything else needs a live `AppHandle`.
#[cfg(target_os = "macos")]
pub fn dock_visible(visible_windows: usize) -> bool {
    visible_windows > 0
}

/// Show the Dock icon only while a window is on screen.
///
/// The app is a menu bar utility for most of its life; leaving it in the Dock
/// and Cmd-Tab while it sits invisible in the background misrepresents it.
#[cfg(target_os = "macos")]
pub fn apply_activation_policy(app: &AppHandle) {
    let visible = app
        .webview_windows()
        .values()
        .filter(|w| w.is_visible().unwrap_or(false))
        .count();
    let policy = if dock_visible(visible) {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    };
    if let Err(e) = app.set_activation_policy(policy) {
        tracing::warn!("could not set the macOS activation policy: {e}");
    }
}

/// No-op off macOS: Windows and Linux have no equivalent of the Dock/Accessory
/// distinction, and the taskbar entry follows the window on its own.
#[cfg(not(target_os = "macos"))]
pub fn apply_activation_policy(_app: &AppHandle) {}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn the_dock_follows_the_windows() {
        // Hidden in the background: a menu bar utility, not a Dock app.
        assert!(!dock_visible(0));
        assert!(dock_visible(1));
        // The floating meeting prompt counts too — a prompt with no main
        // window should still be reachable from the Dock and Cmd-Tab.
        assert!(dock_visible(2));
    }
}
