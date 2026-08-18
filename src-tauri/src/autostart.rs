//! Login-item registration.
//!
//! Auto-detection only works while the process is alive, so for it to catch a
//! meeting the user has not anticipated, the app has to already be running.
//! Starting at login is what makes that true without asking anyone to remember
//! to launch it.
//!
//! Thin wrapper over `tauri-plugin-autostart`, kept separate so the rest of the
//! code does not depend on the plugin's trait being in scope, and so the
//! reconciliation rule below has one home.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Turn the login item on or off.
pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

/// Make the OS match the saved preference at startup.
///
/// The two can drift: a user can remove the login item from System Settings, a
/// migration can restore an old settings.json, or an install can be moved. The
/// stored preference is the intent, so it wins — but only when they actually
/// differ, to avoid rewriting the launch agent on every start.
pub fn reconcile(app: &AppHandle, want_enabled: bool) {
    let manager = app.autolaunch();
    match manager.is_enabled() {
        Ok(actual) if actual == want_enabled => {}
        Ok(_) => {
            if let Err(e) = set_enabled(app, want_enabled) {
                tracing::warn!("could not reconcile the login item: {e}");
            } else {
                tracing::info!("login item set to {want_enabled}");
            }
        }
        Err(e) => tracing::warn!("could not read the login item state: {e}"),
    }
}
