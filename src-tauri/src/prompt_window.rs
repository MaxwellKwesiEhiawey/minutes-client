//! Always-on-top floating meeting-start prompt (Granola-style).
//!
//! Used for both manual "New meeting" and macOS call-detection prompts.

use crate::locking::MutexExt;
use crate::models::Meeting;
use crate::recorder;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

const PROMPT_LABEL: &str = "meeting-prompt";
const PROMPT_WIDTH: f64 = 380.0;
const PROMPT_HEIGHT: f64 = 252.0;

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
    Manual,
    Call,
}

impl PromptKind {
    /// Category label for telemetry. Never the detected app or process name —
    /// only whether the prompt was manual or call-detected.
    fn as_str(&self) -> &'static str {
        match self {
            PromptKind::Manual => "manual",
            PromptKind::Call => "call",
        }
    }
}

/// The prompt kind for accept/dismiss telemetry: call-detected prompts carry
/// a `process_name`, manual ones do not. Only that binary distinction is
/// reported — the process name itself never leaves the device.
fn prompt_kind_label(process_name: Option<&str>) -> &'static str {
    if process_name.is_some() {
        "call"
    } else {
        "manual"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingPromptData {
    pub kind: PromptKind,
    pub app_name: Option<String>,
    pub process_name: Option<String>,
    pub suggested_title: Option<String>,
}

/// Staged prompt payloads + dismiss cooldowns shared with call detection.
pub struct PromptState {
    pub pending: std::sync::Mutex<HashMap<u64, MeetingPromptData>>,
    /// process_name → do not re-prompt until this instant (set on dismiss only)
    pub dismiss_until: std::sync::Mutex<HashMap<String, Instant>>,
    /// Where the user last dragged the prompt, in logical screen coordinates.
    /// The card is movable, so it should stay where it was put instead of
    /// springing back to the corner on the next call. Session-only: a position
    /// that made sense with yesterday's monitors may not today.
    pub last_position: std::sync::Mutex<Option<(f64, f64)>>,
}

impl Default for PromptState {
    fn default() -> Self {
        Self {
            pending: std::sync::Mutex::new(HashMap::new()),
            dismiss_until: std::sync::Mutex::new(HashMap::new()),
            last_position: std::sync::Mutex::new(None),
        }
    }
}

impl PromptState {
    /// Used by macOS call detection to suppress re-prompts after "Not now".
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn is_process_cooled_down(&self, process_name: &str) -> bool {
        let mut map = self.dismiss_until.lock_safe();
        match map.get(process_name).copied() {
            Some(until) if Instant::now() < until => true,
            Some(_) => {
                map.remove(process_name);
                false
            }
            None => false,
        }
    }

    pub fn set_cooldown(&self, process_name: &str, minutes: u64) {
        if process_name.is_empty() || minutes == 0 {
            return;
        }
        self.dismiss_until.lock_safe().insert(
            process_name.to_string(),
            Instant::now() + std::time::Duration::from_secs(minutes.saturating_mul(60)),
        );
    }

    /// Used by macOS call detection when a call session ends.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn clear_cooldown(&self, process_name: &str) {
        self.dismiss_until.lock_safe().remove(process_name);
    }
}

fn top_right_position(app: &AppHandle, width: f64, _height: f64) -> (f64, f64) {
    let (screen_width, y) = match app.primary_monitor() {
        Ok(Some(monitor)) => {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let logical_w = size.width as f64 / scale;
            (logical_w, 38.0)
        }
        _ => (1440.0, 38.0),
    };
    let x = (screen_width - width - 16.0).max(8.0);
    (x, y)
}

/// Synchronously tear down any prior prompt window.
///
/// Prefer `destroy()` over `close()` — `close()` only queues a close-request
/// and can leave the label registered, which made the next show silently no-op
/// when we treated "label exists" as "already open".
fn destroy_existing_prompt(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(PROMPT_LABEL) {
        remember_position(app, &win);
        if let Err(e) = win.destroy() {
            tracing::warn!("failed to destroy meeting prompt: {e}");
            let _ = win.close();
        }
    }
}

/// Record where the prompt currently sits, so the next one opens there.
///
/// Logical coordinates, because that is what `position()` on the builder takes;
/// the physical position has to be divided by the window's scale factor or the
/// card jumps on a Retina display.
fn remember_position(app: &AppHandle, win: &tauri::WebviewWindow) {
    let Ok(physical) = win.outer_position() else {
        return;
    };
    let scale = win.scale_factor().unwrap_or(1.0);
    if scale <= 0.0 {
        return;
    }
    if let Some(state) = app.try_state::<AppState>() {
        *state.prompt.last_position.lock_safe() =
            Some((physical.x as f64 / scale, physical.y as f64 / scale));
    }
}

/// Where to open the prompt: where the user last left it, else the top-right
/// corner. A remembered position is clamped back onto the primary monitor —
/// otherwise unplugging the display it was dragged to would open the card
/// off-screen, with no way to reach it.
fn prompt_position(app: &AppHandle, remembered: Option<(f64, f64)>) -> (f64, f64) {
    let Some((x, y)) = remembered else {
        return top_right_position(app, PROMPT_WIDTH, PROMPT_HEIGHT);
    };
    let (screen_w, screen_h) = match app.primary_monitor() {
        Ok(Some(monitor)) => {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            (size.width as f64 / scale, size.height as f64 / scale)
        }
        _ => (1440.0, 900.0),
    };
    let max_x = (screen_w - PROMPT_WIDTH).max(0.0);
    let max_y = (screen_h - PROMPT_HEIGHT).max(0.0);
    (x.clamp(0.0, max_x), y.clamp(0.0, max_y))
}

/// Show (or replace) the floating meeting prompt window.
pub fn show_meeting_prompt(app: &AppHandle, data: MeetingPromptData) -> Result<(), String> {
    let state = app.state::<AppState>();
    if state.is_recording() {
        return Err("already recording".into());
    }

    // Always destroy first so a second New meeting / detection can open a fresh
    // window. Call-detect avoids calling this while a visible prompt is up.
    destroy_existing_prompt(app);

    if prompt_is_open(app) {
        return Err(
            "could not close the previous meeting prompt window; try again or restart Minutes"
                .into(),
        );
    }

    let token = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let kind_label = data.kind.as_str();
    {
        let mut map = state.prompt.pending.lock_safe();
        map.clear();
        map.insert(token, data);
    }

    let remembered = *state.prompt.last_position.lock_safe();
    let (pos_x, pos_y) = prompt_position(app, remembered);
    let url = format!("prompt.html?t={token}");

    match WebviewWindowBuilder::new(app, PROMPT_LABEL, WebviewUrl::App(url.into()))
        .title("Minutes")
        .inner_size(PROMPT_WIDTH, PROMPT_HEIGHT)
        .position(pos_x, pos_y)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .focused(true)
        .skip_taskbar(true)
        .visible(true)
        .build()
    {
        Ok(win) => {
            let _ = win.show();
            let _ = win.set_focus();
            crate::telemetry::event(
                "meeting_prompt_shown",
                &[("prompt_kind", kind_label.into())],
            );
            Ok(())
        }
        Err(e) => {
            state.prompt.pending.lock_safe().remove(&token);
            Err(format!("failed to show meeting prompt: {e}"))
        }
    }
}

pub fn close_prompt_window(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        state.prompt.pending.lock_safe().clear();
    }
    destroy_existing_prompt(app);
}

fn focus_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.unminimize();
        let _ = main.show();
        let _ = main.set_focus();
    }
}

#[tauri::command]
pub fn show_new_meeting_prompt(app: AppHandle) -> Result<(), String> {
    show_meeting_prompt(
        &app,
        MeetingPromptData {
            kind: PromptKind::Manual,
            app_name: None,
            process_name: None,
            suggested_title: None,
        },
    )
}

#[tauri::command]
pub fn get_meeting_prompt(
    state: State<AppState>,
    token: u64,
) -> Result<Option<MeetingPromptData>, String> {
    // Peek (clone), don't remove: remounts must still see the staged payload.
    Ok(state.prompt.pending.lock_safe().get(&token).cloned())
}

#[tauri::command]
pub fn close_meeting_prompt(app: AppHandle) -> Result<(), String> {
    close_prompt_window(&app);
    Ok(())
}

#[tauri::command]
pub async fn start_recording_from_prompt(
    app: AppHandle,
    title: Option<String>,
    process_name: Option<String>,
) -> Result<Meeting, String> {
    // Close the prompt first so its label is freed before we focus main /
    // continue. Do not put the call app on dismiss-cooldown after a successful
    // start — the next meeting should be able to auto-prompt again.
    let prompt_kind = prompt_kind_label(process_name.as_deref());
    close_prompt_window(&app);

    crate::telemetry::event(
        "meeting_prompt_accepted",
        &[("prompt_kind", prompt_kind.into())],
    );
    let trigger: &'static str = if process_name.is_some() {
        "call_prompt"
    } else {
        "manual"
    };
    let result = recorder::start(&app, title);
    crate::commands::emit_recording_start_telemetry(&app, trigger, result.is_ok());
    let meeting = result?;
    focus_main_window(&app);

    let _ = app.emit("meeting-started", &meeting);
    Ok(meeting)
}

#[tauri::command]
pub fn dismiss_meeting_prompt(app: AppHandle, process_name: Option<String>) -> Result<(), String> {
    if let Some(ref proc) = process_name {
        let state = app.state::<AppState>();
        let mins = state.settings.lock_safe().call_detection_cooldown_minutes;
        state.prompt.set_cooldown(proc, mins);
    }
    crate::telemetry::event(
        "meeting_prompt_dismissed",
        &[(
            "prompt_kind",
            prompt_kind_label(process_name.as_deref()).into(),
        )],
    );
    close_prompt_window(&app);
    Ok(())
}

/// Whether the floating prompt is currently open (call detector should not re-open).
pub fn prompt_is_open(app: &AppHandle) -> bool {
    app.get_webview_window(PROMPT_LABEL).is_some()
}
