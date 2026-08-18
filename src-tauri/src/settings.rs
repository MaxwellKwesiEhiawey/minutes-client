use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// User-configurable settings. Persisted as JSON in the app config dir.
///
/// The Minutes server URL and bearer token are provisioned at **CI build time**
/// (`DESKSEC_API_URL` / `DESKSEC_TOKEN` → compile-time embed). Embedded values
/// always win over runtime env, `settings.json`, and the OS credential store.
/// On first launch the embedded token is copied into the keychain for secure
/// storage. Local `.env` overrides apply only in dev builds where nothing was
/// embedded at compile time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Base URL of the Minutes summarization server, e.g. `https://minutes.example.com`.
    /// Persisted in settings.json — it is not a secret, just an endpoint, so it
    /// doesn't need OS-credential-store protection. (It used to live only in
    /// the OS credential store; that made it fragile to code-signing identity
    /// changes, since keychain items are ACL-scoped to the signing identity
    /// that created them — an app switching from ad-hoc to a real Developer ID
    /// signature can silently lose read access to items an earlier ad-hoc
    /// build wrote, with no user-visible error beyond "the URL looks empty."
    /// `load()` below still does a one-time pull from the legacy keychain
    /// entry for anyone upgrading from that era.)
    #[serde(default = "default_server_url")]
    pub server_url: String,
    /// Bearer token presented to the Minutes server. Held in memory for the
    /// session; persisted in the OS credential store, never in settings.json.
    #[serde(default, skip)]
    pub server_token: String,
    /// Whisper model used for on-device transcription: `tiny`, `base` (default),
    /// `small`, `medium`, or `large-v3`. Larger = more accurate, slower.
    #[serde(default = "default_whisper_model")]
    pub whisper_model: String,
    /// `whisper` = on-device; `deepgram` = online via Minutes server (default).
    #[serde(default = "default_transcription_engine")]
    pub transcription_engine: String,
    /// Enable on-device speaker diarization (pyannote-rs). Speaker labels are
    /// attached to transcript segments when the diarization models are present.
    #[serde(default = "default_true")]
    pub diarization_enabled: bool,
    /// When true, export each completed meeting to `~/meetings/*.md` so the
    /// vendored Minutes CLI/MCP/graph tools (see `minutes/`) can see this app's
    /// recordings.
    #[serde(default = "default_true")]
    pub export_markdown: bool,
    #[serde(default = "default_anthropic_model")]
    pub anthropic_model: String,
    /// Length of a finalized transcript chunk, in seconds.
    #[serde(default = "default_chunk_secs")]
    pub chunk_secs: f32,
    /// How often to emit an interim (partial) on-device transcription of the
    /// in-progress chunk, in seconds. Set to 0 to disable partials.
    #[serde(default = "default_partial_secs")]
    pub partial_secs: f32,
    /// Capture the microphone. Almost always on; off records only system audio.
    #[serde(default = "default_true")]
    pub capture_microphone: bool,
    /// Microphone to capture. `None`/empty = whichever device is the system
    /// default *at the time* — and, because the capture layer re-resolves this
    /// on device loss, whichever becomes the default mid-recording.
    #[serde(default)]
    pub input_device: Option<String>,
    /// Also capture everything the machine is playing — the far side of a
    /// meeting call — and mix it with the microphone.
    ///
    /// **On by default.** Capturing only the microphone records half a meeting:
    /// you, and not the people you are meeting with. Bot-free capture of the far
    /// side is the product's whole point, so the useful configuration is the
    /// default one. It stays a visible toggle in Settings because it does record
    /// every sound the device plays.
    #[serde(default = "default_true")]
    pub capture_system_audio: bool,
    /// Output (Windows) or monitor/loopback input (macOS, Linux) to capture
    /// system audio from. `None` = the current default output.
    #[serde(default)]
    pub system_audio_device: Option<String>,
    /// Deprecated, read only to migrate pre-existing settings files. Capture is
    /// now microphone-first with system audio as an additive source, so there is
    /// no longer a "mix the mic back in" mode. See `load()`.
    #[serde(default)]
    pub mix_microphone: bool,
    /// Deprecated, see `mix_microphone`.
    #[serde(default)]
    pub microphone_device: Option<String>,
    /// Optional default instructions applied to every generated summary
    /// (e.g. "Do not include names mentioned in the meeting"). Empty = none.
    #[serde(default)]
    pub summary_instructions: String,
    /// Spoken language of the meeting, passed to whisper. Empty or `auto` =
    /// auto-detect; otherwise an ISO 639-1 code (e.g. `es`, `fr`, `de`, `ur`).
    #[serde(default)]
    pub transcription_language: String,
    /// Target language the AI summary should be written in, as a human-readable
    /// name (e.g. `Spanish`). Empty = match the transcript / model default.
    #[serde(default)]
    pub summary_language: String,
    /// Register a login item so the app is already running — and so already
    /// detecting meetings — before anyone opens it. Opt-in: adding a login item
    /// unasked is the kind of thing users resent, and on a managed fleet IT
    /// wants that decision. `serde(default)` keeps existing settings.json files
    /// loading, and existing users opted out on upgrade.
    #[serde(default)]
    pub start_at_login: bool,
    /// Generate a summary on its own as soon as a meeting finishes, instead of
    /// waiting for the user to ask. On by default: notes are the reason to
    /// record, and the manual button remains for regenerating.
    ///
    /// This is a privacy-relevant switch, not just a convenience one — with it
    /// on, every completed meeting's transcript is sent to the summarization
    /// server without a per-meeting action, so it is surfaced in Settings and
    /// can be turned off.
    ///
    /// On by default for a *fresh* install only: an install that predates this
    /// field is migrated to `false` so the upgrade does not flip a privacy
    /// default under someone. See `migrate_auto_summarize`.
    #[serde(default = "default_true")]
    pub auto_summarize: bool,
    /// Highest onboarding version this install has finished, or `0` for never.
    ///
    /// Lives here rather than in `localStorage` so it survives a webview data
    /// reset — being asked to redo setup because a cache was cleared is exactly
    /// the surprise onboarding exists to remove. Versioned rather than a plain
    /// bool so adding a permission later can reopen just the new step; see
    /// `permissions::ONBOARDING_VERSION`.
    #[serde(default)]
    pub onboarding_completed_version: u32,
    /// When true, poll for native call apps using the mic and show a floating
    /// "Take notes" prompt. Defaults on for macOS, off elsewhere.
    #[serde(default = "default_call_detection_enabled")]
    pub call_detection_enabled: bool,
    /// Minutes to wait after dismiss/start before re-prompting the same app.
    #[serde(default = "default_call_detection_cooldown_minutes")]
    pub call_detection_cooldown_minutes: u64,
    /// Seconds between call-detection polls.
    #[serde(default = "default_call_detection_poll_interval_secs")]
    pub call_detection_poll_interval_secs: u64,
    /// Process binary names to watch (e.g. `zoom.us`, `Slack`).
    #[serde(default = "default_call_detection_apps")]
    pub call_detection_apps: Vec<String>,
    /// Share anonymous usage statistics (counts, duration buckets, error
    /// categories — never recordings, transcripts, summaries, titles, names,
    /// or file paths; see `telemetry.rs` and docs/TELEMETRY.md). On by
    /// default with prominent disclosure in Settings; turning it off stops
    /// all emission immediately and deletes the random install id. The
    /// exporter also ships fully inert unless an endpoint is configured.
    #[serde(default = "default_true")]
    pub telemetry_enabled: bool,
}

fn default_server_url() -> String {
    String::new()
}
/// Whisper model names supported for on-device transcription.
pub const VALID_WHISPER_MODELS: [&str; 5] = ["tiny", "base", "small", "medium", "large-v3"];

pub const VALID_TRANSCRIPTION_ENGINES: [&str; 2] = ["whisper", "deepgram"];

fn default_transcription_engine() -> String {
    "deepgram".to_string()
}

pub fn normalize_transcription_engine(engine: &str) -> String {
    let e = engine.trim();
    if VALID_TRANSCRIPTION_ENGINES.contains(&e) {
        e.to_string()
    } else {
        default_transcription_engine()
    }
}

pub fn is_whisper_engine(engine: &str) -> bool {
    engine.trim().eq_ignore_ascii_case("whisper")
}

fn default_whisper_model() -> String {
    "base".to_string()
}

/// Return a supported whisper model name, falling back to the default for
/// unknown/corrupted values.
pub fn normalize_whisper_model(model: &str) -> String {
    let m = model.trim();
    if VALID_WHISPER_MODELS.contains(&m) {
        m.to_string()
    } else {
        default_whisper_model()
    }
}
fn default_true() -> bool {
    true
}
/// Default summarization model override sent to the Minutes server. The server
/// is now Fireworks-only (no Anthropic support), so this must be a Fireworks
/// model id.
fn default_anthropic_model() -> String {
    "accounts/fireworks/models/gpt-oss-120b".to_string()
}

fn default_chunk_secs() -> f32 {
    3.0
}
fn default_partial_secs() -> f32 {
    1.0
}

fn default_call_detection_enabled() -> bool {
    cfg!(target_os = "macos")
}

fn default_call_detection_cooldown_minutes() -> u64 {
    5
}

fn default_call_detection_poll_interval_secs() -> u64 {
    2
}

fn default_call_detection_apps() -> Vec<String> {
    vec![
        "zoom.us".into(),
        "Microsoft Teams".into(),
        "MSTeams".into(),
        "Slack".into(),
        "FaceTime".into(),
        "WhatsApp".into(),
        "Webex".into(),
        // Browser sentinels (detected via AppleScript tab URLs while mic is live)
        "google-meet".into(),
        "teams-web".into(),
    ]
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            server_url: default_server_url(),
            server_token: String::new(),
            whisper_model: default_whisper_model(),
            transcription_engine: default_transcription_engine(),
            diarization_enabled: true,
            export_markdown: true,
            anthropic_model: default_anthropic_model(),
            chunk_secs: default_chunk_secs(),
            partial_secs: default_partial_secs(),
            capture_microphone: true,
            input_device: None,
            capture_system_audio: true,
            system_audio_device: None,
            mix_microphone: false,
            microphone_device: None,
            summary_instructions: String::new(),
            transcription_language: String::new(),
            start_at_login: false,
            summary_language: String::new(),
            auto_summarize: true,
            // `0`, not the current version: a default-constructed Settings is
            // what a fresh install looks like, and that install has onboarded
            // nothing yet.
            onboarding_completed_version: 0,
            call_detection_enabled: default_call_detection_enabled(),
            call_detection_cooldown_minutes: default_call_detection_cooldown_minutes(),
            call_detection_poll_interval_secs: default_call_detection_poll_interval_secs(),
            call_detection_apps: default_call_detection_apps(),
            telemetry_enabled: true,
        }
    }
}

const ENV_API_URL: &str = "DESKSEC_API_URL";
const ENV_TOKEN: &str = "DESKSEC_TOKEN";
const LEGACY_ENV_API_URL: &str = "PARLEY_API_URL";
const LEGACY_ENV_TOKEN: &str = "PARLEY_TOKEN";

// `server_config_from_env` and its `env_var_set` helper lived here to feed a
// startup `eprintln!` that dumped where the server URL and token came from. The
// print is gone (it ran on every launch, before logging even existed), and the
// same four facts are already on `SettingsView` as `server_url_from_env` /
// `server_token_from_build` and friends, which Settings renders — so this was the
// only caller and the functions went with it.

fn resolve_env(primary: &str, legacy: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(legacy).ok())
        .and_then(|v| sanitize_value(&v))
}

/// Compile-time server URL baked in during `tauri build` (from CI).
pub fn embedded_api_url() -> Option<String> {
    option_env!("DESKSEC_EMBEDDED_API_URL")
        .or(option_env!("PARLEY_EMBEDDED_API_URL"))
        .and_then(sanitize_value)
}

/// Compile-time bearer token baked in during `tauri build` (from CI).
pub fn embedded_token() -> Option<String> {
    option_env!("DESKSEC_EMBEDDED_TOKEN")
        .or(option_env!("PARLEY_EMBEDDED_TOKEN"))
        .and_then(sanitize_value)
}

/// Push CI-embedded URL/token into settings + OS credential store. Returns
/// whether `settings.json` needs rewriting.
fn apply_embedded_server_config(settings: &mut Settings) -> bool {
    let mut changed = false;
    if let Some(url) = embedded_api_url() {
        if settings.server_url != url {
            settings.server_url = url.clone();
            changed = true;
        }
        if let Err(e) = crate::secrets::set_api_url(&url) {
            tracing::warn!("failed to store embedded server URL in OS credential store: {e}");
        }
    }
    if let Some(token) = embedded_token() {
        if settings.server_token != token {
            settings.server_token = token.clone();
            changed = true;
        }
        if let Err(e) = crate::secrets::set_token(&token) {
            tracing::warn!("failed to store embedded token in OS credential store: {e}");
        }
    }
    changed
}

/// Candidate locations for the project `.env`, in priority order.
fn env_file_candidates() -> Vec<PathBuf> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut v = vec![manifest.join("../.env"), manifest.join(".env")];
    if let Ok(cwd) = std::env::current_dir() {
        v.push(cwd.join(".env"));
        v.push(cwd.join("../.env"));
    }
    v
}

/// Re-read the managed server config from the `.env` file into the process
/// environment so edits are picked up without a full app restart. Skipped when
/// CI-embedded values exist — nothing local may override a release build.
pub fn reload_env_keys() {
    for path in env_file_candidates() {
        let Ok(iter) = dotenvy::from_path_iter(&path) else {
            continue;
        };
        for item in iter.flatten() {
            let (key, value) = item;
            let is_url_key = key == ENV_API_URL || key == LEGACY_ENV_API_URL;
            let is_token_key = key == ENV_TOKEN || key == LEGACY_ENV_TOKEN;
            if !is_url_key && !is_token_key {
                continue;
            }
            if is_url_key && embedded_api_url().is_some() {
                continue;
            }
            if is_token_key && embedded_token().is_some() {
                continue;
            }
            if !is_placeholder_key(&value) {
                std::env::set_var(&key, value);
            }
        }
        return; // first existing .env wins
    }
}

/// Returns `true` if a value is empty or an obvious placeholder/example.
pub fn is_placeholder_key(raw: &str) -> bool {
    let k = raw.trim();
    if k.is_empty() {
        return true;
    }
    let lower = k.to_lowercase();
    const MARKERS: [&str; 7] = [
        "your-",
        "-here",
        "placeholder",
        "xxxx",
        "...",
        "changeme",
        "example",
    ];
    MARKERS.iter().any(|m| lower.contains(m))
}

fn sanitize_value(raw: &str) -> Option<String> {
    if is_placeholder_key(raw) {
        None
    } else {
        Some(raw.trim().to_string())
    }
}

/// The bearer token and full meeting transcript are sent to the summarization
/// server. Refuse to do that over cleartext `http://` unless the host is
/// loopback (local dev), so a mis-typed or malicious remote URL can never leak
/// them in plaintext. Returns a user-facing error message when insecure.
pub fn validate_server_url(url: &str) -> Result<(), String> {
    let u = url.trim();
    if u.is_empty() {
        return Err("server URL is empty".into());
    }
    let parsed = reqwest::Url::parse(u).map_err(|_| "server URL is invalid".to_string())?;
    if parsed.host_str().is_none() {
        return Err("server URL must include a host".into());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("server URL must not include credentials".into());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("server URL must not include a query string or fragment".into());
    }

    match parsed.scheme() {
        "https" => Ok(()),
        "http"
            if parsed.host_str().is_some_and(|host| {
                host.eq_ignore_ascii_case("localhost")
                    || host.to_ascii_lowercase().ends_with(".localhost")
                    || host
                        .parse::<std::net::IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback())
            }) =>
        {
            Ok(())
        }
        "http" => Err(format!(
            "insecure server URL: {u}. Use https:// for remote servers (plain http:// is only allowed for localhost)."
        )),
        _ => Err(format!(
            "unsupported server URL scheme: {u}. Use https:// (or http:// for localhost)."
        )),
    }
}

fn resolve_server_url(stored: &str) -> Option<String> {
    embedded_api_url()
        .or_else(|| resolve_env(ENV_API_URL, LEGACY_ENV_API_URL))
        .or_else(|| sanitize_value(stored))
}

fn resolve_server_token(stored: &str) -> Option<String> {
    embedded_token()
        .or_else(|| resolve_env(ENV_TOKEN, LEGACY_ENV_TOKEN))
        .or_else(|| sanitize_value(stored))
}

impl Settings {
    /// Effective Minutes server URL (CI embed wins, then env, then stored).
    pub fn server_url(&self) -> String {
        resolve_server_url(&self.server_url).unwrap_or_default()
    }

    /// Effective Minutes access token (CI embed wins, then env, then keychain).
    pub fn server_token(&self) -> Option<String> {
        resolve_server_token(&self.server_token)
    }
}

fn config_path(config_dir: &Path) -> PathBuf {
    config_dir.join("settings.json")
}

/// Whether `settings.json` already existed when this process started.
///
/// Captured during `load` because `load` itself can resave (token migration,
/// model normalization, embedded config), so by the time anything else looks the
/// file always exists. Onboarding uses this to tell a genuinely fresh install
/// from an upgrade: both deserialize `onboarding_completed_version` to `0`, and
/// only the fresh one should see the full wizard.
static PREEXISTING_INSTALL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn was_preexisting_install() -> bool {
    // Defaults to "fresh" when `load` was never called (tests). An unreadable
    // file counts as fresh too — showing setup once too often is a smaller harm
    // than silently skipping the permissions the app needs.
    *PREEXISTING_INSTALL.get().unwrap_or(&false)
}

pub fn load(config_dir: &PathBuf) -> Settings {
    let path = config_path(config_dir);
    let raw = std::fs::read_to_string(&path).ok();
    let _ = PREEXISTING_INSTALL.set(raw.is_some());
    let mut settings: Settings = raw
        .as_ref()
        .and_then(|r| serde_json::from_str(r).ok())
        .unwrap_or_default();
    let mut needs_resave = false;

    // Parsed once and shared by the migrations below that need to tell "the file
    // said this" apart from "serde filled in a default" — a distinction the typed
    // `Settings` above has already erased.
    let raw_json = raw
        .as_ref()
        .and_then(|r| serde_json::from_str::<serde_json::Value>(r).ok());

    // Migrate legacy plaintext token from settings.json into the OS store. (The
    // server URL is no longer migrated in this direction — it's a normal
    // serialized field now, deserialized above like any other setting; see the
    // one-time keychain-fallback pull further down for the opposite-direction
    // migration, for anyone upgrading from when the URL lived in the keychain.)
    if let Some(v) = &raw_json {
        if let Some(token) = v.get("server_token").and_then(|t| t.as_str()) {
            let token = token.trim();
            if !token.is_empty() && !is_placeholder_key(token) {
                match crate::secrets::set_token(token) {
                    Ok(()) => needs_resave = true,
                    Err(e) => tracing::warn!("failed to migrate token to OS store: {e}"),
                }
            }
        }
    }

    if migrate_auto_summarize(&mut settings, raw_json.as_ref()) {
        needs_resave = true;
    }

    // Upgrade empty/legacy Anthropic model ids to the current Fireworks default.
    // The server no longer supports Anthropic at all, so any leftover "claude-*"
    // value (the old client default included) would otherwise fail every
    // summary with a model-not-found error.
    let model = settings.anthropic_model.trim();
    if model.is_empty() || model.starts_with("claude-") {
        settings.anthropic_model = default_anthropic_model();
        needs_resave = true;
    }

    // Guard against a corrupted/unknown whisper model string so first-run
    // downloads don't 404 on a bogus model file.
    let whisper = normalize_whisper_model(&settings.whisper_model);
    if whisper != settings.whisper_model {
        settings.whisper_model = whisper;
        needs_resave = true;
    }

    let engine = normalize_transcription_engine(&settings.transcription_engine);
    if engine != settings.transcription_engine {
        settings.transcription_engine = engine;
        needs_resave = true;
    }

    if migrate_capture_config(&mut settings) {
        needs_resave = true;
    }

    // Dev-only hydration from keychain when nothing was embedded at build time.
    if embedded_api_url().is_none()
        && resolve_env(ENV_API_URL, LEGACY_ENV_API_URL).is_none()
        && settings.server_url.trim().is_empty()
    {
        match crate::secrets::get_api_url() {
            Ok(Some(url)) if !url.trim().is_empty() => {
                settings.server_url = url;
                needs_resave = true;
            }
            Ok(_) => {}
            Err(e) => tracing::warn!("failed to read server URL from OS store: {e}"),
        }
    }
    if embedded_token().is_none() && resolve_env(ENV_TOKEN, LEGACY_ENV_TOKEN).is_none() {
        match crate::secrets::get_token() {
            Ok(Some(token)) => settings.server_token = token,
            Ok(None) => {}
            Err(e) => tracing::warn!("failed to read token from OS store: {e}"),
        }
    }

    // CI-embedded config is authoritative — sync into settings + keychain last.
    if apply_embedded_server_config(&mut settings) {
        needs_resave = true;
    }

    if needs_resave {
        let _ = save(config_dir, &settings);
    }

    settings
}

/// Turn auto-summarize off for an install that predates the setting. Returns
/// whether anything changed.
///
/// `auto_summarize` defaults to `true`, which is right for a fresh install:
/// notes are the reason to record. It is the wrong answer for an *existing*
/// install, because that install has been summarizing on demand only, and
/// inheriting the default would start sending every finished meeting's
/// transcript to the summarization server without the user ever asking. A
/// privacy default is not something to change under someone silently, so the
/// upgrade keeps the old behaviour and the Settings toggle offers the new one.
///
/// Distinguishing the two cases needs the raw JSON: by the time the typed
/// `Settings` exists, a `false` that came from the file and a `true` that serde
/// invented look identical. An absent key with a settings file present means an
/// upgrade; no file at all means a fresh install and the default stands.
///
/// The caller persists the result, so this decision is made exactly once and the
/// key is explicit from then on — including for anyone who later turns it on.
fn migrate_auto_summarize(settings: &mut Settings, raw: Option<&serde_json::Value>) -> bool {
    let Some(raw) = raw else {
        return false;
    };
    if raw.get("auto_summarize").is_some() {
        return false;
    }
    settings.auto_summarize = false;
    tracing::info!("existing install predates auto_summarize; leaving it off");
    true
}

/// Fold the old single-capture-source config into the mic-plus-system-audio
/// model. Returns whether anything changed.
///
/// The old shape had one `input_device` that had to be *replaced* by a loopback
/// device to record system audio, with `mix_microphone` adding the microphone
/// back in on top. That made "record my mic and what's playing" — the common
/// case — reachable only by an inversion nobody guessed, so capture is now
/// microphone-first with system audio as an additive source.
fn migrate_capture_config(settings: &mut Settings) -> bool {
    let Some(source) = settings.input_device.clone() else {
        return false;
    };
    if crate::audio::classify_device(&source) != crate::models::AudioDeviceKind::Loopback {
        return false;
    }

    settings.capture_system_audio = true;
    settings.system_audio_device = Some(source);
    // A loopback source with `mix_microphone` off meant system audio only.
    settings.capture_microphone = settings.mix_microphone;
    settings.input_device = if settings.mix_microphone {
        settings.microphone_device.take()
    } else {
        None
    };
    settings.mix_microphone = false;
    settings.microphone_device = None;
    tracing::info!(
        "migrated legacy loopback capture config to capture_system_audio (microphone: {})",
        settings.capture_microphone
    );
    true
}

pub fn save(config_dir: &PathBuf, settings: &Settings) -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir)?;
    let path = config_path(config_dir);
    let raw = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, raw)?;
    Ok(())
}

/// A settings view safe to send to the UI: the secret token is never returned,
/// but we report whether it is configured (via settings or env).
#[derive(Debug, Clone, Serialize)]
pub struct SettingsView {
    pub server_url: String,
    pub whisper_model: String,
    pub transcription_engine: String,
    pub diarization_enabled: bool,
    pub export_markdown: bool,
    pub anthropic_model: String,
    pub chunk_secs: f32,
    pub partial_secs: f32,
    pub capture_microphone: bool,
    pub input_device: Option<String>,
    pub capture_system_audio: bool,
    pub system_audio_device: Option<String>,
    pub summary_instructions: String,
    pub transcription_language: String,
    pub summary_language: String,
    pub auto_summarize: bool,
    pub onboarding_completed_version: u32,
    pub call_detection_enabled: bool,
    pub call_detection_cooldown_minutes: u64,
    pub call_detection_poll_interval_secs: u64,
    pub call_detection_apps: Vec<String>,
    pub call_detection_supported: bool,
    pub start_at_login: bool,
    /// Whether this platform has an OS share picker to hand a file to another
    /// app. A platform fact, not a stored preference, so it lives only on the
    /// view — the UI hides the action where it would not work.
    pub share_supported: bool,
    pub telemetry_enabled: bool,
    pub server_url_from_env: bool,
    pub server_url_from_build: bool,
    pub server_token_present: bool,
    pub server_token_from_env: bool,
    pub server_token_from_build: bool,
    /// Server-assigned id for this install, once registered.
    ///
    /// Shown read-only in Settings because revoking a device server-side means
    /// naming its id — without this there is no way for a user to tell support
    /// which row to revoke.
    pub device_id: Option<String>,
}

impl Settings {
    pub fn to_view(&self) -> SettingsView {
        let url_embedded = embedded_api_url().is_some();
        let token_embedded = embedded_token().is_some();
        let url_env = !url_embedded && resolve_env(ENV_API_URL, LEGACY_ENV_API_URL).is_some();
        let token_env = !token_embedded && resolve_env(ENV_TOKEN, LEGACY_ENV_TOKEN).is_some();
        SettingsView {
            server_url: self.server_url(),
            whisper_model: self.whisper_model.clone(),
            transcription_engine: self.transcription_engine.clone(),
            diarization_enabled: self.diarization_enabled,
            export_markdown: self.export_markdown,
            anthropic_model: self.anthropic_model.clone(),
            chunk_secs: self.chunk_secs,
            partial_secs: self.partial_secs,
            capture_microphone: self.capture_microphone,
            input_device: self.input_device.clone(),
            capture_system_audio: self.capture_system_audio,
            system_audio_device: self.system_audio_device.clone(),
            summary_instructions: self.summary_instructions.clone(),
            transcription_language: self.transcription_language.clone(),
            summary_language: self.summary_language.clone(),
            auto_summarize: self.auto_summarize,
            onboarding_completed_version: self.onboarding_completed_version,
            call_detection_enabled: self.call_detection_enabled,
            call_detection_cooldown_minutes: self.call_detection_cooldown_minutes,
            call_detection_poll_interval_secs: self.call_detection_poll_interval_secs,
            call_detection_apps: self.call_detection_apps.clone(),
            call_detection_supported: cfg!(target_os = "macos"),
            start_at_login: self.start_at_login,
            share_supported: crate::share::supported(),
            telemetry_enabled: self.telemetry_enabled,
            server_url_from_env: url_env,
            server_url_from_build: url_embedded,
            server_token_present: self.server_token().is_some(),
            server_token_from_env: token_env,
            server_token_from_build: token_embedded,
            device_id: crate::secrets::get_device_id().ok().flatten(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_whisper_model_keeps_valid_names() {
        for m in VALID_WHISPER_MODELS {
            assert_eq!(normalize_whisper_model(m), m);
        }
    }

    #[test]
    fn normalize_whisper_model_falls_back_for_unknown() {
        assert_eq!(normalize_whisper_model("not-a-model"), "base");
    }

    #[test]
    fn normalize_transcription_engine_defaults_unknown_to_deepgram() {
        assert_eq!(normalize_transcription_engine("deepgram"), "deepgram");
        assert_eq!(normalize_transcription_engine("whisper"), "whisper");
        assert_eq!(normalize_transcription_engine("invalid"), "deepgram");
    }

    #[test]
    fn validate_server_url_allows_https_and_localhost_http() {
        assert!(validate_server_url("https://api.desksec.example").is_ok());
        assert!(validate_server_url("http://localhost:8787").is_ok());
        assert!(validate_server_url("http://127.0.0.1:8787").is_ok());
        assert!(validate_server_url("http://foo.localhost").is_ok());
    }

    #[test]
    fn validate_server_url_rejects_cleartext_remote_and_bad_schemes() {
        assert!(validate_server_url("http://api.desksec.example").is_err());
        assert!(validate_server_url("http://192.168.1.5:8787").is_err());
        assert!(validate_server_url("ftp://example.com").is_err());
        assert!(validate_server_url("").is_err());
        assert!(validate_server_url("https://user:pass@example.com").is_err());
        assert!(validate_server_url("https://example.com?redirect=http://localhost").is_err());
        assert!(validate_server_url("https://example.com/#fragment").is_err());
        assert!(validate_server_url("https://").is_err());
    }

    #[test]
    fn migrates_loopback_source_with_mic_mixed_in() {
        let mut s = Settings {
            input_device: Some(crate::audio::wasapi_loopback_id("Speakers (Realtek)")),
            mix_microphone: true,
            microphone_device: Some("Headset Mic".into()),
            ..Settings::default()
        };
        assert!(migrate_capture_config(&mut s));
        assert!(s.capture_system_audio);
        assert_eq!(
            s.system_audio_device.as_deref(),
            Some(crate::audio::wasapi_loopback_id("Speakers (Realtek)").as_str())
        );
        assert!(s.capture_microphone);
        assert_eq!(s.input_device.as_deref(), Some("Headset Mic"));
        assert!(!s.mix_microphone);
        assert_eq!(s.microphone_device, None);
    }

    #[test]
    fn an_upgrade_does_not_silently_add_a_login_item() {
        // The field postdates every install in the field, so its absence must
        // read as "off". Defaulting the other way would register a launch agent
        // on someone's machine because they installed an update.
        let existing = r#"{"server_url":"https://example.test","whisper_model":"small"}"#;
        let loaded: Settings = serde_json::from_str(existing).expect("old settings must load");
        assert!(!loaded.start_at_login);
    }

    #[test]
    fn an_explicit_choice_survives_a_round_trip() {
        let chosen = Settings {
            start_at_login: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&chosen).expect("serialize");
        let back: Settings = serde_json::from_str(&json).expect("deserialize");
        assert!(back.start_at_login);
        // And it reaches the UI, which is the only way the toggle can show it.
        assert!(back.to_view().start_at_login);
    }

    #[test]
    fn migrates_loopback_only_source_to_system_audio_only() {
        let mut s = Settings {
            input_device: Some("Stereo Mix (Realtek Audio)".into()),
            mix_microphone: false,
            ..Settings::default()
        };
        assert!(migrate_capture_config(&mut s));
        assert!(s.capture_system_audio);
        assert!(!s.capture_microphone);
        assert_eq!(s.input_device, None);
    }

    #[test]
    fn leaves_microphone_source_untouched() {
        let mut s = Settings {
            input_device: Some("Microphone Array (Intel SST)".into()),
            ..Settings::default()
        };
        assert!(!migrate_capture_config(&mut s));
        assert!(s.capture_microphone);
        // Untouched means left at the default, not forced off.
        assert_eq!(
            s.capture_system_audio,
            Settings::default().capture_system_audio
        );
        assert_eq!(
            s.input_device.as_deref(),
            Some("Microphone Array (Intel SST)")
        );
    }

    #[test]
    fn system_audio_is_on_by_default_and_survives_a_settings_file_without_it() {
        assert!(Settings::default().capture_system_audio);
        // A settings.json written before these fields existed must still opt in:
        // capturing only the microphone records half a meeting.
        let older: Settings = serde_json::from_str(
            r#"{"server_url":"http://localhost:8787","whisper_model":"base","input_device":null}"#,
        )
        .expect("older settings files must still deserialize");
        assert!(older.capture_microphone);
        assert!(older.capture_system_audio);
    }

    #[test]
    fn telemetry_defaults_on_and_survives_a_settings_file_without_it() {
        assert!(Settings::default().telemetry_enabled);
        // Files written before the field existed keep the disclosed default…
        let older: Settings = serde_json::from_str(r#"{"whisper_model":"base"}"#).unwrap();
        assert!(older.telemetry_enabled);
        // …and an explicit opt-out is respected on load.
        let opted_out: Settings = serde_json::from_str(r#"{"telemetry_enabled":false}"#).unwrap();
        assert!(!opted_out.telemetry_enabled);
    }

    #[test]
    fn onboarding_version_reads_zero_when_the_field_is_absent() {
        // A settings file written before onboarding existed must report "never
        // onboarded" rather than failing to deserialize or defaulting to the
        // current version (which would skip setup for everyone upgrading).
        assert_eq!(Settings::default().onboarding_completed_version, 0);
        let older: Settings = serde_json::from_str(r#"{"whisper_model":"base"}"#).unwrap();
        assert_eq!(older.onboarding_completed_version, 0);

        let stamped: Settings =
            serde_json::from_str(r#"{"onboarding_completed_version":3}"#).unwrap();
        assert_eq!(stamped.onboarding_completed_version, 3);
        assert_eq!(stamped.to_view().onboarding_completed_version, 3);
    }

    #[test]
    fn auto_summarize_defaults_on_when_the_field_is_absent() {
        assert!(Settings::default().auto_summarize);
        // Deserialization alone defaults it on. Whether an *existing* install
        // keeps that default is a separate decision made by
        // `migrate_auto_summarize`, covered below — this only pins the serde
        // default, so a settings file without the key still parses.
        let older: Settings = serde_json::from_str(r#"{"whisper_model":"base"}"#).unwrap();
        assert!(older.auto_summarize);
        // Turning it off sticks, since it governs whether transcripts are sent to
        // the summarization server without an explicit action.
        let opted_out: Settings = serde_json::from_str(r#"{"auto_summarize":false}"#).unwrap();
        assert!(!opted_out.auto_summarize);
        assert!(!opted_out.to_view().auto_summarize);
    }

    #[test]
    fn auto_summarize_migrates_off_for_an_install_that_predates_it() {
        // The case that matters: a real settings file with no `auto_summarize`
        // key. Inheriting the `true` default here would start uploading every
        // finished transcript for someone who had only ever summarized on demand.
        let raw: serde_json::Value = serde_json::from_str(r#"{"whisper_model":"base"}"#).unwrap();
        let mut settings: Settings = serde_json::from_value(raw.clone()).unwrap();
        assert!(settings.auto_summarize, "serde default should be on");

        assert!(migrate_auto_summarize(&mut settings, Some(&raw)));
        assert!(!settings.auto_summarize);
    }

    #[test]
    fn auto_summarize_migration_leaves_an_explicit_choice_alone() {
        // Both explicit values are already decisions the user (or a previous
        // migration) made, so neither is second-guessed. `true` in particular
        // must survive: re-running the migration over its own output would
        // otherwise turn the feature off again on every launch.
        for (json, expected) in [
            (r#"{"auto_summarize":true}"#, true),
            (r#"{"auto_summarize":false}"#, false),
        ] {
            let raw: serde_json::Value = serde_json::from_str(json).unwrap();
            let mut settings: Settings = serde_json::from_value(raw.clone()).unwrap();

            assert!(
                !migrate_auto_summarize(&mut settings, Some(&raw)),
                "{json} should not be migrated"
            );
            assert_eq!(settings.auto_summarize, expected);
        }
    }

    #[test]
    fn auto_summarize_migration_leaves_a_fresh_install_on() {
        // No settings file at all — nobody's expectations to preserve, so the
        // default stands and new users get summaries without hunting for a toggle.
        let mut settings = Settings::default();
        assert!(!migrate_auto_summarize(&mut settings, None));
        assert!(settings.auto_summarize);
    }

    #[test]
    fn placeholder_detection() {
        assert!(is_placeholder_key(""));
        assert!(is_placeholder_key("your-token-here"));
        assert!(is_placeholder_key("changeme"));
        assert!(!is_placeholder_key("a6535f0a04bdf489f737db5180aecc5e"));
    }
}
