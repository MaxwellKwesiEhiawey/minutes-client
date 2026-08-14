//! Auto-detect native call apps and open the floating meeting-start prompt.
//!
//! Strategy (macOS):
//! - Native apps: process list + mic-input PID ownership (Zoom, Slack, Teams, …)
//! - Browser Meet / Teams: AppleScript tab probe while the mic is live
//! - Prompt on each **entry** (idle → in-call), not once per app process lifetime
//!
//! Inspired by the vendored Minutes `call_detect.rs`.
//!
//! Runtime detection is macOS-only. Pure helpers stay available under
//! `cfg(any(test, target_os = "macos"))` so Linux CI can still unit-test them.

#![cfg_attr(not(any(test, target_os = "macos")), allow(dead_code))]

use tauri::AppHandle;

#[cfg(target_os = "macos")]
use crate::locking::MutexExt;
#[cfg(target_os = "macos")]
use crate::prompt_window::{self, MeetingPromptData, PromptKind};
#[cfg(target_os = "macos")]
use crate::state::AppState;
#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(any(test, target_os = "macos"))]
use std::collections::HashSet;
#[cfg(target_os = "macos")]
use std::sync::Mutex;
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};
#[cfg(target_os = "macos")]
use tauri::Manager;

#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct RunningProcess {
    pid: u32,
    ppid: u32,
    name: String,
}

#[cfg(target_os = "macos")]
struct ActiveCallState {
    process_name: String,
    display_name: String,
    /// Consecutive polls where this call app no longer owned the mic.
    idle_polls: u8,
    /// We already showed (or tried to show) a prompt for this entry.
    prompted: bool,
}

/// Idle polls before we treat the meeting as left (re-entry can prompt again).
#[cfg(target_os = "macos")]
const CALL_END_MISS_THRESHOLD: u8 = 3;
#[cfg(target_os = "macos")]
const BROWSER_PROBE_INTERVAL_SECS: u64 = 15;
#[cfg(target_os = "macos")]
const BROWSER_PROBE_BACKOFF_SECS: u64 = 300;
#[cfg(target_os = "macos")]
const BROWSER_MEETING_STICKY_SECS: u64 = 20;

#[cfg(target_os = "macos")]
struct DetectionTransition {
    /// Fresh meeting entry that should get a prompt (unless cooled / busy).
    entered: Option<(String, String)>,
    /// Process name of a session that just ended.
    ended_process: Option<String>,
}

/// Background call detector. Spawned once at app startup on macOS.
#[cfg(target_os = "macos")]
pub struct CallDetector {
    active_call: Mutex<Option<ActiveCallState>>,
    browser_probe_next_allowed_at: Mutex<Option<Instant>>,
    recent_google_meet_until: Mutex<Option<Instant>>,
    recent_teams_web_until: Mutex<Option<Instant>>,
}

/// Per-browser probe backoff, keyed by AppleScript app name.
///
/// Process-global rather than a `CallDetector` field because the detector is a
/// local owned by its own thread (see `spawn`) and nothing else can reach it,
/// while onboarding needs to clear a browser's backoff the moment the user
/// grants Automation — otherwise detection stays dead for up to
/// `BROWSER_PROBE_BACKOFF_SECS` after a grant the user just watched succeed.
/// Only one detector is ever spawned, so there is no shared-state ambiguity.
#[cfg(target_os = "macos")]
static BROWSER_PROBE_BACKOFF: std::sync::LazyLock<Mutex<HashMap<String, Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Drop a browser's probe backoff so the next poll retries immediately.
#[cfg(target_os = "macos")]
pub fn clear_browser_backoff(app_name: &str) {
    BROWSER_PROBE_BACKOFF.lock_safe().remove(app_name);
}

#[cfg(not(target_os = "macos"))]
pub fn clear_browser_backoff(_app_name: &str) {}

#[cfg(not(target_os = "macos"))]
pub struct CallDetector;

impl CallDetector {
    #[cfg(target_os = "macos")]
    fn new() -> Self {
        Self {
            active_call: Mutex::new(None),
            browser_probe_next_allowed_at: Mutex::new(None),
            recent_google_meet_until: Mutex::new(None),
            recent_teams_web_until: Mutex::new(None),
        }
    }

    pub fn spawn(app: AppHandle) {
        #[cfg(target_os = "macos")]
        {
            let detector = CallDetector::new();
            std::thread::Builder::new()
                .name("desksec-call-detect".into())
                .spawn(move || detector.run_loop(app))
                .ok();
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = app;
        }
    }

    #[cfg(target_os = "macos")]
    fn run_loop(&self, app: AppHandle) {
        tracing::info!("call detection started (entry-edge prompts)");
        loop {
            let (enabled, poll_secs, apps) = {
                let Some(state) = app.try_state::<AppState>() else {
                    std::thread::sleep(Duration::from_secs(2));
                    continue;
                };
                let s = state.settings.lock_safe();
                (
                    s.call_detection_enabled,
                    s.call_detection_poll_interval_secs.max(1),
                    s.call_detection_apps.clone(),
                )
            };

            if !enabled {
                *self.active_call.lock_safe() = None;
                std::thread::sleep(Duration::from_secs(poll_secs));
                continue;
            }

            let is_recording = app
                .try_state::<AppState>()
                .map(|s| s.is_recording())
                .unwrap_or(false);
            let prompt_open = prompt_window::prompt_is_open(&app);
            let detected = self.detect_active_call(&apps);

            // Always track enter/leave — even while recording — so hanging up
            // clears the session and the *next* join can prompt again.
            let entry = self.note_detection(detected.as_ref());

            if let Some(ended) = entry.ended_process {
                tracing::debug!(process = %ended, "call session ended — ready for next entry");
                if let Some(state) = app.try_state::<AppState>() {
                    state.prompt.clear_cooldown(&ended);
                }
            }

            let can_show = !is_recording && !prompt_open;
            if can_show {
                if let Some((display_name, process_name)) = entry.entered {
                    let cooled = app
                        .try_state::<AppState>()
                        .map(|s| s.prompt.is_process_cooled_down(&process_name))
                        .unwrap_or(false);
                    if cooled {
                        tracing::debug!(
                            process = %process_name,
                            "call entry suppressed (dismiss cooldown)"
                        );
                        if let Some(state) = self.active_call.lock_safe().as_mut() {
                            state.prompted = true;
                        }
                    } else {
                        tracing::info!(
                            app = %display_name,
                            process = %process_name,
                            "call entry detected — showing meeting prompt"
                        );
                        match prompt_window::show_meeting_prompt(
                            &app,
                            MeetingPromptData {
                                kind: PromptKind::Call,
                                app_name: Some(display_name.clone()),
                                process_name: Some(process_name.clone()),
                                suggested_title: Some(format!("{display_name} call")),
                            },
                        ) {
                            Ok(()) => {
                                if let Some(state) = self.active_call.lock_safe().as_mut() {
                                    state.prompted = true;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("failed to show call prompt: {e}");
                                // Leave prompted=false so a later poll can retry
                                // once the window/recording state is clear.
                            }
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_secs(poll_secs));
        }
    }

    /// Update session state from this poll. Returns whether we just entered a
    /// meeting (idle→active or app switch) and/or left one.
    #[cfg(target_os = "macos")]
    fn note_detection(&self, detected: Option<&(String, String)>) -> DetectionTransition {
        let mut active = self.active_call.lock_safe();
        match (active.as_mut(), detected) {
            (None, Some((display_name, process_name))) => {
                *active = Some(ActiveCallState {
                    process_name: process_name.clone(),
                    display_name: display_name.clone(),
                    idle_polls: 0,
                    prompted: false,
                });
                DetectionTransition {
                    entered: Some((display_name.clone(), process_name.clone())),
                    ended_process: None,
                }
            }
            (Some(state), Some((display_name, process_name)))
                if state.process_name == *process_name =>
            {
                state.idle_polls = 0;
                state.display_name = display_name.clone();
                // Same continuous in-call session — prompt only if we never did.
                let entered = if state.prompted {
                    None
                } else {
                    Some((display_name.clone(), process_name.clone()))
                };
                DetectionTransition {
                    entered,
                    ended_process: None,
                }
            }
            (Some(prev), Some((display_name, process_name))) => {
                let ended = prev.process_name.clone();
                *active = Some(ActiveCallState {
                    process_name: process_name.clone(),
                    display_name: display_name.clone(),
                    idle_polls: 0,
                    prompted: false,
                });
                DetectionTransition {
                    entered: Some((display_name.clone(), process_name.clone())),
                    ended_process: Some(ended),
                }
            }
            (Some(state), None) => {
                state.idle_polls = state.idle_polls.saturating_add(1);
                if state.idle_polls >= CALL_END_MISS_THRESHOLD {
                    let ended = active.take().map(|s| s.process_name);
                    DetectionTransition {
                        entered: None,
                        ended_process: ended,
                    }
                } else {
                    DetectionTransition {
                        entered: None,
                        ended_process: None,
                    }
                }
            }
            (None, None) => DetectionTransition {
                entered: None,
                ended_process: None,
            },
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_active_call(&self, apps: &[String]) -> Option<(String, String)> {
        let processes = running_processes();
        let active_input_pids = active_input_process_pids();
        let mic_live = active_input_pids
            .as_ref()
            .map(|pids| !pids.is_empty())
            .unwrap_or_else(is_mic_in_use);

        if mic_live && sticky_alive(&self.recent_google_meet_until) {
            return Some(("Google Meet".into(), "google-meet".into()));
        }
        if mic_live && sticky_alive(&self.recent_teams_web_until) {
            return Some(("Teams".into(), "teams-web".into()));
        }

        if !mic_live {
            return None;
        }

        let native_apps: Vec<&String> = apps
            .iter()
            .filter(|app| app.as_str() != "google-meet" && app.as_str() != "teams-web")
            .collect();

        if let Some(active_pids) = active_input_pids.as_ref() {
            for config_app in &native_apps {
                if native_app_has_active_input(config_app, &processes, active_pids) {
                    return Some((display_name_for(config_app), (*config_app).clone()));
                }
            }
        } else {
            // Fallback without PID list: process running + mic live.
            for config_app in &native_apps {
                if processes
                    .iter()
                    .any(|p| process_name_matches_config_app(config_app, &p.name))
                {
                    return Some((display_name_for(config_app), (*config_app).clone()));
                }
            }
        }

        // Browser Meet / Teams (Chrome, Arc, Safari, …) via AppleScript tabs.
        if self.browser_probe_due() {
            self.schedule_next_browser_probe();
            let running: Vec<String> = processes.iter().map(|p| p.name.clone()).collect();
            match self.detect_browser_meeting(&running) {
                BrowserMeetProbe::Detected { provider } => {
                    let sticky = match provider {
                        MeetingProvider::GoogleMeet => &self.recent_google_meet_until,
                        MeetingProvider::TeamsWeb => &self.recent_teams_web_until,
                    };
                    remember_sticky(sticky, Duration::from_secs(BROWSER_MEETING_STICKY_SECS));
                    let (display, sentinel) = provider.names();
                    return Some((display.into(), sentinel.into()));
                }
                BrowserMeetProbe::PermissionDenied { browser_app } => {
                    self.defer_browser_probe_for(&browser_app, "apple_events_permission_denied");
                    tracing::warn!(
                        browser = %browser_app,
                        "Allow Minutes to control this browser in System Settings → Privacy & Security → Automation so Google Meet / Teams tabs can be detected"
                    );
                }
                BrowserMeetProbe::Error {
                    browser_app,
                    reason,
                } => {
                    self.defer_browser_probe_for(&browser_app, &reason);
                }
                BrowserMeetProbe::NoMatch | BrowserMeetProbe::NoBrowserProcesses => {}
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    fn browser_probe_due(&self) -> bool {
        let mut next = self.browser_probe_next_allowed_at.lock_safe();
        match *next {
            Some(until) if Instant::now() < until => false,
            Some(_) => {
                *next = None;
                true
            }
            None => true,
        }
    }

    #[cfg(target_os = "macos")]
    fn schedule_next_browser_probe(&self) {
        *self.browser_probe_next_allowed_at.lock_safe() =
            Some(Instant::now() + Duration::from_secs(BROWSER_PROBE_INTERVAL_SECS));
    }

    #[cfg(target_os = "macos")]
    fn browser_probe_allowed_for(&self, browser_app: &str) -> bool {
        let mut backoff = BROWSER_PROBE_BACKOFF.lock_safe();
        match backoff.get(browser_app).copied() {
            Some(until) if Instant::now() < until => false,
            Some(_) => {
                backoff.remove(browser_app);
                true
            }
            None => true,
        }
    }

    #[cfg(target_os = "macos")]
    fn defer_browser_probe_for(&self, browser_app: &str, reason: &str) {
        BROWSER_PROBE_BACKOFF.lock_safe().insert(
            browser_app.to_string(),
            Instant::now() + Duration::from_secs(BROWSER_PROBE_BACKOFF_SECS),
        );
        tracing::warn!(
            browser = %browser_app,
            reason,
            backoff_secs = BROWSER_PROBE_BACKOFF_SECS,
            "browser call-detect probe deferred"
        );
    }

    #[cfg(target_os = "macos")]
    fn detect_browser_meeting(&self, running: &[String]) -> BrowserMeetProbe {
        let running_lower: Vec<String> = running.iter().map(|s| s.to_lowercase()).collect();
        let mut saw_browser = false;

        for browser in known_browsers() {
            let KnownBrowser {
                proc_fragment,
                app_name,
                kind,
                exact,
            } = *browser;
            let proc_match = if exact {
                running_lower.iter().any(|p| p == proc_fragment)
            } else {
                running_lower.iter().any(|p| p.contains(proc_fragment))
            };
            if !proc_match {
                continue;
            }
            saw_browser = true;
            if !self.browser_probe_allowed_for(app_name) {
                continue;
            }

            match query_browser_tabs(app_name, kind) {
                AppleScriptProbe::Tabs(tabs) => {
                    for tab in &tabs {
                        if looks_like_google_meet_meeting_url(&tab.url) {
                            return BrowserMeetProbe::Detected {
                                provider: MeetingProvider::GoogleMeet,
                            };
                        }
                        if looks_like_teams_meeting_tab(&tab.url, &tab.title) {
                            return BrowserMeetProbe::Detected {
                                provider: MeetingProvider::TeamsWeb,
                            };
                        }
                    }
                }
                AppleScriptProbe::PermissionDenied => {
                    return BrowserMeetProbe::PermissionDenied {
                        browser_app: (*app_name).to_string(),
                    };
                }
                AppleScriptProbe::Error { stderr } => {
                    let snippet: String = stderr.chars().take(240).collect();
                    let reason = if snippet.is_empty() {
                        "browser_probe_error".to_string()
                    } else {
                        format!("browser_probe_error: {snippet}")
                    };
                    return BrowserMeetProbe::Error {
                        browser_app: (*app_name).to_string(),
                        reason,
                    };
                }
            }
        }

        if saw_browser {
            BrowserMeetProbe::NoMatch
        } else {
            BrowserMeetProbe::NoBrowserProcesses
        }
    }
}

#[cfg(any(test, target_os = "macos"))]
fn binary_name_from_command(command: &str) -> String {
    let trimmed = command.trim();
    trimmed.rsplit('/').next().unwrap_or(trimmed).to_string()
}

#[cfg(any(test, target_os = "macos"))]
fn split_first_field(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_start();
    let end = trimmed.find(char::is_whitespace)?;
    Some((&trimmed[..end], &trimmed[end..]))
}

#[cfg(any(test, target_os = "macos"))]
fn process_snapshots_from_ps_output(text: &str) -> Vec<RunningProcess> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (pid_text, rest) = split_first_field(trimmed)?;
            let (ppid_text, command) = split_first_field(rest)?;
            let pid = pid_text.parse().ok()?;
            let ppid = ppid_text.parse().ok()?;
            Some(RunningProcess {
                pid,
                ppid,
                name: binary_name_from_command(command),
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn running_processes() -> Vec<RunningProcess> {
    let output = std::process::Command::new("ps")
        .args(["-axo", "pid=,ppid=,comm="])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            process_snapshots_from_ps_output(&text)
        }
        _ => Vec::new(),
    }
}

#[cfg(any(test, target_os = "macos"))]
fn config_app_aliases(config_app: &str) -> Vec<&'static str> {
    match config_app {
        "Microsoft Teams" | "MSTeams" | "Microsoft Teams (work or school)" | "Teams" => vec![
            "Microsoft Teams",
            "MSTeams",
            "Microsoft Teams (work or school)",
            "Teams",
        ],
        _ => vec![],
    }
}

#[cfg(any(test, target_os = "macos"))]
fn process_name_matches_config_app(config_app: &str, process_name: &str) -> bool {
    let process_lower = process_name.to_lowercase();
    let aliases = config_app_aliases(config_app);
    let names: Vec<&str> = if aliases.is_empty() {
        vec![config_app]
    } else {
        aliases
    };
    names.iter().any(|alias| {
        let config_lower = alias.to_lowercase();
        process_lower == config_lower
            || process_lower.starts_with(&format!("{config_lower}."))
            || process_lower.starts_with(&format!("{config_lower} "))
    })
}

#[cfg(any(test, target_os = "macos"))]
fn native_app_candidate_process_pids(
    config_app: &str,
    processes: &[RunningProcess],
) -> HashSet<u32> {
    let mut candidates: HashSet<u32> = processes
        .iter()
        .filter(|process| process_name_matches_config_app(config_app, &process.name))
        .map(|process| process.pid)
        .collect();

    let mut changed = true;
    while changed {
        changed = false;
        for process in processes {
            if candidates.contains(&process.ppid) && candidates.insert(process.pid) {
                changed = true;
            }
        }
    }

    candidates
}

#[cfg(any(test, target_os = "macos"))]
fn native_app_has_active_input(
    config_app: &str,
    processes: &[RunningProcess],
    active_input_pids: &HashSet<u32>,
) -> bool {
    let candidate_pids = native_app_candidate_process_pids(config_app, processes);
    candidate_pids
        .iter()
        .any(|pid| active_input_pids.contains(pid))
}

#[cfg(any(test, target_os = "macos"))]
fn display_name_for(process: &str) -> String {
    match process {
        "zoom.us" => "Zoom".into(),
        "Microsoft Teams"
        | "MSTeams"
        | "Microsoft Teams (work or school)"
        | "Teams"
        | "teams-web" => "Teams".into(),
        "FaceTime" => "FaceTime".into(),
        "Webex" => "Webex".into(),
        "Slack" => "Slack".into(),
        "WhatsApp" => "WhatsApp".into(),
        "google-meet" => "Google Meet".into(),
        other => other.into(),
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeetingProvider {
    GoogleMeet,
    TeamsWeb,
}

#[cfg(target_os = "macos")]
impl MeetingProvider {
    fn names(self) -> (&'static str, &'static str) {
        match self {
            MeetingProvider::GoogleMeet => ("Google Meet", "google-meet"),
            MeetingProvider::TeamsWeb => ("Teams", "teams-web"),
        }
    }
}

#[cfg(target_os = "macos")]
enum BrowserMeetProbe {
    Detected { provider: MeetingProvider },
    PermissionDenied { browser_app: String },
    Error { browser_app: String, reason: String },
    NoBrowserProcesses,
    NoMatch,
}

#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserKind {
    ChromeLike,
    Safari,
}

/// A browser whose tabs the detector knows how to read.
///
/// Shared with the first-run permission onboarding (`permissions.rs`), which
/// asks for Automation consent per target app. Both must work from the same
/// list: offering to grant a browser the detector never probes — or probing one
/// onboarding never mentioned — is how the two silently drift apart.
#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Clone, Copy)]
pub struct KnownBrowser {
    /// Lowercase fragment matched against running process names.
    pub proc_fragment: &'static str,
    /// The name AppleScript and `open -Ra` know the app by.
    pub app_name: &'static str,
    pub kind: BrowserKind,
    /// Match `proc_fragment` exactly rather than as a substring. "Arc" is three
    /// letters and would otherwise match half the process table.
    pub exact: bool,
}

#[cfg(any(test, target_os = "macos"))]
pub fn known_browsers() -> &'static [KnownBrowser] {
    &[
        KnownBrowser {
            proc_fragment: "google chrome",
            app_name: "Google Chrome",
            kind: BrowserKind::ChromeLike,
            exact: false,
        },
        KnownBrowser {
            proc_fragment: "chrome canary",
            app_name: "Google Chrome Canary",
            kind: BrowserKind::ChromeLike,
            exact: false,
        },
        KnownBrowser {
            proc_fragment: "chromium",
            app_name: "Chromium",
            kind: BrowserKind::ChromeLike,
            exact: false,
        },
        KnownBrowser {
            proc_fragment: "microsoft edge",
            app_name: "Microsoft Edge",
            kind: BrowserKind::ChromeLike,
            exact: false,
        },
        KnownBrowser {
            proc_fragment: "arc",
            app_name: "Arc",
            kind: BrowserKind::ChromeLike,
            exact: true,
        },
        KnownBrowser {
            proc_fragment: "safari",
            app_name: "Safari",
            kind: BrowserKind::Safari,
            exact: false,
        },
    ]
}

/// What a failed `osascript` invocation tells us about Automation consent.
#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsascriptFailure {
    /// TCC refused: the user denied Automation for this target app, or never
    /// answered and the prompt was dismissed. macOS will not ask again.
    Denied,
    /// The grant is fine — the target app just is not running right now.
    NotRunning,
    /// Anything else (syntax, a hung app, an OSA error we do not model).
    Unknown,
}

/// Classify `osascript` stderr. Shared with onboarding so "denied" means the
/// same thing in both places.
#[cfg(any(test, target_os = "macos"))]
pub fn classify_osascript_failure(stderr: &str) -> OsascriptFailure {
    let lower = stderr.to_lowercase();
    if lower.contains("not authorized")
        || lower.contains("not permitted")
        || lower.contains("(-1743)")
    {
        return OsascriptFailure::Denied;
    }
    // -600 procNotFound / -1728 "can't get application" both mean the app is not
    // running. Consent is unaffected, so onboarding must not report these as a
    // denial the user has to go fix in System Settings.
    if lower.contains("(-600)") || lower.contains("(-1728)") || lower.contains("isn't running") {
        return OsascriptFailure::NotRunning;
    }
    OsascriptFailure::Unknown
}

#[cfg(target_os = "macos")]
struct BrowserTab {
    url: String,
    title: String,
}

#[cfg(target_os = "macos")]
enum AppleScriptProbe {
    Tabs(Vec<BrowserTab>),
    PermissionDenied,
    Error { stderr: String },
}

#[cfg(target_os = "macos")]
fn remember_sticky(sticky: &Mutex<Option<Instant>>, ttl: Duration) {
    *sticky.lock_safe() = Some(Instant::now() + ttl);
}

#[cfg(target_os = "macos")]
fn sticky_alive(sticky: &Mutex<Option<Instant>>) -> bool {
    let mut guard = sticky.lock_safe();
    match *guard {
        Some(until) if Instant::now() < until => true,
        Some(_) => {
            *guard = None;
            false
        }
        None => false,
    }
}

#[cfg(target_os = "macos")]
fn query_browser_tabs(app_name: &str, kind: BrowserKind) -> AppleScriptProbe {
    let title_property = match kind {
        BrowserKind::ChromeLike => "title",
        BrowserKind::Safari => "name",
    };
    let script = format!(
        r#"tell application "{app_name}"
set output to ""
repeat with w in windows
  repeat with t in tabs of w
    set tabUrl to ""
    set tabTitle to ""
    try
      set tabUrl to (URL of t as text)
    end try
    try
      set tabTitle to ({title_property} of t as text)
    end try
    set output to output & tabUrl & linefeed & tabTitle & linefeed
  end repeat
end repeat
return output
end tell"#
    );
    run_applescript_tabs(&script)
}

#[cfg(target_os = "macos")]
fn run_applescript_tabs(script: &str) -> AppleScriptProbe {
    let output = match std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            return AppleScriptProbe::Error {
                stderr: format!("osascript spawn failed: {e}"),
            }
        }
    };

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = text.lines().collect();
        let mut tabs = Vec::with_capacity(lines.len() / 2);
        for chunk in lines.chunks(2) {
            let url = chunk
                .first()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let title = chunk
                .get(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if url.is_empty() && title.is_empty() {
                continue;
            }
            tabs.push(BrowserTab { url, title });
        }
        return AppleScriptProbe::Tabs(tabs);
    }

    let stderr_raw = String::from_utf8_lossy(&output.stderr).to_string();
    match classify_osascript_failure(&stderr_raw) {
        OsascriptFailure::Denied => AppleScriptProbe::PermissionDenied,
        // `NotRunning` stays an error here on purpose: for the detector there is
        // nothing to read either way, and the existing backoff is the right
        // response. Onboarding is the only caller that needs the distinction.
        OsascriptFailure::NotRunning | OsascriptFailure::Unknown => AppleScriptProbe::Error {
            stderr: stderr_raw.trim().to_string(),
        },
    }
}

#[cfg(any(test, target_os = "macos"))]
fn looks_like_google_meet_meeting_url(url: &str) -> bool {
    let lower = url.trim().to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);

    let Some(rest) = without_scheme.strip_prefix("meet.google.com/") else {
        return false;
    };

    let first_segment = rest
        .split(['?', '#', '/'])
        .next()
        .unwrap_or_default()
        .trim();

    looks_like_google_meet_meeting_code(first_segment)
}

#[cfg(any(test, target_os = "macos"))]
fn looks_like_google_meet_meeting_code(segment: &str) -> bool {
    let parts: Vec<&str> = segment.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let expected_lengths = [3, 4, 3];
    parts
        .iter()
        .zip(expected_lengths)
        .all(|(part, expected_len)| {
            part.len() == expected_len && part.chars().all(|ch| ch.is_ascii_lowercase())
        })
}

#[cfg(any(test, target_os = "macos"))]
const TEAMS_MEETING_TITLE_PREFIXES: &[&str] = &[
    "meeting",
    "call ",
    "calling",
    "spotkanie",
    "reunión",
    "reunion",
    "llamada",
    "réunion",
    "appel",
    "besprechung",
    "anruf",
    "reunião",
    "chamada",
    "riunione",
    "chiamata",
    "vergadering",
    "gesprek",
    "会議",
    "会议",
    "會議",
    "회의",
];

#[cfg(any(test, target_os = "macos"))]
fn title_indicates_teams_meeting(title: &str) -> bool {
    let lower = title.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }
    TEAMS_MEETING_TITLE_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

#[cfg(any(test, target_os = "macos"))]
fn is_teams_v2_root(url: &str) -> bool {
    let lower = url.trim().to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);
    without_scheme.starts_with("teams.live.com/v2/")
        || without_scheme.starts_with("teams.microsoft.com/v2/")
        || without_scheme == "teams.live.com/v2"
        || without_scheme == "teams.microsoft.com/v2"
}

#[cfg(any(test, target_os = "macos"))]
fn looks_like_teams_meeting_tab(url: &str, title: &str) -> bool {
    if looks_like_teams_meeting_url(url) {
        return true;
    }
    is_teams_v2_root(url) && title_indicates_teams_meeting(title)
}

#[cfg(any(test, target_os = "macos"))]
fn looks_like_teams_meeting_url(url: &str) -> bool {
    let lower = url.trim().to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);

    if let Some(rest) = without_scheme.strip_prefix("teams.live.com/") {
        if rest.starts_with("meet/") {
            return true;
        }
        return rest.contains("pre-join-calling/")
            || rest.contains("meetup-join/")
            || rest.contains("modern-calling/")
            || rest.contains("calling-screen/")
            || rest.contains("meet/");
    }

    let Some(rest) = without_scheme.strip_prefix("teams.microsoft.com/") else {
        return false;
    };

    if rest.starts_with("l/meetup-join/") || rest.starts_with("meetup-join/") {
        return true;
    }

    rest.contains("pre-join-calling/")
        || rest.contains("meetup-join/")
        || rest.contains("modern-calling/")
        || rest.contains("calling-screen/")
}

// ── mic helpers (macOS) ──────────────────────────────────────────

#[cfg(target_os = "macos")]
fn find_mic_check_binary() -> Option<std::path::PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        let beside_exe = exe.parent().unwrap_or(exe.as_ref()).join("mic_check");
        if beside_exe.exists() {
            return Some(beside_exe);
        }
    }
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin/mic_check");
    if dev_path.exists() {
        return Some(dev_path);
    }
    None
}

#[cfg(target_os = "macos")]
fn run_mic_check_script(args: &[&str]) -> Option<std::process::Output> {
    if let Some(path) = find_mic_check_binary() {
        let mut cmd = std::process::Command::new(path);
        cmd.args(args);
        return cmd.output().ok();
    }

    // Fallback: write the embedded Swift source to a temp file and run it.
    let dir = std::env::temp_dir();
    let script = dir.join("desksec_mic_check.swift");
    if std::fs::write(&script, include_str!("mic_check.swift")).is_err() {
        return None;
    }
    let mut cmd = std::process::Command::new("swift");
    cmd.arg(&script);
    cmd.args(args);
    cmd.output().ok()
}

#[cfg(target_os = "macos")]
fn is_mic_in_use() -> bool {
    match run_mic_check_script(&[]) {
        Some(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim() == "1",
        _ => false,
    }
}

#[cfg(target_os = "macos")]
fn active_input_process_pids() -> Option<HashSet<u32>> {
    let out = run_mic_check_script(&["--active-input-pids"])?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut pids = HashSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let pid = trimmed.parse().ok()?;
        pids.insert(pid);
    }
    Some(pids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_name_match_is_exact_or_prefix() {
        assert!(process_name_matches_config_app("zoom.us", "zoom.us"));
        assert!(process_name_matches_config_app("Slack", "Slack"));
        assert!(process_name_matches_config_app(
            "Microsoft Teams",
            "MSTeams"
        ));
        assert!(process_name_matches_config_app(
            "Microsoft Teams",
            "Microsoft Teams (work or school)"
        ));
        assert!(!process_name_matches_config_app(
            "FaceTime",
            "com.apple.FaceTime.FTConversationService"
        ));
    }

    #[test]
    fn google_meet_and_teams_url_heuristics() {
        assert!(looks_like_google_meet_meeting_url(
            "https://meet.google.com/abc-defg-hij"
        ));
        assert!(!looks_like_google_meet_meeting_url(
            "https://meet.google.com/landing"
        ));
        assert!(looks_like_teams_meeting_url(
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting"
        ));
        assert!(looks_like_teams_meeting_tab(
            "https://teams.microsoft.com/v2/",
            "Meeting | Contoso"
        ));
        assert!(!looks_like_teams_meeting_tab(
            "https://teams.microsoft.com/v2/",
            "Chat | Contoso"
        ));
    }

    #[test]
    fn ps_snapshot_parsing() {
        let text = "  123  1 /Applications/zoom.us.app/Contents/MacOS/zoom.us\n";
        let procs = process_snapshots_from_ps_output(text);
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0].pid, 123);
        assert_eq!(procs[0].name, "zoom.us");
    }

    #[test]
    fn native_app_matches_child_input_pid() {
        let processes = vec![
            RunningProcess {
                pid: 10,
                ppid: 1,
                name: "zoom.us".into(),
            },
            RunningProcess {
                pid: 11,
                ppid: 10,
                name: "CptHost".into(),
            },
        ];
        let mut pids = HashSet::new();
        pids.insert(11u32);
        assert!(native_app_has_active_input("zoom.us", &processes, &pids));
        assert!(!native_app_has_active_input("Slack", &processes, &pids));
    }

    #[test]
    fn display_names() {
        assert_eq!(display_name_for("zoom.us"), "Zoom");
        assert_eq!(display_name_for("Slack"), "Slack");
    }
}
