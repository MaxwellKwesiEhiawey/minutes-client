//! First-run permission onboarding.
//!
//! The app needs a few things granted before it works properly, and until now it
//! asked for none of them deliberately: the macOS microphone prompt fired
//! whenever someone first hit Record, and Automation consent — the thing that
//! lets us notice a Google Meet or Teams tab in a browser — was requested
//! mid-meeting by whichever `call_detect` poll happened to run first. Worse, a
//! denial there is logged and nothing else (`call_detect.rs`), so detection just
//! quietly stopped working.
//!
//! This module backs a one-time setup pass that asks while the user is expecting
//! to be asked. It owns three things:
//!
//! 1. **The step machine** (`decide`) — which steps this install should see,
//!    given the platform, whether this is a fresh install, and what is already
//!    granted. Pure and unit-tested; the frontend never derives gating itself.
//! 2. **The probes** — real status for each step, per platform.
//! 3. **The prompts** — deliberately triggering the OS dialog, where one exists.
//!
//! ## What this cannot do
//!
//! macOS TCC permissions cannot be granted programmatically. All we control is
//! *when* the system prompt appears, plus a deep link into the right System
//! Settings pane. macOS also asks **once ever** per app+target: after a denial
//! `osascript` returns `-1743` forever without re-prompting, so a denied state
//! has to say so rather than offering a button that silently does nothing.
//!
//! Windows has no consent dialog for desktop apps at all — a blocked app simply
//! receives silence — so there the microphone step checks and guides instead of
//! requesting.

use serde::{Deserialize, Serialize};

/// Bump when a step is added, and give the new step that `introduced_in`. An
/// install that finished version N is shown only steps introduced after N, so a
/// future permission reopens one step rather than the whole wizard.
pub const ONBOARDING_VERSION: u32 = 1;

/// How long to wait for the user to answer the macOS microphone prompt before
/// giving up and re-reading the status. Generous: the dialog is modal and people
/// read it.
#[cfg(target_os = "macos")]
const MICROPHONE_PROMPT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// The full set of states the frontend can be sent, on every platform.
///
/// This is a serialized IPC contract, so the variants exist everywhere even where
/// nothing local constructs them: on Linux the stubs below only ever return
/// `NotApplicable`, which leaves `Denied` / `NotDetermined` / `Unknown` with no
/// live construction site and trips `dead_code` under `-D warnings`. The derived
/// `Serialize` does not count as a use — rustc ignores derived impls when
/// computing liveness. Hence the allow, scoped so macOS (where every variant is
/// constructed) still gets real dead-code checking.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionState {
    /// Allowed. Nothing more to do.
    Granted,
    /// Refused, and on macOS that means the OS will not ask again — the only
    /// route left is System Settings.
    Denied,
    /// Never asked. This is the only state where prompting does anything.
    NotDetermined,
    /// This platform has no such permission, so there is nothing to grant.
    NotApplicable,
    /// We could not tell. Rendered as "not set up" rather than as a failure,
    /// because guessing "denied" would send the user to fix a non-problem.
    Unknown,
}

impl PermissionState {
    fn is_satisfied(self) -> bool {
        matches!(self, Self::Granted | Self::NotApplicable)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StepId {
    /// Microphone access: the recording itself.
    Microphone,
    /// Automation consent per browser, so meetings opened from a link are seen.
    BrowserDetection,
    /// Not a permission: telling Windows/Linux users that automatic detection is
    /// macOS-only, so its absence reads as a known limit and not a broken app.
    DetectionUnavailable,
}

struct StepSpec {
    id: StepId,
    introduced_in: u32,
    /// `std::env::consts::OS` values this step exists on.
    platforms: &'static [&'static str],
    /// Whether the step asks the user for something. Informational steps are
    /// treated as already satisfied on an upgrade, so people who have been using
    /// the app for months are not handed a wizard that tells them things without
    /// asking for anything.
    actionable: bool,
}

const STEPS: &[StepSpec] = &[
    StepSpec {
        id: StepId::Microphone,
        introduced_in: 1,
        // Not Linux: there is no permission model in a deb/AppImage build.
        // (Flatpak would mean portals, which is separate work.)
        platforms: &["macos", "windows"],
        actionable: true,
    },
    StepSpec {
        id: StepId::BrowserDetection,
        introduced_in: 1,
        platforms: &["macos"],
        actionable: true,
    },
    // Deliberately no system-audio step. Installing a loopback driver is not a
    // permission and not something first run can complete — it needs a download,
    // an installer and a restart — so it belongs in Settings → Audio, where
    // `loopbackSetupHint` already covers it, rather than in a sequence the user
    // is trying to get through.
    StepSpec {
        id: StepId::DetectionUnavailable,
        introduced_in: 1,
        platforms: &["windows", "linux"],
        actionable: false,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingDecision {
    /// Show the wizard.
    pub required: bool,
    /// Steps to show, in catalogue order.
    pub steps: Vec<StepId>,
    /// Stamp the version without showing anything — there is nothing to ask, and
    /// recomputing this on every launch is wasted work.
    pub stamp_now: bool,
}

/// Decide what this install should see. Pure, so the rules are testable without
/// a real TCC database or a settings file.
///
/// `satisfied` answers "is this step already taken care of" for actionable
/// steps; it is not consulted for informational ones.
pub fn decide(
    os: &str,
    preexisting_install: bool,
    completed_version: u32,
    satisfied: &dyn Fn(StepId) -> bool,
) -> OnboardingDecision {
    if completed_version >= ONBOARDING_VERSION {
        return OnboardingDecision {
            required: false,
            steps: Vec::new(),
            stamp_now: false,
        };
    }

    // A fresh install gets the full introduction, including steps that happen to
    // be granted already (a reinstall inherits TCC decisions) — they render as
    // "allowed" and give the walkthrough something truthful to say.
    let fresh_install = !preexisting_install;

    let steps: Vec<StepId> = STEPS
        .iter()
        .filter(|spec| spec.platforms.contains(&os))
        .filter(|spec| spec.introduced_in > completed_version)
        .filter(|spec| {
            if fresh_install {
                return true;
            }
            // Upgrade: only surface what is actionable and still outstanding.
            // Someone who has been using the app and already granted everything
            // sees nothing at all.
            spec.actionable && !satisfied(spec.id)
        })
        .map(|spec| spec.id)
        .collect();

    if steps.is_empty() {
        return OnboardingDecision {
            required: false,
            steps,
            stamp_now: true,
        };
    }

    OnboardingDecision {
        required: true,
        steps,
        stamp_now: false,
    }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserPermission {
    /// AppleScript/`open -Ra` name, and the key `request_browser_automation`
    /// takes back. Only ever a value from `call_detect::known_browsers()`.
    pub app_name: String,
    pub state: PermissionState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsReport {
    pub onboarding_required: bool,
    pub steps: Vec<StepId>,
    pub completed_version: u32,
    pub current_version: u32,
    pub microphone: PermissionState,
    /// Installed browsers only. Listing a browser the user does not have would
    /// ask them to grant access to nothing.
    pub browsers: Vec<BrowserPermission>,
    /// `macos` | `windows` | `linux` | other, so the UI picks the right
    /// translated copy rather than the backend sending prose.
    pub platform: String,
}

/// Which System Settings pane a deep link should open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrivacyPane {
    Microphone,
    Automation,
}

impl PrivacyPane {
    /// The URL for this pane on this platform, or `None` where there is nothing
    /// to open. Mapped here rather than passed in from the frontend so a webview
    /// string can never become a URL the app opens.
    pub fn url(self) -> Option<&'static str> {
        #[cfg(target_os = "macos")]
        {
            Some(match self {
                Self::Microphone => {
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
                }
                Self::Automation => {
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Automation"
                }
            })
        }
        #[cfg(target_os = "windows")]
        {
            match self {
                Self::Microphone => Some("ms-settings:privacy-microphone"),
                // Windows has no Automation equivalent.
                Self::Automation => None,
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = self;
            None
        }
    }
}

/// Current state of everything onboarding covers. Never prompts, so it is safe
/// to call on every mount.
pub fn report(preexisting_install: bool, completed_version: u32) -> PermissionsReport {
    let microphone = microphone_state();
    let browsers = browser_permissions();

    let decision = decide(
        std::env::consts::OS,
        preexisting_install,
        completed_version,
        &|step| match step {
            StepId::Microphone => microphone.is_satisfied(),
            // Satisfied once at least one installed browser is allowed: that is
            // enough for detection to work, and demanding all of them would nag
            // someone who deliberately allowed only their work browser.
            StepId::BrowserDetection => {
                browsers.is_empty() || browsers.iter().any(|b| b.state == PermissionState::Granted)
            }
            StepId::DetectionUnavailable => true,
        },
    );

    PermissionsReport {
        onboarding_required: decision.required,
        steps: decision.steps,
        completed_version,
        current_version: ONBOARDING_VERSION,
        microphone,
        browsers,
        platform: std::env::consts::OS.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Microphone
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub fn microphone_state() -> PermissionState {
    use minutes_core::macos_permissions::MacPermissionStatus as S;
    // Consumed read-only from the vendored crate: it already wraps
    // `AVCaptureDevice.authorizationStatusForMediaType`, which is the only way
    // to tell a TCC denial from a dead device. cpal cannot — it reports both as
    // an opaque stream-build failure, which is exactly why the current
    // "microphone unavailable" message tells the user nothing.
    match minutes_core::macos_permissions::microphone_status() {
        S::Granted => PermissionState::Granted,
        S::Denied => PermissionState::Denied,
        S::NotDetermined => PermissionState::NotDetermined,
        S::NotNeeded => PermissionState::NotApplicable,
        S::Unsupported | S::StaleOrRestartNeeded | S::Unknown => PermissionState::Unknown,
    }
}

#[cfg(target_os = "windows")]
pub fn microphone_state() -> PermissionState {
    windows_mic::consent_state()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn microphone_state() -> PermissionState {
    // ALSA/PulseAudio/PipeWire have no per-app gate in a deb/AppImage build.
    PermissionState::NotApplicable
}

/// Raise the microphone prompt, if raising it can do anything.
///
/// Blocking: it waits for the user's answer so the caller can report the real
/// outcome instead of a stale status. Run it off the UI thread — the dialog is
/// modal and the window would otherwise freeze behind the very prompt we raised.
#[cfg(target_os = "macos")]
pub fn request_microphone() -> PermissionState {
    let current = microphone_state();
    // Prompting is only meaningful once. When already denied, macOS returns
    // immediately without showing anything, so a caller that re-prompted would
    // look like a dead button.
    if current != PermissionState::NotDetermined {
        return current;
    }

    use block2::RcBlock;
    use objc2_av_foundation::{AVCaptureDevice, AVMediaTypeAudio};

    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    // Heap-allocated: the completion handler is called on an arbitrary queue
    // long after this function's stack frame is gone.
    let handler = RcBlock::new(move |granted: objc2::runtime::Bool| {
        let _ = tx.send(granted.as_bool());
    });

    unsafe {
        let Some(media_type) = AVMediaTypeAudio else {
            return PermissionState::Unknown;
        };
        AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &handler);
    }

    match rx.recv_timeout(MICROPHONE_PROMPT_TIMEOUT) {
        // Re-read rather than trusting the boolean: the authorization status is
        // the thing the rest of the app will act on.
        Ok(_) => microphone_state(),
        Err(_) => {
            tracing::warn!("microphone permission prompt was not answered in time");
            microphone_state()
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_microphone() -> PermissionState {
    // Nothing to raise: Windows has no consent dialog for desktop apps (a
    // blocked app just gets silence), and Linux has no per-app gate. Both are
    // check-and-guide, so the status is the whole answer.
    microphone_state()
}

/// Windows microphone privacy state.
///
/// There is no API for this — the documented surface is UWP-only — so the
/// ConsentStore registry value is the only signal available to a desktop app.
/// Read via `reg query` rather than Win32 calls on purpose: this repo has no
/// Windows toolchain, so Windows code ships compiled by CI and never run
/// locally, and a small parser that can be unit-tested on any platform is worth
/// more here than `unsafe` FFI that cannot.
///
/// Being undocumented, it is treated as a hint: only an explicit `Deny` is
/// reported as denied. Anything else is `Unknown`, which renders as "not set up"
/// rather than sending someone to fix a setting that is already fine.
#[cfg(target_os = "windows")]
mod windows_mic {
    use super::{parse_reg_consent_output, PermissionState};

    const CONSENT_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone";

    pub fn consent_state() -> PermissionState {
        let output = std::process::Command::new("reg")
            .args(["query", CONSENT_KEY, "/v", "Value"])
            .output();
        match output {
            Ok(out) if out.status.success() => {
                parse_reg_consent_output(&String::from_utf8_lossy(&out.stdout))
            }
            _ => PermissionState::Unknown,
        }
    }
}

/// Parse `reg query … /v Value` output into a consent state.
///
/// Compiled on every platform so it is unit-testable on the Mac this is
/// developed on — the reason for shelling out to `reg` instead of calling Win32
/// is precisely that the surrounding code cannot be run here.
#[cfg(any(test, target_os = "windows"))]
pub(crate) fn parse_reg_consent_output(text: &str) -> PermissionState {
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        if fields.next() != Some("Value") {
            continue;
        }
        // Skip the type column (REG_SZ) and read the data.
        let _ = fields.next();
        return match fields.next() {
            Some(v) if v.eq_ignore_ascii_case("allow") => PermissionState::Granted,
            Some(v) if v.eq_ignore_ascii_case("deny") => PermissionState::Denied,
            _ => PermissionState::Unknown,
        };
    }
    PermissionState::Unknown
}

// ---------------------------------------------------------------------------
// Browser Automation
// ---------------------------------------------------------------------------

/// Installed browsers and whether Automation is allowed for each.
#[cfg(target_os = "macos")]
pub fn browser_permissions() -> Vec<BrowserPermission> {
    crate::call_detect::known_browsers()
        .iter()
        .filter(|b| browser_is_installed(b.app_name))
        .map(|b| BrowserPermission {
            app_name: b.app_name.to_string(),
            state: automation_state(b.app_name),
        })
        .collect()
}

#[cfg(not(target_os = "macos"))]
pub fn browser_permissions() -> Vec<BrowserPermission> {
    Vec::new()
}

/// Is this browser installed?
///
/// `open -Ra` *reveals* the app rather than launching it, and exits non-zero
/// when there is nothing to reveal. Cheaper and less surprising than launching
/// six browsers to find out.
#[cfg(target_os = "macos")]
fn browser_is_installed(app_name: &str) -> bool {
    std::process::Command::new("/usr/bin/open")
        .arg("-Ra")
        .arg(app_name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// The smallest AppleScript that still needs Automation consent for `app_name`.
///
/// Deliberately not a tab query: this reads no page data and changes nothing,
/// but it is the same per-target grant, so consenting here is what makes the
/// real tab probe in `call_detect` work. Not `is running` either — System Events
/// answers that without ever triggering the per-target prompt, which would make
/// the button appear to succeed while granting nothing.
#[cfg(any(test, target_os = "macos"))]
pub fn automation_probe_script(app_name: &str) -> String {
    format!("tell application \"{app_name}\" to count windows")
}

/// Automation consent for one app. Triggers the TCC prompt the first time.
#[cfg(target_os = "macos")]
pub fn automation_state(app_name: &str) -> PermissionState {
    use crate::call_detect::OsascriptFailure;

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(automation_probe_script(app_name))
        .output();

    let output = match output {
        Ok(output) => output,
        Err(e) => {
            tracing::warn!(app = %app_name, "osascript spawn failed: {e}");
            return PermissionState::Unknown;
        }
    };

    if output.status.success() {
        return PermissionState::Granted;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    match crate::call_detect::classify_osascript_failure(&stderr) {
        OsascriptFailure::Denied => PermissionState::Denied,
        // The grant is fine, the app is just closed. Reporting this as denied
        // would send the user to System Settings to fix nothing.
        OsascriptFailure::NotRunning => PermissionState::Granted,
        OsascriptFailure::Unknown => {
            tracing::warn!(app = %app_name, "unclassified osascript failure during permission probe");
            PermissionState::Unknown
        }
    }
}

/// Ask for Automation consent for one browser, then report the outcome.
///
/// Validated against `known_browsers()` by the caller: a frontend string must
/// never reach `format!` into an AppleScript body.
#[cfg(target_os = "macos")]
pub fn request_browser_automation(app_name: &str) -> PermissionState {
    let state = automation_state(app_name);
    if state == PermissionState::Granted {
        // The detector backs a browser off for minutes after a denial, and that
        // timer is still running from before the user said yes. Clearing it means
        // detection starts working now instead of after the backoff they cannot
        // see.
        crate::call_detect::clear_browser_backoff(app_name);
    }
    state
}

#[cfg(not(target_os = "macos"))]
pub fn request_browser_automation(_app_name: &str) -> PermissionState {
    PermissionState::NotApplicable
}

/// Is this a browser the detector actually probes? Guards the command boundary.
pub fn is_known_browser(app_name: &str) -> bool {
    #[cfg(any(test, target_os = "macos"))]
    {
        crate::call_detect::known_browsers()
            .iter()
            .any(|b| b.app_name == app_name)
    }
    #[cfg(not(any(test, target_os = "macos")))]
    {
        let _ = app_name;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing is satisfied — the worst case for a fresh install.
    fn none_satisfied(_: StepId) -> bool {
        false
    }

    fn all_satisfied(_: StepId) -> bool {
        true
    }

    #[test]
    fn fresh_macos_install_sees_every_macos_step() {
        let decision = decide("macos", false, 0, &none_satisfied);
        assert!(decision.required);
        assert!(!decision.stamp_now);
        assert_eq!(
            decision.steps,
            vec![StepId::Microphone, StepId::BrowserDetection]
        );
        // The "detection is macOS-only" note must never appear on macOS.
        assert!(!decision.steps.contains(&StepId::DetectionUnavailable));
    }

    #[test]
    fn fresh_install_still_walks_through_already_granted_steps() {
        // A reinstall inherits TCC decisions. The walkthrough should still
        // introduce them (rendered as "allowed") rather than silently skipping
        // straight into the app.
        let decision = decide("macos", false, 0, &all_satisfied);
        assert!(decision.required);
        assert_eq!(decision.steps.len(), 2);
    }

    #[test]
    fn upgrade_with_everything_granted_shows_nothing_and_stamps() {
        let decision = decide("macos", true, 0, &all_satisfied);
        assert!(!decision.required);
        assert!(decision.stamp_now, "must not recompute on every launch");
        assert!(decision.steps.is_empty());
    }

    #[test]
    fn upgrade_surfaces_only_the_outstanding_step() {
        let decision = decide("macos", true, 0, &|step| step != StepId::Microphone);
        assert!(decision.required);
        assert_eq!(decision.steps, vec![StepId::Microphone]);
    }

    #[test]
    fn upgrade_never_shows_an_informational_only_wizard() {
        // A long-time Windows user has nothing to grant beyond the mic, so if
        // that is already fine they must not be handed a wizard whose only
        // content is telling them things.
        let decision = decide("windows", true, 0, &all_satisfied);
        assert!(!decision.required);
        assert!(decision.stamp_now);
    }

    #[test]
    fn a_completed_install_is_never_asked_again() {
        let decision = decide("macos", true, ONBOARDING_VERSION, &none_satisfied);
        assert!(!decision.required);
        assert!(
            !decision.stamp_now,
            "already stamped; nothing left to write"
        );
        assert!(decision.steps.is_empty());
    }

    #[test]
    fn a_future_version_bump_reopens_only_the_new_step() {
        // Simulates ONBOARDING_VERSION = 2 with a step introduced in 2: an
        // install that finished 1 sees just that step, not the whole wizard.
        let completed = 1;
        let reopened: Vec<StepId> = STEPS
            .iter()
            .filter(|s| s.platforms.contains(&"macos"))
            .filter(|s| s.introduced_in > completed)
            .map(|s| s.id)
            .collect();
        assert!(
            reopened.is_empty(),
            "every current step is introduced_in 1, so version 1 has nothing left to show"
        );
        // And the guard above it holds regardless of the catalogue.
        assert!(!decide("macos", true, 1, &none_satisfied).required);
    }

    #[test]
    fn windows_and_linux_get_shortened_but_non_empty_first_runs() {
        // Skipping onboarding off macOS would leave the biggest platform gap —
        // that automatic detection does not exist there — completely invisible.
        let windows = decide("windows", false, 0, &none_satisfied);
        assert_eq!(
            windows.steps,
            vec![StepId::Microphone, StepId::DetectionUnavailable]
        );
        assert!(!windows.steps.contains(&StepId::BrowserDetection));

        // Linux has no per-app microphone gate and no call detection, so the
        // only thing first run can honestly do there is say so.
        let linux = decide("linux", false, 0, &none_satisfied);
        assert_eq!(linux.steps, vec![StepId::DetectionUnavailable]);
        assert!(!linux.steps.contains(&StepId::Microphone));
    }

    #[test]
    fn setting_up_a_loopback_driver_is_never_an_onboarding_step() {
        // Installing BlackHole needs a download, an installer and a restart —
        // nothing first run can carry through — so it belongs in Settings →
        // Audio, where `loopbackSetupHint` covers it, and not in a sequence the
        // user is trying to finish.
        for os in ["macos", "windows", "linux"] {
            let steps = decide(os, false, 0, &none_satisfied).steps;
            assert!(
                steps.len() <= 2,
                "{os} first run should stay short, got {steps:?}"
            );
        }
    }

    #[test]
    fn an_unknown_platform_asks_for_nothing() {
        let decision = decide("freebsd", false, 0, &none_satisfied);
        assert!(!decision.required);
        assert!(decision.stamp_now);
    }

    #[test]
    fn steps_are_returned_in_catalogue_order() {
        // The UI renders "step 2 of 4" against this order, so it has to be the
        // catalogue's and not the filter's incidental one.
        let decision = decide("macos", false, 0, &none_satisfied);
        let catalogue: Vec<StepId> = STEPS
            .iter()
            .filter(|s| s.platforms.contains(&"macos"))
            .map(|s| s.id)
            .collect();
        assert_eq!(decision.steps, catalogue);
    }

    #[test]
    fn probe_script_is_minimal_and_reads_nothing() {
        for browser in crate::call_detect::known_browsers() {
            let script = automation_probe_script(browser.app_name);
            assert_eq!(
                script,
                format!("tell application \"{}\" to count windows", browser.app_name)
            );
            // Must not read page data: the grant is all we are after.
            assert!(!script.contains("URL"));
            assert!(!script.contains("tabs"));
            // `is running` would not trigger the per-target prompt at all.
            assert!(!script.contains("is running"));
        }
    }

    #[test]
    fn only_browsers_the_detector_probes_are_accepted() {
        assert!(is_known_browser("Safari"));
        assert!(is_known_browser("Google Chrome"));
        // Rejecting unknown names is what keeps a webview string out of an
        // AppleScript body.
        assert!(!is_known_browser("Firefox"));
        assert!(!is_known_browser("Safari\" to do shell script \"whoami"));
        assert!(!is_known_browser(""));
    }

    #[test]
    fn satisfied_states_are_the_ones_needing_no_action() {
        assert!(PermissionState::Granted.is_satisfied());
        // Nothing to grant is as good as granted for gating purposes.
        assert!(PermissionState::NotApplicable.is_satisfied());
        assert!(!PermissionState::Denied.is_satisfied());
        assert!(!PermissionState::NotDetermined.is_satisfied());
        // Unknown must not count as satisfied, or a failed probe would silently
        // skip a step the user needs.
        assert!(!PermissionState::Unknown.is_satisfied());
    }

    #[test]
    fn privacy_pane_urls_are_backend_owned() {
        // The frontend sends an enum, never a URL. On macOS both panes resolve.
        #[cfg(target_os = "macos")]
        {
            assert!(PrivacyPane::Microphone
                .url()
                .expect("mic pane")
                .starts_with("x-apple.systempreferences:"));
            assert!(PrivacyPane::Automation
                .url()
                .expect("automation pane")
                .contains("Privacy_Automation"));
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Automation is a macOS concept; elsewhere there is nothing to open.
            assert!(PrivacyPane::Automation.url().is_none());
        }
    }

    #[test]
    fn step_ids_serialize_as_stable_camel_case() {
        // The frontend switches on these, so a rename is a breaking change.
        let json = serde_json::to_string(&StepId::BrowserDetection).unwrap();
        assert_eq!(json, "\"browserDetection\"");
        assert_eq!(
            serde_json::to_string(&StepId::DetectionUnavailable).unwrap(),
            "\"detectionUnavailable\""
        );
        assert_eq!(
            serde_json::to_string(&PermissionState::NotDetermined).unwrap(),
            "\"notDetermined\""
        );
    }

    #[test]
    fn windows_consent_output_is_parsed_conservatively() {
        // Runs on macOS too — the Windows path ships compiled by CI and never
        // executed locally, so the parser is the only part that can be proven.
        let allow = "\r\nHKEY_CURRENT_USER\\...\\microphone\r\n    Value    REG_SZ    Allow\r\n";
        assert_eq!(parse_reg_consent_output(allow), PermissionState::Granted);
        let deny = "    Value    REG_SZ    Deny";
        assert_eq!(parse_reg_consent_output(deny), PermissionState::Denied);
        // Undocumented key: anything unrecognised must not become "denied".
        assert_eq!(parse_reg_consent_output(""), PermissionState::Unknown);
        assert_eq!(
            parse_reg_consent_output("    Value    REG_SZ    Prompt"),
            PermissionState::Unknown
        );
    }
}

#[cfg(all(test, target_os = "macos"))]
mod live_probe {
    /// Prints what the real probes see on this machine. `--ignored` only: the
    /// answers depend on the developer's own TCC state and installed browsers.
    #[test]
    #[ignore]
    fn show_live_permission_state() {
        println!("microphone: {:?}", super::microphone_state());
        for b in super::browser_permissions() {
            println!("browser {:<22} {:?}", b.app_name, b.state);
        }
        // The decision these feed, for both install shapes.
        println!("upgrade:      {:?}", super::report(true, 0).steps);
        println!("fresh install: {:?}", super::report(false, 0).steps);
    }
}
