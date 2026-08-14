//! Privacy-preserving usage/error telemetry.
//!
//! Design rules (see `docs/TELEMETRY.md` for the full event schema):
//!
//! - **Metadata only, never content.** No transcript text, no summary text,
//!   no meeting titles, no participant names, no file paths, no audio. Every
//!   event attribute key must be on [`ALLOWED_ATTR_KEYS`]; anything else is
//!   silently dropped, and a unit test guards the allowlist against
//!   content-shaped keys ever being added.
//! - **Pseudonymous.** The only identifier is a random UUID generated on
//!   first use and stored in a plain file in the app config dir. It is not
//!   derived from the machine, hostname, username, or anything else, and
//!   deleting the file (done automatically when the user opts out) resets it.
//! - **Asynchronous, never synchronous.** [`event`] does an atomic check, an
//!   allowlist filter, and a bounded-channel `try_send`. When the queue is
//!   full the event is dropped. All I/O — network *and* spool disk writes —
//!   happens on the background export worker. Nothing here can ever block or
//!   slow the recording pipeline, and nothing here may panic.
//! - **Opt-out is absolute.** The toggle gates emission, export *and* the
//!   disk spool: a disabled gate stops events reaching the queue, the export
//!   worker discards anything already queued rather than sending it, and the
//!   spool directory is deleted so nothing already written to disk can be
//!   sent later. "Off" can never mean "off until the collector comes back".
//! - **Durable, but bounded.** A failed export is written to a spool
//!   directory and retried with exponential backoff plus jitter, so a laptop
//!   that goes offline (or is closed mid-flush) does not lose its events.
//!   The spool is hard-capped by both batch count and total bytes
//!   ([`SPOOL_MAX_BATCHES`] / [`SPOOL_MAX_BYTES`]); when it is full the
//!   *oldest* batches are dropped and the loss is reported as a
//!   `telemetry_spool_dropped` event, so a gap is never silently invisible.
//!   Spool files hold OTLP payloads that are content-free by construction —
//!   the same allowlisted metadata that would have gone over the wire, never
//!   meeting data.
//! - **Inert unless credentialed.** The destination is compiled in
//!   ([`DEFAULT_OTLP_ENDPOINT`], [`GRAFANA_INSTANCE_ID`]) because neither is
//!   secret. Internal release builds also receive the Grafana write token from
//!   CI; local/runtime configuration can override it — see [`telemetry_token`].
//!   Without either credential, [`init`] starts no worker, queue, spool, or
//!   network request.
//!
//! Transport: OTLP/HTTP **JSON** logs (`/v1/logs`), hand-built with
//! `serde_json` and POSTed with the `reqwest` client the app already ships.
//! JSON is confirmed accepted by the destination — the live Grafana Cloud
//! OTLP gateway answered a hand-built payload with HTTP 204 — so protobuf is
//! not required and this choice does not need revisiting.
//! We deliberately do not depend on the `opentelemetry`/`opentelemetry-otlp`
//! SDK crates: this crate's dependency graph pins `ort`, `rusqlite`, and
//! `cpal` for load-bearing reasons (see CONTRIBUTING.md), and the OTel SDK
//! stack drags in a large tree (`http`, `tonic`/`prost` for gRPC, its own
//! `reqwest` feature matrix) for what is, here, a few dozen JSON structs a
//! day. The payload follows the OTLP JSON encoding, so a stock OpenTelemetry
//! Collector (→ Loki) accepts it unchanged, and the tiny public surface
//! (`telemetry::event(name, attrs)`) means the transport can be swapped for
//! the real SDK later without touching any call site.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Bump when the meaning of an event or attribute changes.
pub const SCHEMA_VERSION: i64 = 1;

/// Events waiting to be exported. Beyond this, new events are dropped.
const QUEUE_CAPACITY: usize = 256;
/// Export at least this often while events are pending.
const FLUSH_INTERVAL_SECS: u64 = 30;
/// Export immediately once a batch reaches this size.
const MAX_BATCH: usize = 64;
/// One attempt per batch, bounded hard — telemetry must never hold sockets
/// open behind the recording pipeline's HTTP traffic.
const EXPORT_TIMEOUT_SECS: u64 = 10;

const INSTALL_ID_FILE: &str = "telemetry_install_id";
/// Spool directory name, under the app config dir (next to the install id).
const SPOOL_DIR: &str = "telemetry_spool";
/// Finished spool batches use this extension; partially written ones use
/// `.json.tmp` and are ignored, so a crash mid-write can never be read back.
const SPOOL_EXT: &str = "json";

/// Hard cap on spooled batches. Beyond this the **oldest** are dropped: an
/// offline laptop must not grow an unbounded queue of stale telemetry.
pub const SPOOL_MAX_BATCHES: usize = 200;
/// Hard cap on total spool bytes on disk, enforced together with
/// [`SPOOL_MAX_BATCHES`] (whichever binds first).
pub const SPOOL_MAX_BYTES: u64 = 5 * 1024 * 1024;

/// Grafana Cloud OTLP gateway for this org. Non-secret, so it is hardcoded:
/// the app must work with zero per-machine configuration. The code appends
/// `/v1/logs`, which is the correct signal URL for this base. Overridable via
/// `OTEL_EXPORTER_OTLP_ENDPOINT` (see [`exporter_config_from_env`]).
///
/// Confirmed working: this gateway accepts OTLP/HTTP **JSON** and answered a
/// hand-built payload with HTTP 204. Protobuf is not required, so there is no
/// reason to pull in the heavyweight OpenTelemetry SDK stack.
pub const DEFAULT_OTLP_ENDPOINT: &str = "https://otlp-gateway-prod-eu-west-2.grafana.net/otlp";

/// Grafana Cloud instance id. This is the HTTP Basic *username*; the password
/// is the write token, which is never in this repository (see
/// [`telemetry_token`]). Non-secret on its own, so it is hardcoded too.
pub const GRAFANA_INSTANCE_ID: &str = "1549080";

/// First retry window after a failed export.
pub const RETRY_BASE_SECS: u64 = 30;
/// Backoff ceiling. Past this the worker retries roughly every 15–30 min,
/// which is cheap enough to leave running for days on an offline machine.
pub const RETRY_MAX_SECS: u64 = 30 * 60;

/// Every attribute key any event may carry. **Keys are the privacy boundary**:
/// an attribute whose key is not listed here is dropped before it ever
/// reaches the queue. When adding a key, re-read the module doc — numbers,
/// buckets, and closed category sets only. Never free text, never content.
pub const ALLOWED_ATTR_KEYS: &[&str] = &[
    "schema_version",
    // configuration mix (categories / flags only)
    "engine",                 // "whisper" | "deepgram"
    "whisper_model",          // "tiny" .. "large-v3"
    "diarization",            // bool
    "capture_microphone",     // bool
    "capture_system_audio",   // bool
    "call_detection_enabled", // bool
    "export_markdown",        // bool
    "theme",                  // "system" | "light" | "dark"
    // engagement / cost proxies (buckets only, never raw values)
    "duration_bucket",          // e.g. "5-15m"
    "transcript_length_bucket", // e.g. "5k-20k" (characters, bucketed)
    "format",                   // export format: "md" | "txt" | "docx"
    "trigger",                  // "manual" | "call_prompt"
    "prompt_kind",              // "manual" | "call"
    // performance (buckets only — see `latency_bucket_ms`)
    "app_startup_duration_bucket", // process start -> UI ready
    // Wall-clock time of a summary request. NOT named `summary_duration_*`:
    // the allowlist guard test reserves the `summary_` prefix for content
    // fields (`summary_text`, `summary_instructions`), and that guard is
    // deliberately not weakened for a naming preference.
    "summarize_duration_bucket",
    "transcription_latency_bucket", // on-device: time to transcribe one chunk
    "download_duration_bucket",     // whisper model download wall-clock
    "outcome",                      // "success" | "failed" | "cancelled"
    // reliability
    // OTel semconv `error.type`: network | timeout | auth | server | internal
    "error.type",
    "area",            // "summary" | "recording_start" | "export" | ...
    "recovered_count", // interrupted meetings recovered at startup
    "granted",         // OS microphone permission outcome (bool)
    "dropped",         // spooled batches lost to the spool bounds
];

/// Every **resource** attribute key a batch may carry.
///
/// # The cardinality rule
///
/// Resource attributes are the ones an OTLP-to-Loki pipeline is most likely
/// to promote into Loki **index labels**, and a label whose value is unique
/// per user or per launch is a cardinality blow-up that degrades the whole
/// stack — not just our data. So this list is restricted to values from small,
/// finite sets: a constant, an app version, an OS name and coarse version, a
/// CPU architecture, a core count, and a build channel.
///
/// **High-cardinality identifiers do not belong here.** The install id and the
/// session id go on the log record instead — see [`IDENTITY_ATTR_KEYS`]. This
/// is enforced defensively because we cannot inspect the gateway's exact
/// label-promotion config, and it is cheaper to be safe than to find out.
///
/// Unlike [`ALLOWED_ATTR_KEYS`] these are not caller-supplied at all:
/// [`build_export_payload`] builds this exact list from a [`Resource`], and a
/// test pins the two together.
pub const ALLOWED_RESOURCE_ATTR_KEYS: &[&str] = &[
    "service.name",      // constant "desksec"
    "service.namespace", // constant — groups Minutes inside the shared stack
    "service.version",   // app version, e.g. "0.1.0"
    "os.type",           // "macos" | "windows" | "linux" | ...
    "os.version",        // coarse major.minor, e.g. "15.3" — never a build id
    "device.arch",       // "aarch64" | "x86_64" | ...
    "device.cpu.cores",  // small int
    "app.channel",       // "debug" | "release"
];

/// Identifiers attached to every **log record**, never to the resource.
///
/// Both are unique — per launch and per install — which is exactly why they
/// live here: log record attributes are payload, not index labels, so they
/// cost storage rather than cardinality. See [`ALLOWED_RESOURCE_ATTR_KEYS`].
pub const IDENTITY_ATTR_KEYS: &[&str] = &[
    "session.id",         // random UUID, regenerated every launch
    "desksec.install.id", // random UUID, resettable, not machine-derived
];

/// `service.name`. Free on the shared Grafana stack (the neighbours are
/// `codex_cli_rs`, `claude-code-desktop` and `cowork`), so `{service_name="desksec"}`
/// isolates this app's data.
pub const SERVICE_NAME: &str = "desksec";
/// `service.namespace`: groups Minutes with the org's other services. A
/// constant, so it is safe to promote to a label.
pub const SERVICE_NAMESPACE: &str = "amalitech";

/// The closed set of value shapes an attribute may carry. There is no
/// free-form escape hatch on purpose: strings should be members of small
/// category sets or bucket labels, never sentences.
#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

impl From<&str> for AttrValue {
    fn from(v: &str) -> Self {
        AttrValue::Str(v.to_string())
    }
}
impl From<String> for AttrValue {
    fn from(v: String) -> Self {
        AttrValue::Str(v)
    }
}
impl From<i64> for AttrValue {
    fn from(v: i64) -> Self {
        AttrValue::Int(v)
    }
}
impl From<bool> for AttrValue {
    fn from(v: bool) -> Self {
        AttrValue::Bool(v)
    }
}

/// One telemetry event, already filtered to allowlisted attribute keys.
#[derive(Debug, Clone)]
pub struct Event {
    pub name: &'static str,
    pub attrs: Vec<(&'static str, AttrValue)>,
    pub time_unix_nano: u128,
}

impl Event {
    /// Build an event, dropping any attribute whose key is not allowlisted.
    pub fn new(name: &'static str, attrs: &[(&'static str, AttrValue)]) -> Self {
        let attrs = attrs
            .iter()
            .filter(|(k, _)| {
                let ok = ALLOWED_ATTR_KEYS.contains(k);
                if !ok {
                    tracing::debug!("telemetry: dropping non-allowlisted attribute key {k:?}");
                }
                ok
            })
            .cloned()
            .collect();
        let time_unix_nano = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Event {
            name,
            attrs,
            time_unix_nano,
        }
    }
}

/// Static, content-free description of this install.
///
/// Split across two places in the payload on purpose — see
/// [`ALLOWED_RESOURCE_ATTR_KEYS`] and [`IDENTITY_ATTR_KEYS`]. The bounded,
/// low-cardinality fields become OTLP *resource* attributes; the two unique
/// identifiers become *log record* attributes instead.
#[derive(Debug, Clone)]
pub struct Resource {
    /// Random UUID, generated locally, resettable. NOT a machine id.
    /// **Log record attribute** — unique per install, never a resource
    /// attribute.
    pub install_id: String,
    /// Random UUID regenerated on every launch. Groups events *within* one
    /// app run (which is what makes funnel analysis possible) without
    /// linking one run to the next. **Log record attribute** — unique per
    /// launch, never a resource attribute.
    pub session_id: String,
    pub app_version: String,
    /// `std::env::consts::OS`: "macos" | "windows" | "linux" | ...
    pub os: String,
    /// Coarse OS version, major.minor only (e.g. "15.3"), or "unknown".
    pub os_version: String,
    /// `std::env::consts::ARCH`. Apple Silicon vs Intel is the single biggest
    /// factor in on-device Whisper speed, so this is what makes a
    /// "transcription is slow" report interpretable.
    pub arch: String,
    /// Logical CPU count, 0 when unavailable. Same purpose as `arch`.
    pub cpu_cores: i64,
    /// "debug" | "release" — lets developer noise be filtered out of
    /// adoption numbers.
    pub channel: String,
}

// ---------------------------------------------------------------------------
// Environment description (content-free by construction)
// ---------------------------------------------------------------------------

/// Reduce a raw OS version string to at most `major.minor`.
///
/// Build numbers and patch levels are deliberately discarded: a full string
/// like macOS "15.3.1 (24D70)" is close to unique in a population of a few
/// hundred installs and would undo the pseudonymity the install id exists to
/// provide. Non-numeric input degrades to "unknown", never to free text.
pub fn coarsen_os_version(raw: &str) -> String {
    let mut parts = raw.trim().split('.').filter_map(|p| {
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        (!digits.is_empty()).then_some(digits)
    });
    match (parts.next(), parts.next()) {
        (Some(major), Some(minor)) => format!("{major}.{minor}"),
        (Some(major), None) => major,
        _ => "unknown".to_string(),
    }
}

/// Coarse OS version for the `os.version` resource attribute.
///
/// Windows is special-cased: it reports `10.0.<build>` for *both* Windows 10
/// and Windows 11, and only the build number separates them — but a build
/// number is exactly the kind of near-unique value [`coarsen_os_version`]
/// exists to strip. So we collapse it to the marketing major version ("10"
/// or "11") instead, which is the useful, non-identifying half.
pub fn os_version() -> String {
    let info = os_info::get();
    match info.version() {
        os_info::Version::Semantic(major, minor, build) => {
            if info.os_type() == os_info::Type::Windows && *major == 10 {
                if *build >= 22000 { "11" } else { "10" }.to_string()
            } else {
                format!("{major}.{minor}")
            }
        }
        other => coarsen_os_version(&other.to_string()),
    }
}

/// Logical CPU count, or 0 when the platform will not say.
pub fn cpu_cores() -> i64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(0)
}

/// Build channel. Free, and it keeps developer machines out of adoption
/// numbers instead of quietly inflating them.
pub fn app_channel() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

static SESSION_ID: OnceLock<String> = OnceLock::new();

/// Random id for this process. Regenerated every launch and never persisted,
/// so it groups events inside one run without linking runs to each other.
pub fn session_id() -> &'static str {
    SESSION_ID.get_or_init(|| uuid::Uuid::new_v4().to_string())
}

/// 64 random bits, reusing the `getrandom`-backed generator `uuid` already
/// brings in rather than adding a `rand` dependency for jitter and spool file
/// names. Only fully random bytes are taken — v4 UUIDs fix the version nibble
/// in byte 6 and the variant bits in byte 8.
fn random_u64() -> u64 {
    let b = uuid::Uuid::new_v4().into_bytes();
    u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[9], b[10]])
}

/// Where and how to export. `None` (unset/invalid env) = telemetry is inert.
#[derive(Debug, Clone, PartialEq)]
pub struct ExporterConfig {
    /// Full OTLP/HTTP logs URL, e.g. `https://otlp.grafana.internal/v1/logs`.
    pub endpoint: String,
    /// Extra request headers, e.g. auth / tenant id.
    pub headers: Vec<(String, String)>,
}

/// Parse endpoint + headers strings into a config. Returns `None` — a silent
/// no-op — for anything missing, placeholder-looking, or malformed. This must
/// never panic: it runs on every startup with arbitrary user environments.
pub fn parse_exporter_config(
    endpoint: Option<&str>,
    headers: Option<&str>,
) -> Option<ExporterConfig> {
    let raw = endpoint?.trim();
    if raw.is_empty() || crate::settings::is_placeholder_key(raw) {
        return None;
    }
    if !(raw.starts_with("https://") || raw.starts_with("http://")) {
        return None;
    }
    // Accept either a full signal URL or a base OTLP endpoint (the standard
    // OTEL_EXPORTER_OTLP_ENDPOINT is a base; the logs path is appended).
    let endpoint = if raw.ends_with("/v1/logs") {
        raw.to_string()
    } else {
        format!("{}/v1/logs", raw.trim_end_matches('/'))
    };
    // OTel env-var header syntax: "key1=value1,key2=value2".
    let headers = headers
        .unwrap_or("")
        .split(',')
        .filter_map(|pair| {
            let pair = pair.trim();
            let (k, v) = pair.split_once('=')?;
            let (k, v) = (k.trim(), v.trim());
            if k.is_empty() || v.is_empty() {
                None
            } else {
                Some((k.to_string(), v.to_string()))
            }
        })
        .collect();
    Some(ExporterConfig { endpoint, headers })
}

/// Trim, then reject empty and placeholder-looking values.
fn usable_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || crate::settings::is_placeholder_key(trimmed) {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Grafana Cloud write token — the HTTP Basic *password*.
///
/// **Never stored in this repository.** Runtime `DESKSEC_OTLP_TOKEN` (also
/// picked up from `.env` via `dotenvy`) takes precedence. Internal release
/// builds fall back to the value CI embeds with `option_env!` so telemetry is
/// live without per-device provisioning.
///
/// The embedded shared token is extractable from the binary. This is an
/// accepted exception for the internal-only, VPN-distributed app and requires
/// rotation if an installer leaves that trust boundary. With no runtime or
/// embedded token, telemetry remains switched off.
pub fn telemetry_token() -> Option<String> {
    std::env::var("DESKSEC_OTLP_TOKEN")
        .ok()
        .and_then(|v| usable_value(&v))
        .or_else(|| option_env!("DESKSEC_OTLP_TOKEN").and_then(usable_value))
}

/// `Authorization: Basic base64("<instance id>:<token>")`, which is what the
/// Grafana Cloud OTLP gateway expects. Assembled here from its two halves so
/// the token never has to exist anywhere in pre-encoded form.
fn basic_auth_header(instance_id: &str, token: &str) -> String {
    use base64::Engine as _;
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("{instance_id}:{token}"));
    format!("Basic {encoded}")
}

/// Assemble the exporter config from its inputs. Split out from
/// [`exporter_config_from_env`] so the precedence rules are testable without
/// mutating process-global environment variables.
///
/// Precedence, highest first:
///
/// 1. An explicit `endpoint` / `headers` pair from the environment. Headers
///    given this way win outright — that is how a self-hosted collector or a
///    staging tenant is targeted, and it must work with no Grafana token at
///    all.
/// 2. `token`, combined with `instance_id` into a Basic auth header against
///    the (hardcoded, non-secret) default endpoint.
/// 3. Neither ⇒ `None`, i.e. completely inert. We never POST unauthenticated:
///    the gateway would reject every batch, and a spool full of doomed
///    batches is worse than sending nothing.
pub fn exporter_config_from_parts(
    endpoint: Option<&str>,
    headers: Option<&str>,
    instance_id: &str,
    token: Option<&str>,
) -> Option<ExporterConfig> {
    let endpoint = endpoint
        .and_then(|e| {
            let trimmed = e.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .unwrap_or(DEFAULT_OTLP_ENDPOINT);

    if let Some(raw) = headers.map(str::trim).filter(|h| !h.is_empty()) {
        let config = parse_exporter_config(Some(endpoint), Some(raw))?;
        // Headers were supplied but parsed to nothing usable. That is a
        // misconfiguration, not a reason to fall back to shipping data
        // somewhere the operator did not ask for.
        if config.headers.is_empty() {
            return None;
        }
        return Some(config);
    }

    let token = token?;
    let mut config = parse_exporter_config(Some(endpoint), None)?;
    config.headers.push((
        "Authorization".to_string(),
        basic_auth_header(instance_id, token),
    ));
    Some(config)
}

/// Read the exporter config from the environment, falling back to the
/// hardcoded Grafana Cloud defaults. Minutes-specific variables win; the
/// standard OpenTelemetry ones are honored as a fallback.
pub fn exporter_config_from_env() -> Option<ExporterConfig> {
    let endpoint = std::env::var("DESKSEC_TELEMETRY_ENDPOINT")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_LOGS_ENDPOINT"))
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT"))
        .ok();
    let headers = std::env::var("DESKSEC_TELEMETRY_HEADERS")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_HEADERS"))
        .ok();
    exporter_config_from_parts(
        endpoint.as_deref(),
        headers.as_deref(),
        GRAFANA_INSTANCE_ID,
        telemetry_token().as_deref(),
    )
}

// ---------------------------------------------------------------------------
// Install ID
// ---------------------------------------------------------------------------

fn install_id_path(config_dir: &Path) -> std::path::PathBuf {
    config_dir.join(INSTALL_ID_FILE)
}

fn looks_like_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Read the pseudonymous install id, creating (and persisting) a fresh random
/// UUID on first use or when the stored value is unreadable/corrupt. Returns
/// `None` only when the id cannot be persisted — in that case telemetry
/// stays inert rather than sending an id that changes every launch.
pub fn load_or_create_install_id(config_dir: &Path) -> Option<String> {
    let path = install_id_path(config_dir);
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let existing = existing.trim().to_string();
        if looks_like_uuid(&existing) {
            return Some(existing);
        }
    }
    let fresh = uuid::Uuid::new_v4().to_string();
    if std::fs::create_dir_all(config_dir).is_err() {
        return None;
    }
    match std::fs::write(&path, &fresh) {
        Ok(()) => Some(fresh),
        Err(e) => {
            tracing::debug!("telemetry: could not persist install id: {e}");
            None
        }
    }
}

/// Delete the stored install id. The next enable/launch generates a fresh
/// one, unlinkable to the old. Called automatically when the user opts out.
pub fn reset_install_id(config_dir: &Path) {
    let _ = std::fs::remove_file(install_id_path(config_dir));
}

// ---------------------------------------------------------------------------
// Disk spool
// ---------------------------------------------------------------------------

/// A bounded, on-disk queue of export batches that could not be delivered.
///
/// Each file is one complete OTLP/JSON payload — the same bytes that would
/// have been POSTed, and therefore content-free by construction: allowlisted
/// metadata only, never transcript, summary, title, or path data. Files are
/// named by creation time so lexical order is chronological order, which is
/// what "drop the oldest" and "drain oldest first" rely on.
///
/// Every method is best-effort and infallible from the caller's point of
/// view: a full disk, a read-only config dir, or a partially written file
/// degrades to "this batch is lost", never to an error the app has to handle
/// and never to a panic.
pub struct Spool {
    dir: PathBuf,
    max_batches: usize,
    max_bytes: u64,
}

/// Monotonic within one process, so two batches spooled in the same clock
/// nanosecond still sort in the order they were produced.
static SPOOL_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Spool {
    pub fn new(config_dir: &Path) -> Self {
        Spool::with_limits(config_dir, SPOOL_MAX_BATCHES, SPOOL_MAX_BYTES)
    }

    /// Same spool with explicit bounds. Used by tests to exercise eviction
    /// without writing 5 MB.
    pub fn with_limits(config_dir: &Path, max_batches: usize, max_bytes: u64) -> Self {
        Spool {
            dir: config_dir.join(SPOOL_DIR),
            max_batches,
            max_bytes,
        }
    }

    /// Complete batches, oldest first. Anything that is not a finished
    /// `.json` file — a `.json.tmp` left by an interrupted write, a
    /// subdirectory, an entry that cannot be stat'd — is ignored rather than
    /// treated as an error.
    pub fn batches(&self) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(SPOOL_EXT) && p.is_file())
            .collect();
        files.sort();
        files
    }

    /// Number of spooled batches and the bytes they occupy.
    pub fn stats(&self) -> (usize, u64) {
        let files = self.batches();
        let bytes = files.iter().map(|p| file_len(p)).sum();
        (files.len(), bytes)
    }

    /// Persist one serialized batch, then trim the spool back inside its
    /// bounds. Returns how many **older** batches had to be dropped to make
    /// room, so the caller can report the loss instead of hiding it.
    pub fn push(&self, payload: &[u8]) -> u64 {
        if std::fs::create_dir_all(&self.dir).is_err() {
            tracing::debug!("telemetry: cannot create spool dir; batch dropped");
            return 0;
        }
        // The nanosecond timestamp orders files across runs; the per-process
        // counter orders files *within* a run, since two pushes can land in
        // the same nanosecond on a coarse clock. Both are zero-padded so
        // lexical order (what `batches()` sorts by) is chronological order.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = SPOOL_SEQ.fetch_add(1, Ordering::Relaxed);
        let stem = format!("{nanos:039}-{seq:016x}");
        // Write-then-rename: a crash mid-write leaves a `.json.tmp` that
        // `batches()` ignores, never a truncated batch that looks complete.
        let tmp = self.dir.join(format!("{stem}.{SPOOL_EXT}.tmp"));
        let final_path = self.dir.join(format!("{stem}.{SPOOL_EXT}"));
        if std::fs::write(&tmp, payload).is_err() || std::fs::rename(&tmp, &final_path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            tracing::debug!("telemetry: could not write spool file; batch dropped");
            return 0;
        }
        self.enforce_bounds()
    }

    /// Drop oldest-first until both bounds hold. Returns the number dropped.
    fn enforce_bounds(&self) -> u64 {
        let files = self.batches();
        let mut bytes: u64 = files.iter().map(|p| file_len(p)).sum();
        let mut dropped = 0;
        let mut i = 0;
        // `i` advances unconditionally, so a file that refuses to be deleted
        // costs one wasted iteration rather than an infinite loop.
        while i < files.len() && (files.len() - i > self.max_batches || bytes > self.max_bytes) {
            let size = file_len(&files[i]);
            if std::fs::remove_file(&files[i]).is_ok() {
                bytes = bytes.saturating_sub(size);
                dropped += 1;
            }
            i += 1;
        }
        if dropped > 0 {
            tracing::debug!("telemetry: spool full, dropped {dropped} oldest batch(es)");
        }
        dropped
    }

    /// Forget one batch (delivered, or permanently undeliverable).
    fn remove(&self, path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    /// Delete everything, including partial writes. Called on opt-out: if the
    /// spool survived the toggle, "telemetry off" would still leak whatever
    /// was queued the moment the collector came back.
    pub fn purge(&self) {
        if let Err(e) = std::fs::remove_dir_all(&self.dir) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::debug!("telemetry: could not purge spool: {e}");
            }
        }
    }
}

fn file_len(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Delete any spooled telemetry under `config_dir`. Safe to call at any time,
/// including when telemetry was never configured. Call this whenever the user
/// opts out, alongside [`reset_install_id`].
pub fn purge_spool(config_dir: &Path) {
    Spool::new(config_dir).purge();
}

// ---------------------------------------------------------------------------
// Retry policy
// ---------------------------------------------------------------------------

/// What to do with a batch after one export attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportOutcome {
    /// The collector took it. Forget it.
    Delivered,
    /// Transient: offline, timed out, rate limited, or the collector is
    /// unwell. Keep the batch and try again later.
    Retry,
    /// Permanent: the payload or the credentials are wrong. Retrying cannot
    /// fix that, and a spool that retries forever is a leak of both disk and
    /// collector capacity — so throw the batch away.
    Discard,
}

/// Classify an HTTP status. 429 and 5xx are the collector saying "later";
/// every other non-2xx is the collector saying "never" (bad request, bad
/// auth, wrong path, unfollowed redirect).
pub fn classify_status(status: u16) -> ExportOutcome {
    match status {
        200..=299 => ExportOutcome::Delivered,
        429 => ExportOutcome::Retry,
        500..=599 => ExportOutcome::Retry,
        _ => ExportOutcome::Discard,
    }
}

/// Deterministic backoff window for attempt `n` (0-based): doubling from
/// [`RETRY_BASE_SECS`], clamped to [`RETRY_MAX_SECS`].
pub fn backoff_base_secs(attempt: u32) -> u64 {
    let shift = attempt.min(32);
    RETRY_BASE_SECS
        .checked_shl(shift)
        .unwrap_or(RETRY_MAX_SECS)
        .min(RETRY_MAX_SECS)
}

/// "Equal jitter": half the window fixed, half random, giving a delay
/// uniformly in `[base/2, base]`.
///
/// The jitter is the point. Without it, every client in the org that was
/// online during an outage retries on exactly the same schedule afterwards
/// and stampedes the collector the moment it recovers. The lower half is kept
/// fixed so the delay can never collapse to zero (or, with a signed
/// implementation, go negative) and spin the worker.
pub fn jittered_delay(base_secs: u64) -> Duration {
    let half = base_secs / 2;
    let jitter = if half == 0 {
        0
    } else {
        random_u64() % (half + 1)
    };
    Duration::from_secs((half + jitter).max(1))
}

/// Retry schedule for the export worker.
#[derive(Debug, Default)]
pub struct Backoff {
    attempt: u32,
}

impl Backoff {
    pub fn new() -> Self {
        Backoff::default()
    }

    /// Next delay, then widen the window for the attempt after this one.
    pub fn next_delay(&mut self) -> Duration {
        let delay = jittered_delay(backoff_base_secs(self.attempt));
        self.attempt = self.attempt.saturating_add(1);
        delay
    }

    /// Back to the base window after any progress.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

// ---------------------------------------------------------------------------
// Emission (public surface)
// ---------------------------------------------------------------------------

/// Runtime handle: the opt-out gate plus the bounded queue into the exporter.
pub struct Telemetry {
    /// Shared with the export worker so opting out also discards events that
    /// were already queued, instead of only stopping new ones.
    enabled: Arc<AtomicBool>,
    tx: tokio::sync::mpsc::Sender<Event>,
}

impl Telemetry {
    pub fn new(enabled: bool, tx: tokio::sync::mpsc::Sender<Event>) -> Self {
        Telemetry {
            enabled: Arc::new(AtomicBool::new(enabled)),
            tx,
        }
    }

    /// Handle on the opt-out gate for the export worker.
    fn gate(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.enabled)
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }

    /// Fire-and-forget. Never blocks: `try_send` drops the event when the
    /// queue is full, and a disabled toggle short-circuits before any work.
    pub fn emit(&self, name: &'static str, attrs: &[(&'static str, AttrValue)]) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        let _ = self.tx.try_send(Event::new(name, attrs));
    }
}

static INSTANCE: OnceLock<Telemetry> = OnceLock::new();

/// Record one telemetry event. Safe to call from anywhere, at any time:
/// before [`init`], with telemetry unconfigured, or with the user opted out,
/// it is a no-op. Attribute keys must be on [`ALLOWED_ATTR_KEYS`].
pub fn event(name: &'static str, attrs: &[(&'static str, AttrValue)]) {
    if let Some(t) = INSTANCE.get() {
        t.emit(name, attrs);
    }
}

// ---------------------------------------------------------------------------
// Startup timing
// ---------------------------------------------------------------------------

static PROCESS_START: OnceLock<std::time::Instant> = OnceLock::new();
static STARTUP_REPORTED: AtomicBool = AtomicBool::new(false);

/// Stamp the start of the process. Call this as the first thing in `run()`;
/// calling it later just makes the measurement flatter, never wrong in a way
/// that can hurt.
pub fn mark_process_start() {
    let _ = PROCESS_START.set(std::time::Instant::now());
}

/// Emit `app_startup_completed` the first time the UI asks the backend for
/// anything — that first command is the earliest moment the webview is
/// actually running, so it is our "UI is ready" signal.
///
/// Subsequent calls are a no-op, so this is safe to hang off a command the
/// frontend calls more than once. A bucket is sent, never the raw duration.
pub fn mark_ui_ready() {
    if STARTUP_REPORTED.swap(true, Ordering::Relaxed) {
        return;
    }
    let Some(start) = PROCESS_START.get() else {
        return;
    };
    let ms = start.elapsed().as_millis();
    event(
        "app_startup_completed",
        &[("app_startup_duration_bucket", latency_bucket_ms(ms).into())],
    );
}

/// Flip the opt-out gate at runtime (Settings toggle). Takes effect
/// immediately and in both directions: a disabled gate stops new events
/// before they reach the queue, and the export worker discards any batch
/// still waiting. No-op when telemetry was never configured.
pub fn set_enabled(on: bool) {
    if let Some(t) = INSTANCE.get() {
        t.set_enabled(on);
    }
}

/// Start telemetry if — and only if — an exporter credential resolves (see
/// [`exporter_config_from_parts`]). Otherwise this returns without doing
/// anything: no queue, no background task, no spool, no network, no errors.
/// Must be called after the tauri async runtime exists (it spawns the export
/// worker there).
pub fn init(config_dir: &Path, app_version: &str, enabled: bool) {
    if !enabled {
        // The user opted out in an earlier session. Anything still spooled
        // from before that decision must not survive to be sent now.
        purge_spool(config_dir);
    }
    let Some(config) = exporter_config_from_env() else {
        tracing::debug!("telemetry: no exporter credential configured; staying inert");
        return;
    };
    // Resolve the id once up front: if it cannot be persisted, stay inert
    // rather than send an id that changes on every launch.
    if load_or_create_install_id(config_dir).is_none() {
        return;
    }
    let (tx, rx) = tokio::sync::mpsc::channel(QUEUE_CAPACITY);
    let telemetry = Telemetry::new(enabled, tx);
    // Read the environment description once. It cannot change while the
    // process runs, and `os_info::get()` does real work (a plist read, an
    // `/etc/os-release` read, or a syscall) that should not repeat per batch.
    let ctx = ExportContext {
        config_dir: config_dir.to_path_buf(),
        app_version: app_version.to_string(),
        os: std::env::consts::OS.to_string(),
        os_version: os_version(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_cores: cpu_cores(),
        channel: app_channel().to_string(),
        session_id: session_id().to_string(),
        gate: telemetry.gate(),
    };
    if INSTANCE.set(telemetry).is_err() {
        return; // double init — keep the first
    }
    tauri::async_runtime::spawn(export_worker(config, ctx, rx));
}

// ---------------------------------------------------------------------------
// Export worker
// ---------------------------------------------------------------------------

/// Everything the export worker needs to describe this install, plus the
/// opt-out gate. The install id is deliberately *not* cached here — see
/// [`ExportContext::resource`].
struct ExportContext {
    config_dir: PathBuf,
    app_version: String,
    os: String,
    os_version: String,
    arch: String,
    cpu_cores: i64,
    channel: String,
    session_id: String,
    gate: Arc<AtomicBool>,
}

impl ExportContext {
    /// Resource attributes for one batch. The install id is re-read from disk
    /// per batch rather than cached, so opting out (which deletes the file)
    /// and later opting back in yields a fresh id that cannot be linked to the
    /// old one — without needing an app restart. One tiny read at most every
    /// [`FLUSH_INTERVAL_SECS`], on a background task, off the recording path.
    fn resource(&self) -> Option<Resource> {
        Some(Resource {
            install_id: load_or_create_install_id(&self.config_dir)?,
            session_id: self.session_id.clone(),
            app_version: self.app_version.clone(),
            os: self.os.clone(),
            os_version: self.os_version.clone(),
            arch: self.arch.clone(),
            cpu_cores: self.cpu_cores,
            channel: self.channel.clone(),
        })
    }
}

/// What one pass over the spool achieved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrainResult {
    /// Nothing on disk (or telemetry is off) — no reason to back off.
    Empty,
    /// At least one batch left the disk.
    Progress,
    /// The collector is still refusing everything. Back off further.
    Stalled,
}

/// Owns everything the background worker needs to get a batch delivered:
/// the HTTP client, the destination, the install description, the disk spool
/// and the retry schedule.
///
/// All of its I/O — network and filesystem — runs on the export task. The
/// recording pipeline only ever touches [`Telemetry::emit`]'s `try_send`.
struct Exporter {
    client: reqwest::Client,
    config: ExporterConfig,
    ctx: ExportContext,
    spool: Spool,
    backoff: Backoff,
    /// Batches lost to the spool bounds since the last successful report.
    /// Carried into the next batch as a `telemetry_spool_dropped` event so a
    /// gap in the data is visible rather than silently misleading.
    dropped_batches: u64,
}

impl Exporter {
    /// POST one already-serialized batch and classify the result. Never
    /// panics; a transport error is simply "try again later".
    async fn post(&self, body: Vec<u8>) -> ExportOutcome {
        let mut req = self
            .client
            .post(&self.config.endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body);
        for (k, v) in &self.config.headers {
            req = req.header(k, v);
        }
        match req.send().await {
            Ok(resp) => {
                let outcome = classify_status(resp.status().as_u16());
                if outcome != ExportOutcome::Delivered {
                    tracing::debug!(
                        "telemetry: collector returned {} -> {outcome:?}",
                        resp.status()
                    );
                }
                outcome
            }
            // Connection refused, DNS failure, TLS failure, timeout: exactly
            // the "the laptop went offline" case the spool exists for.
            Err(e) => {
                tracing::debug!("telemetry: export failed: {e}");
                ExportOutcome::Retry
            }
        }
    }

    /// Whether telemetry may touch the network **or** the disk right now.
    ///
    /// A disabled gate purges the spool as a side effect. This is the single
    /// most important rule in the retry design: if opting out only stopped
    /// new sends, everything already spooled would still go out the next time
    /// the collector was reachable, and "off" would have leaked.
    fn allowed(&self) -> bool {
        if self.ctx.gate.load(Ordering::Relaxed) {
            return true;
        }
        self.spool.purge();
        false
    }

    /// Send whatever has accumulated in memory. On a retryable failure the
    /// batch goes to the spool instead of being lost.
    async fn flush(&mut self, pending: &mut Vec<Event>) {
        if pending.is_empty() {
            return;
        }
        // The user may have opted out while these events sat in the queue.
        // "Off" means nothing is sent, including whatever was already waiting.
        if !self.allowed() {
            pending.clear();
            return;
        }
        let mut events: Vec<Event> = std::mem::take(pending);
        if self.dropped_batches > 0 {
            events.push(Event::new(
                "telemetry_spool_dropped",
                &[("dropped", (self.dropped_batches as i64).into())],
            ));
            // Cleared now, not on success: if this batch fails it is spooled,
            // and the count travels to disk inside it.
            self.dropped_batches = 0;
        }
        let Some(resource) = self.ctx.resource() else {
            return;
        };
        let payload = build_export_payload(&resource, &events);
        let Ok(body) = serde_json::to_vec(&payload) else {
            return;
        };
        match self.post(body.clone()).await {
            ExportOutcome::Delivered => self.backoff.reset(),
            // The endpoint or the credentials are wrong. Spooling would build
            // a queue that can never drain, so drop it and stop escalating.
            ExportOutcome::Discard => self.backoff.reset(),
            ExportOutcome::Retry => {
                self.dropped_batches += self.spool.push(&body);
                let (count, bytes) = self.spool.stats();
                tracing::debug!("telemetry: batch spooled ({count} batches, {bytes} bytes)");
            }
        }
    }

    /// Try to deliver spooled batches, oldest first. Stops at the first batch
    /// the collector still will not take, so one pass costs at most one failed
    /// request once the network is down.
    async fn drain_spool(&mut self) -> DrainResult {
        if !self.allowed() {
            return DrainResult::Empty;
        }
        let batches = self.spool.batches();
        if batches.is_empty() {
            return DrainResult::Empty;
        }
        let mut progressed = false;
        for path in batches {
            let body = match std::fs::read(&path) {
                Ok(body) if !body.is_empty() => body,
                // Unreadable or empty: a truncated write, a permissions
                // change, a file someone deleted underneath us. Drop it —
                // one bad file must never wedge the whole spool.
                _ => {
                    tracing::debug!("telemetry: unreadable spool file, discarding");
                    self.spool.remove(&path);
                    progressed = true;
                    continue;
                }
            };
            if serde_json::from_slice::<Value>(&body).is_err() {
                tracing::debug!("telemetry: corrupt spool file, discarding");
                self.spool.remove(&path);
                progressed = true;
                continue;
            }
            match self.post(body).await {
                ExportOutcome::Delivered | ExportOutcome::Discard => {
                    self.spool.remove(&path);
                    progressed = true;
                }
                // Still offline / still rate limited. Leave the rest on disk
                // and wait for the next backoff window.
                ExportOutcome::Retry => {
                    return if progressed {
                        DrainResult::Progress
                    } else {
                        DrainResult::Stalled
                    };
                }
            }
        }
        if progressed {
            DrainResult::Progress
        } else {
            DrainResult::Stalled
        }
    }
}

async fn export_worker(
    config: ExporterConfig,
    ctx: ExportContext,
    mut rx: tokio::sync::mpsc::Receiver<Event>,
) {
    // Own client: short timeout, isolated from the app's 120 s summary client.
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(EXPORT_TIMEOUT_SECS))
        .build()
    else {
        return;
    };
    let spool = Spool::new(&ctx.config_dir);
    let mut ex = Exporter {
        client,
        config,
        ctx,
        spool,
        backoff: Backoff::new(),
        dropped_batches: 0,
    };

    // Anything left by a previous run goes out first: events have to survive
    // an app restart, not just a momentary network blip.
    let _ = ex.drain_spool().await;

    let mut pending: Vec<Event> = Vec::new();
    let mut tick = tokio::time::interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // Absolute deadline, so recreating the sleep future on every loop
    // iteration re-arms the *same* instant instead of restarting the timer.
    let mut retry_at = tokio::time::Instant::now() + ex.backoff.next_delay();
    loop {
        tokio::select! {
            ev = rx.recv() => match ev {
                Some(ev) => {
                    pending.push(ev);
                    if pending.len() >= MAX_BATCH {
                        ex.flush(&mut pending).await;
                    }
                }
                None => {
                    ex.flush(&mut pending).await;
                    return;
                }
            },
            _ = tick.tick() => {
                ex.flush(&mut pending).await;
            }
            _ = tokio::time::sleep_until(retry_at) => {
                // Only a genuinely stalled spool widens the window; an empty
                // spool resets it so the first retry after a fresh failure is
                // ~30 s rather than however far the last outage escalated.
                match ex.drain_spool().await {
                    DrainResult::Empty | DrainResult::Progress => ex.backoff.reset(),
                    DrainResult::Stalled => {}
                }
                retry_at = tokio::time::Instant::now() + ex.backoff.next_delay();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// OTLP/HTTP JSON payload
// ---------------------------------------------------------------------------

fn otlp_value(v: &AttrValue) -> Value {
    match v {
        AttrValue::Str(s) => json!({ "stringValue": s }),
        AttrValue::Int(i) => json!({ "intValue": i.to_string() }),
        AttrValue::Bool(b) => json!({ "boolValue": b }),
    }
}

fn otlp_attr(key: &str, v: &AttrValue) -> Value {
    json!({ "key": key, "value": otlp_value(v) })
}

fn otlp_str_attr(key: &str, s: &str) -> Value {
    otlp_attr(key, &AttrValue::Str(s.to_string()))
}

fn otlp_int_attr(key: &str, i: i64) -> Value {
    otlp_attr(key, &AttrValue::Int(i))
}

/// Build the OTLP `ExportLogsServiceRequest` JSON for one batch. Attribute
/// names follow OpenTelemetry semantic conventions where one exists
/// (`service.name`, `service.version`, `os.type`, `os.version`, `device.arch`,
/// `event.name`, `error.type`); the install id uses an app-scoped
/// `desksec.install.id`.
///
/// The resource attribute keys produced here are pinned by
/// [`ALLOWED_RESOURCE_ATTR_KEYS`] and by a test that compares the two, so a
/// new one cannot appear without a deliberate edit in both places.
pub fn build_export_payload(resource: &Resource, events: &[Event]) -> Value {
    let log_records: Vec<Value> = events
        .iter()
        .map(|ev| {
            let mut attributes = vec![
                otlp_str_attr("event.name", ev.name),
                // Identity lives on the record, not the resource. See the
                // cardinality note on ALLOWED_RESOURCE_ATTR_KEYS: these two
                // values are unique per launch and per install, so promoting
                // them to Loki labels would be a cardinality blow-up.
                otlp_str_attr("session.id", &resource.session_id),
                otlp_str_attr("desksec.install.id", &resource.install_id),
            ];
            // Dev-build drift guard, mirroring the resource one below: the
            // identity keys emitted here must stay exactly IDENTITY_ATTR_KEYS.
            debug_assert!(
                attributes[1..]
                    .iter()
                    .map(|a| a["key"].as_str().unwrap_or_default())
                    .eq(IDENTITY_ATTR_KEYS.iter().copied()),
                "record identity attributes drifted from IDENTITY_ATTR_KEYS"
            );
            attributes.extend(
                ev.attrs
                    .iter()
                    .filter(|(k, _)| ALLOWED_ATTR_KEYS.contains(k))
                    .map(|(k, v)| otlp_attr(k, v)),
            );
            json!({
                "timeUnixNano": ev.time_unix_nano.to_string(),
                "severityNumber": 9,
                "severityText": "INFO",
                "body": { "stringValue": ev.name },
                "attributes": attributes,
            })
        })
        .collect();

    // ---- resource attributes: LOW CARDINALITY ONLY --------------------------
    //
    // These are the attributes an OTLP -> Loki pipeline is most likely to
    // promote into index labels, so every value here must come from a small,
    // finite set. A unique-per-user or unique-per-launch value would be a
    // cardinality blow-up for the whole shared stack.
    //
    // Unique identifiers (install id, session id) therefore go on the LOG
    // RECORD instead — see the record builder above. Do not move them here,
    // and do not add anything whose value set is unbounded.
    let resource_attributes = vec![
        otlp_str_attr("service.name", SERVICE_NAME),
        otlp_str_attr("service.namespace", SERVICE_NAMESPACE),
        otlp_str_attr("service.version", &resource.app_version),
        otlp_str_attr("os.type", &resource.os),
        otlp_str_attr("os.version", &resource.os_version),
        otlp_str_attr("device.arch", &resource.arch),
        otlp_int_attr("device.cpu.cores", resource.cpu_cores),
        otlp_str_attr("app.channel", &resource.channel),
    ];
    // Dev-build drift guard: the list above and ALLOWED_RESOURCE_ATTR_KEYS
    // must stay in lockstep, so a resource attribute can never be added in
    // one place only. A release build pays nothing for this.
    debug_assert!(
        resource_attributes
            .iter()
            .map(|a| a["key"].as_str().unwrap_or_default())
            .eq(ALLOWED_RESOURCE_ATTR_KEYS.iter().copied()),
        "resource attributes drifted from ALLOWED_RESOURCE_ATTR_KEYS"
    );

    json!({
        "resourceLogs": [{
            "resource": { "attributes": resource_attributes },
            "scopeLogs": [{
                "scope": {
                    "name": "desksec.telemetry",
                    "version": SCHEMA_VERSION.to_string(),
                },
                "logRecords": log_records,
            }],
        }],
    })
}

// ---------------------------------------------------------------------------
// Bucket helpers — the only way durations and lengths may leave the device.
// ---------------------------------------------------------------------------

/// Meeting duration → coarse bucket. Raw durations are never sent.
pub fn duration_bucket_secs(secs: u64) -> &'static str {
    match secs {
        0..=59 => "<1m",
        60..=299 => "1-5m",
        300..=899 => "5-15m",
        900..=1799 => "15-30m",
        1800..=3599 => "30-60m",
        3600..=7199 => "1-2h",
        _ => ">2h",
    }
}

/// Wall-clock latency in milliseconds → coarse bucket. Used by every
/// performance attribute (`app_startup_duration_bucket`,
/// `summarize_duration_bucket`, `transcription_latency_bucket`,
/// `download_duration_bucket`) so they stay comparable to each other.
///
/// A bucket, never the raw value: raw millisecond timings across many events
/// are a surprisingly good fingerprint, and coarse ranges answer every
/// question we actually have ("is on-device slow on this hardware?").
pub fn latency_bucket_ms(ms: u128) -> &'static str {
    match ms {
        0..=499 => "<0.5s",
        500..=999 => "0.5-1s",
        1_000..=2_999 => "1-3s",
        3_000..=4_999 => "3-5s",
        5_000..=9_999 => "5-10s",
        10_000..=29_999 => "10-30s",
        30_000..=59_999 => "30-60s",
        60_000..=179_999 => "1-3m",
        180_000..=599_999 => "3-10m",
        _ => ">10m",
    }
}

/// Transcript length in characters → coarse bucket. This is the cost proxy
/// for summary-model input size; the text itself is never sent.
pub fn transcript_length_bucket(chars: usize) -> &'static str {
    match chars {
        0 => "0",
        1..=999 => "1-1k",
        1000..=4999 => "1k-5k",
        5000..=19_999 => "5k-20k",
        20_000..=49_999 => "20k-50k",
        50_000..=99_999 => "50k-100k",
        _ => ">100k",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_resource() -> Resource {
        Resource {
            install_id: "11111111-2222-4333-8444-555555555555".into(),
            session_id: "99999999-8888-4777-8666-555555555555".into(),
            app_version: "1.2.3".into(),
            os: "linux".into(),
            os_version: "22.04".into(),
            arch: "aarch64".into(),
            cpu_cores: 10,
            channel: "release".into(),
        }
    }

    // -- privacy regression guards ------------------------------------------

    /// If one of these ever appears inside an allowlisted attribute key, a
    /// content or identity field is about to leak. This test failing is a
    /// feature: it forces a privacy review before the key can ship.
    const FORBIDDEN_KEY_FRAGMENTS: &[&str] = &[
        "title",
        "text",
        "content",
        "body",
        "snippet",
        "summary_",
        "instruction",
        "name",
        "email",
        "user",
        "host",
        "device",
        "path",
        "file",
        "url",
        "token",
        "speaker",
        "participant",
        "address",
        "serial",
        "query",
        "audio_",
    ];

    #[test]
    fn allowlist_contains_no_content_shaped_keys() {
        for key in ALLOWED_ATTR_KEYS {
            for frag in FORBIDDEN_KEY_FRAGMENTS {
                assert!(
                    !key.contains(frag),
                    "allowlisted telemetry key {key:?} contains forbidden fragment {frag:?} — \
                     telemetry must never carry content or identity"
                );
            }
        }
    }

    #[test]
    fn non_allowlisted_attributes_never_reach_the_payload() {
        // Simulates a future contributor "helpfully" attaching the meeting
        // title to an event. It must vanish before export.
        let ev = Event::new(
            "recording_completed",
            &[
                ("duration_bucket", "5-15m".into()),
                ("meeting_title", "Q3 board meeting — LAYOFFS".into()),
                ("transcript", "we decided to...".into()),
            ],
        );
        assert_eq!(ev.attrs.len(), 1);
        assert_eq!(ev.attrs[0].0, "duration_bucket");

        let payload = build_export_payload(&test_resource(), &[ev]);
        let serialized = payload.to_string();
        assert!(!serialized.contains("meeting_title"));
        assert!(!serialized.contains("LAYOFFS"));
        assert!(!serialized.contains("we decided to"));
        assert!(serialized.contains("duration_bucket"));
    }

    #[test]
    fn payload_shape_is_otlp_logs_json() {
        let resource = test_resource();
        let ev = Event::new(
            "summary_generated",
            &[
                ("transcript_length_bucket", "5k-20k".into()),
                ("engine", "deepgram".into()),
                ("schema_version", SCHEMA_VERSION.into()),
            ],
        );
        let payload = build_export_payload(&resource, &[ev]);

        let record = &payload["resourceLogs"][0]["scopeLogs"][0]["logRecords"][0];
        assert_eq!(record["body"]["stringValue"], "summary_generated");
        assert_eq!(
            record["attributes"][0]["key"], "event.name",
            "every record must carry event.name for Loki-side filtering"
        );
        let res_attrs = payload["resourceLogs"][0]["resource"]["attributes"]
            .as_array()
            .unwrap();
        let keys: Vec<&str> = res_attrs
            .iter()
            .map(|a| a["key"].as_str().unwrap())
            .collect();
        assert_eq!(
            keys, ALLOWED_RESOURCE_ATTR_KEYS,
            "the resource attribute set is pinned: adding one must be a \
             deliberate edit in both the payload builder and the allowlist"
        );
        // Identity is on the record, not the resource.
        let record_keys: Vec<&str> = record["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["key"].as_str().unwrap())
            .collect();
        for key in IDENTITY_ATTR_KEYS {
            assert!(record_keys.contains(key), "every record must carry {key}");
        }
        // ints are encoded as strings per OTLP JSON mapping
        let attrs = record["attributes"].as_array().unwrap();
        let sv = attrs
            .iter()
            .find(|a| a["key"] == "schema_version")
            .expect("schema_version attr");
        assert_eq!(sv["value"]["intValue"], SCHEMA_VERSION.to_string());
    }

    // -- opt-out actually suppresses emission --------------------------------

    #[test]
    fn opt_out_suppresses_emission_and_opt_in_restores_it() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let t = Telemetry::new(false, tx);

        t.emit("app_started", &[("engine", "whisper".into())]);
        assert!(
            rx.try_recv().is_err(),
            "disabled telemetry must emit nothing"
        );

        t.set_enabled(true);
        t.emit("app_started", &[("engine", "whisper".into())]);
        let got = rx.try_recv().expect("enabled telemetry must emit");
        assert_eq!(got.name, "app_started");

        t.set_enabled(false);
        t.emit("app_started", &[]);
        assert!(rx.try_recv().is_err(), "re-disabling must stop emission");
    }

    #[test]
    fn opting_out_also_stops_events_already_queued() {
        // Emission-side suppression alone would still let whatever was already
        // in the queue go out after the user said stop. The export worker
        // therefore reads the same gate and drops the batch.
        let (tx, _rx) = tokio::sync::mpsc::channel(8);
        let t = Telemetry::new(true, tx);
        let gate = t.gate();
        assert!(gate.load(Ordering::Relaxed));

        t.set_enabled(false);
        assert!(
            !gate.load(Ordering::Relaxed),
            "the exporter must observe the opt-out, not just the emit path"
        );

        t.set_enabled(true);
        assert!(gate.load(Ordering::Relaxed));
    }

    #[test]
    fn full_queue_drops_events_instead_of_blocking() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        let t = Telemetry::new(true, tx);
        for _ in 0..10 {
            t.emit("app_started", &[]); // must never block or panic
        }
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
        assert!(
            rx.try_recv().is_err(),
            "overflow beyond capacity is dropped"
        );
    }

    #[test]
    fn event_without_init_is_a_silent_noop() {
        // The global INSTANCE is never set in unit tests; both entry points
        // must tolerate that forever (telemetry unconfigured = no-op).
        event("app_started", &[("engine", "whisper".into())]);
        set_enabled(false);
        set_enabled(true);
    }

    // -- configuration parsing ----------------------------------------------

    #[test]
    fn missing_or_invalid_endpoint_means_inert() {
        assert_eq!(parse_exporter_config(None, None), None);
        assert_eq!(parse_exporter_config(Some(""), None), None);
        assert_eq!(parse_exporter_config(Some("   "), None), None);
        assert_eq!(
            parse_exporter_config(Some("your-collector-here"), None),
            None,
            "placeholder values must not configure an exporter"
        );
        assert_eq!(
            parse_exporter_config(Some("ftp://otlp.grafana.internal"), None),
            None
        );
        assert_eq!(
            parse_exporter_config(Some("otlp.grafana.internal"), None),
            None
        );
    }

    #[test]
    fn base_endpoint_gets_the_logs_path_appended() {
        let cfg = parse_exporter_config(Some("https://otlp.grafana.internal"), None).unwrap();
        assert_eq!(cfg.endpoint, "https://otlp.grafana.internal/v1/logs");
        let cfg = parse_exporter_config(Some("https://otlp.grafana.internal/"), None).unwrap();
        assert_eq!(cfg.endpoint, "https://otlp.grafana.internal/v1/logs");
        let cfg =
            parse_exporter_config(Some("https://otlp.grafana.internal/v1/logs"), None).unwrap();
        assert_eq!(cfg.endpoint, "https://otlp.grafana.internal/v1/logs");
    }

    #[test]
    fn headers_parse_otel_env_syntax_and_tolerate_garbage() {
        let cfg = parse_exporter_config(
            Some("https://otlp.grafana.internal"),
            Some("Authorization=Basic abc123, X-Scope-OrgID=desksec ,malformed,=,a="),
        )
        .unwrap();
        assert_eq!(
            cfg.headers,
            vec![
                ("Authorization".to_string(), "Basic abc123".to_string()),
                ("X-Scope-OrgID".to_string(), "desksec".to_string()),
            ]
        );
        let cfg = parse_exporter_config(Some("https://otlp.grafana.internal"), None).unwrap();
        assert!(cfg.headers.is_empty());
    }

    // -- install id ----------------------------------------------------------

    #[test]
    fn install_id_is_a_uuid_stable_across_loads_and_resettable() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_or_create_install_id(dir.path()).expect("id created");
        assert!(looks_like_uuid(&first));

        let second = load_or_create_install_id(dir.path()).expect("id reread");
        assert_eq!(first, second, "install id must be stable across runs");

        reset_install_id(dir.path());
        let third = load_or_create_install_id(dir.path()).expect("id recreated");
        assert!(looks_like_uuid(&third));
        assert_ne!(first, third, "reset must produce a fresh, unlinkable id");
    }

    #[test]
    fn an_unusable_config_dir_stays_inert_instead_of_panicking() {
        // A config dir that cannot be created (here: the path is an existing
        // file) must degrade to "no telemetry", never to a panic and never to
        // an id that changes on every launch.
        let dir = tempfile::tempdir().unwrap();
        let blocked = dir.path().join("this-is-a-file");
        std::fs::write(&blocked, b"not a directory").unwrap();
        assert_eq!(load_or_create_install_id(&blocked), None);
    }

    #[test]
    fn corrupt_install_id_file_is_replaced_not_propagated() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(INSTALL_ID_FILE), "Alice's MacBook Pro").unwrap();
        let id = load_or_create_install_id(dir.path()).expect("id regenerated");
        assert!(looks_like_uuid(&id), "corrupt value must be replaced: {id}");
        assert!(!id.contains("Alice"));
    }

    // -- buckets -------------------------------------------------------------

    #[test]
    fn duration_buckets() {
        assert_eq!(duration_bucket_secs(0), "<1m");
        assert_eq!(duration_bucket_secs(59), "<1m");
        assert_eq!(duration_bucket_secs(60), "1-5m");
        assert_eq!(duration_bucket_secs(300), "5-15m");
        assert_eq!(duration_bucket_secs(1799), "15-30m");
        assert_eq!(duration_bucket_secs(3599), "30-60m");
        assert_eq!(duration_bucket_secs(3600), "1-2h");
        assert_eq!(duration_bucket_secs(90_000), ">2h");
    }

    #[test]
    fn transcript_length_buckets() {
        assert_eq!(transcript_length_bucket(0), "0");
        assert_eq!(transcript_length_bucket(999), "1-1k");
        assert_eq!(transcript_length_bucket(1000), "1k-5k");
        assert_eq!(transcript_length_bucket(19_999), "5k-20k");
        assert_eq!(transcript_length_bucket(50_000), "50k-100k");
        assert_eq!(transcript_length_bucket(250_000), ">100k");
    }

    #[test]
    fn latency_buckets() {
        assert_eq!(latency_bucket_ms(0), "<0.5s");
        assert_eq!(latency_bucket_ms(499), "<0.5s");
        assert_eq!(latency_bucket_ms(500), "0.5-1s");
        assert_eq!(latency_bucket_ms(1_000), "1-3s");
        assert_eq!(latency_bucket_ms(4_999), "3-5s");
        assert_eq!(latency_bucket_ms(9_999), "5-10s");
        assert_eq!(latency_bucket_ms(30_000), "30-60s");
        assert_eq!(latency_bucket_ms(120_000), "1-3m");
        assert_eq!(latency_bucket_ms(600_000), ">10m");
    }

    // -- resource metadata ---------------------------------------------------

    #[test]
    fn resource_attribute_keys_are_not_content_shaped() {
        // Two fragments legitimately appear, and only in these keys:
        // `device.arch` / `device.cpu.cores` are CPU facts, never the *name*
        // of an audio device, and `service.name` is the constant "desksec".
        // Every other forbidden fragment still applies.
        const JUSTIFIED: &[(&str, &str)] = &[
            ("device.arch", "device"),
            ("device.cpu.cores", "device"),
            ("service.name", "name"),
            // `service.namespace` is the constant "amalitech", not anyone's name.
            ("service.namespace", "name"),
        ];
        let keys = ALLOWED_RESOURCE_ATTR_KEYS
            .iter()
            .chain(IDENTITY_ATTR_KEYS.iter());
        for key in keys {
            for frag in FORBIDDEN_KEY_FRAGMENTS {
                if JUSTIFIED.contains(&(*key, *frag)) {
                    continue;
                }
                assert!(
                    !key.contains(frag),
                    "resource attribute {key:?} contains forbidden fragment {frag:?} — \
                     resource metadata must describe the machine class, never the machine"
                );
            }
        }
    }

    #[test]
    fn new_resource_attributes_appear_in_the_payload_with_the_expected_shape() {
        let payload = build_export_payload(&test_resource(), &[Event::new("app_started", &[])]);
        let attrs = payload["resourceLogs"][0]["resource"]["attributes"]
            .as_array()
            .cloned()
            .unwrap();
        let value = |key: &str| {
            attrs
                .iter()
                .find(|a| a["key"] == key)
                .unwrap_or_else(|| panic!("missing resource attribute {key}"))["value"]
                .clone()
        };
        assert_eq!(value("service.name")["stringValue"], SERVICE_NAME);
        assert_eq!(value("service.namespace")["stringValue"], SERVICE_NAMESPACE);
        assert_eq!(value("service.version")["stringValue"], "1.2.3");
        assert_eq!(value("os.type")["stringValue"], "linux");
        assert_eq!(value("os.version")["stringValue"], "22.04");
        assert_eq!(value("device.arch")["stringValue"], "aarch64");
        // ints use the OTLP JSON string encoding
        assert_eq!(value("device.cpu.cores")["intValue"], "10");
        assert_eq!(value("app.channel")["stringValue"], "release");
    }

    /// The cardinality guard. Resource attributes are the ones an OTLP → Loki
    /// pipeline is most likely to promote to index labels, so a unique value
    /// there is a cardinality blow-up for the whole shared stack. The install
    /// id and the session id must live on the log record instead — this test
    /// exists so a future refactor cannot quietly move them back.
    #[test]
    fn resource_attributes_carry_no_high_cardinality_identifiers() {
        let resource = test_resource();
        let payload = build_export_payload(&resource, &[Event::new("app_started", &[])]);
        let block = &payload["resourceLogs"][0]["resource"];
        let keys: Vec<&str> = block["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["key"].as_str().unwrap())
            .collect();

        for banned in IDENTITY_ATTR_KEYS {
            assert!(
                !keys.contains(banned),
                "{banned} is unique per install/launch and must NEVER be a \
                 resource attribute — it may become a Loki index label"
            );
        }
        for key in &keys {
            assert!(
                !key.contains("install") && !key.contains("session"),
                "resource attribute {key:?} looks like a per-install or \
                 per-launch identifier"
            );
        }
        // Belt and braces: the values must not be in the resource block either,
        // under any key.
        let serialized = block.to_string();
        assert!(!serialized.contains(&resource.install_id));
        assert!(!serialized.contains(&resource.session_id));

        // ...and they must still be present, on the record.
        let record = &payload["resourceLogs"][0]["scopeLogs"][0]["logRecords"][0];
        let record_json = record.to_string();
        assert!(record_json.contains(&resource.install_id));
        assert!(record_json.contains(&resource.session_id));
    }

    // -- exporter configuration & auth ---------------------------------------

    // Obviously fake. No real credential appears anywhere in this repository.
    const FAKE_TOKEN: &str = "test-token";

    #[test]
    fn the_default_endpoint_is_used_when_the_environment_is_silent() {
        let config =
            exporter_config_from_parts(None, None, GRAFANA_INSTANCE_ID, Some(FAKE_TOKEN)).unwrap();
        assert_eq!(
            config.endpoint,
            format!("{DEFAULT_OTLP_ENDPOINT}/v1/logs"),
            "the hardcoded base must get the logs path appended"
        );
    }

    #[test]
    fn basic_auth_is_assembled_from_the_instance_id_and_the_token() {
        use base64::Engine as _;
        let config = exporter_config_from_parts(None, None, "1549080", Some(FAKE_TOKEN)).unwrap();
        assert_eq!(config.headers.len(), 1);
        let (name, value) = &config.headers[0];
        assert_eq!(name, "Authorization");
        let encoded = value.strip_prefix("Basic ").expect("Basic scheme");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(
            String::from_utf8(decoded).unwrap(),
            format!("1549080:{FAKE_TOKEN}"),
            "Grafana Cloud expects the instance id as the Basic username"
        );
    }

    #[test]
    fn no_token_anywhere_means_completely_inert() {
        assert_eq!(
            exporter_config_from_parts(None, None, GRAFANA_INSTANCE_ID, None),
            None,
            "an exporter config without a credential must keep the app working \
             with telemetry simply switched off"
        );
        // An endpoint is configured, but there is still no credential: we must
        // not POST unauthenticated and spool a queue of doomed batches.
        assert_eq!(
            exporter_config_from_parts(
                Some("https://otlp.grafana.internal"),
                None,
                GRAFANA_INSTANCE_ID,
                None,
            ),
            None
        );
        // A placeholder is not a token.
        assert_eq!(
            exporter_config_from_parts(
                None,
                None,
                GRAFANA_INSTANCE_ID,
                usable_value("your-token-here").as_deref()
            ),
            None
        );
    }

    #[test]
    fn explicit_headers_win_over_the_constructed_grafana_auth() {
        // How a developer points a build at a staging stack, or at a
        // self-hosted collector that has no Grafana token at all.
        let config = exporter_config_from_parts(
            Some("https://otlp.staging.internal"),
            Some("Authorization=Bearer staging,X-Scope-OrgID=desksec"),
            GRAFANA_INSTANCE_ID,
            Some(FAKE_TOKEN),
        )
        .unwrap();
        assert_eq!(config.endpoint, "https://otlp.staging.internal/v1/logs");
        assert_eq!(
            config.headers,
            vec![
                ("Authorization".to_string(), "Bearer staging".to_string()),
                ("X-Scope-OrgID".to_string(), "desksec".to_string()),
            ],
            "an explicit header override must replace the Grafana Basic header, \
             not sit alongside it"
        );
        // Header override with no token present at all still works.
        assert!(exporter_config_from_parts(
            Some("https://otlp.staging.internal"),
            Some("X-Scope-OrgID=desksec"),
            GRAFANA_INSTANCE_ID,
            None,
        )
        .is_some());
    }

    #[test]
    fn unusable_header_and_endpoint_overrides_stay_inert() {
        // Headers were supplied but parse to nothing: a misconfiguration, not
        // a reason to fall back to shipping data somewhere unintended.
        assert_eq!(
            exporter_config_from_parts(
                None,
                Some("garbage,=,a="),
                GRAFANA_INSTANCE_ID,
                Some(FAKE_TOKEN)
            ),
            None
        );
        // An explicitly bad endpoint override is inert rather than silently
        // falling back to the default.
        assert_eq!(
            exporter_config_from_parts(
                Some("ftp://otlp.grafana.internal"),
                None,
                GRAFANA_INSTANCE_ID,
                Some(FAKE_TOKEN),
            ),
            None
        );
        // An empty override is treated as "unset", so the default applies.
        assert!(exporter_config_from_parts(
            Some("   "),
            Some("   "),
            GRAFANA_INSTANCE_ID,
            Some(FAKE_TOKEN)
        )
        .is_some());
    }

    #[test]
    fn only_non_secret_values_are_hardcoded() {
        // The instance id and the gateway URL are non-secret and hardcoded on
        // purpose, so the app needs no per-machine configuration. The token is
        // never hardcoded: it only ever arrives from the build environment or
        // the process environment (see `telemetry_token`).
        assert_eq!(GRAFANA_INSTANCE_ID, "1549080");
        assert_eq!(
            DEFAULT_OTLP_ENDPOINT,
            "https://otlp-gateway-prod-eu-west-2.grafana.net/otlp"
        );
        // Placeholder-looking values are never accepted as a credential.
        assert_eq!(usable_value("your-token-here"), None);
        assert_eq!(usable_value("   "), None);
        assert_eq!(usable_value(" abc "), Some("abc".to_string()));
    }

    #[test]
    fn os_version_is_coarsened_to_major_minor() {
        assert_eq!(coarsen_os_version("15.3.1"), "15.3");
        assert_eq!(coarsen_os_version("15.3.1 (24D70)"), "15.3");
        assert_eq!(coarsen_os_version("22.04"), "22.04");
        assert_eq!(coarsen_os_version("14"), "14");
        assert_eq!(coarsen_os_version("  10.0.26100  "), "10.0");
        assert_eq!(coarsen_os_version("unknown"), "unknown");
        assert_eq!(coarsen_os_version(""), "unknown");
    }

    #[test]
    fn the_live_environment_description_stays_coarse() {
        let version = os_version();
        assert!(
            version.split('.').count() <= 2,
            "os.version must not carry a build number: {version:?}"
        );
        assert!(
            version == "unknown" || version.chars().all(|c| c.is_ascii_digit() || c == '.'),
            "os.version must be numeric or \"unknown\", never free text: {version:?}"
        );
        assert!(matches!(app_channel(), "debug" | "release"));
        assert!(looks_like_uuid(session_id()));
        assert_eq!(
            session_id(),
            session_id(),
            "the session id must be stable for the whole launch"
        );
        let _ = cpu_cores(); // must not panic on any platform
    }

    // -- spool ---------------------------------------------------------------

    #[test]
    fn spool_keeps_batches_in_order_and_round_trips_them() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::new(dir.path());
        assert!(spool.batches().is_empty());
        for i in 0..3u8 {
            assert_eq!(spool.push(&[b'a' + i; 8]), 0);
        }
        let files = spool.batches();
        assert_eq!(files.len(), 3);
        for (i, path) in files.iter().enumerate() {
            assert_eq!(std::fs::read(path).unwrap(), vec![b'a' + i as u8; 8]);
        }
        assert_eq!(spool.stats(), (3, 24));
    }

    #[test]
    fn spool_drops_the_oldest_batches_when_the_count_cap_is_hit() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::with_limits(dir.path(), 3, SPOOL_MAX_BYTES);
        let mut dropped = 0;
        for i in 0..5u8 {
            dropped += spool.push(&[b'a' + i; 8]);
        }
        assert_eq!(
            dropped, 2,
            "the two oldest batches must be reported as lost"
        );
        let files = spool.batches();
        assert_eq!(files.len(), 3);
        assert_eq!(
            std::fs::read(&files[0]).unwrap(),
            vec![b'c'; 8],
            "eviction must be oldest-first, not newest-first"
        );
    }

    #[test]
    fn spool_drops_the_oldest_batches_when_the_byte_cap_is_hit() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::with_limits(dir.path(), 1000, 300);
        let mut dropped = 0;
        for i in 0..5u8 {
            dropped += spool.push(&[b'a' + i; 100]);
        }
        assert_eq!(dropped, 2);
        assert_eq!(spool.stats(), (3, 300), "the byte cap must bind too");
        assert_eq!(std::fs::read(&spool.batches()[0]).unwrap()[0], b'c');
    }

    #[test]
    fn an_unwritable_spool_directory_is_not_fatal() {
        // The config dir path is an existing *file*, so `create_dir_all`
        // fails. Telemetry must lose the batch, not error and not panic.
        let dir = tempfile::tempdir().unwrap();
        let blocked = dir.path().join("this-is-a-file");
        std::fs::write(&blocked, b"not a directory").unwrap();
        let spool = Spool::new(&blocked);
        assert_eq!(spool.push(b"{}"), 0);
        assert!(spool.batches().is_empty());
        assert_eq!(spool.stats(), (0, 0));
        spool.purge(); // also must not panic
    }

    #[test]
    fn purge_spool_removes_every_spooled_batch() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::new(dir.path());
        for _ in 0..4 {
            spool.push(b"{\"resourceLogs\":[]}");
        }
        assert_eq!(spool.stats().0, 4);
        // The exact entry point commands.rs calls when the user opts out.
        purge_spool(dir.path());
        assert!(spool.batches().is_empty());
        assert!(!spool.dir.exists());
        purge_spool(dir.path()); // idempotent
    }

    // -- retry policy --------------------------------------------------------

    #[test]
    fn retryable_and_non_retryable_statuses_are_distinguished() {
        for status in [200, 201, 202, 204] {
            assert_eq!(classify_status(status), ExportOutcome::Delivered);
        }
        for status in [429, 500, 502, 503, 504] {
            assert_eq!(
                classify_status(status),
                ExportOutcome::Retry,
                "{status} is the collector saying \"later\""
            );
        }
        for status in [301, 400, 401, 403, 404, 413, 415, 422] {
            assert_eq!(
                classify_status(status),
                ExportOutcome::Discard,
                "{status} cannot be fixed by retrying — the batch must be dropped"
            );
        }
    }

    #[test]
    fn backoff_grows_exponentially_and_is_capped() {
        assert_eq!(backoff_base_secs(0), RETRY_BASE_SECS);
        assert_eq!(backoff_base_secs(1), RETRY_BASE_SECS * 2);
        assert_eq!(backoff_base_secs(2), RETRY_BASE_SECS * 4);
        let mut previous = 0;
        for attempt in 0..64 {
            let base = backoff_base_secs(attempt);
            assert!(
                base >= previous,
                "backoff must never shrink as it escalates"
            );
            assert!(
                base <= RETRY_MAX_SECS,
                "backoff must stay under the cap, got {base}"
            );
            previous = base;
        }
        assert_eq!(
            backoff_base_secs(40),
            RETRY_MAX_SECS,
            "a long outage must settle at the cap, not overflow"
        );
    }

    #[test]
    fn jitter_never_yields_a_zero_delay_and_stays_inside_its_window() {
        for base in [1, 2, 30, 300, RETRY_MAX_SECS] {
            for _ in 0..500 {
                let secs = jittered_delay(base).as_secs();
                assert!(secs >= 1, "a zero delay would spin the retry loop");
                assert!(
                    secs >= (base / 2).max(1),
                    "delay {secs}s fell below the window for base {base}s"
                );
                assert!(
                    secs <= base.max(1),
                    "delay {secs}s exceeded the window for base {base}s"
                );
            }
        }
    }

    #[test]
    fn backoff_widens_across_attempts_and_returns_to_base_on_reset() {
        let mut backoff = Backoff::new();
        let first = backoff.next_delay();
        assert!(first <= Duration::from_secs(RETRY_BASE_SECS));
        let mut last = first;
        for _ in 0..12 {
            last = backoff.next_delay();
        }
        assert!(last >= first, "repeated failure must widen the window");
        assert!(last <= Duration::from_secs(RETRY_MAX_SECS));
        backoff.reset();
        assert!(
            backoff.next_delay() <= Duration::from_secs(RETRY_BASE_SECS),
            "progress must bring the schedule back to the base window"
        );
    }

    // -- export path, end to end against a throwaway loopback collector ------

    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }

    struct TestCollector {
        url: String,
        received: Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
    }

    impl TestCollector {
        fn request_count(&self) -> usize {
            self.received.lock().unwrap().len()
        }
        fn last_body(&self) -> String {
            let bodies = self.received.lock().unwrap();
            String::from_utf8_lossy(bodies.last().expect("no request received")).into_owned()
        }
    }

    /// Minimal HTTP/1.1 collector: answers with the next status in `script`
    /// (the final one repeats) and records every body it received. This runs
    /// the real `reqwest` client through the real export path, so status
    /// classification, spooling and draining are tested for what they do
    /// rather than against a mock of themselves.
    async fn spawn_collector(script: Vec<u16>) -> TestCollector {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = Arc::clone(&received);
        let script = Arc::new(std::sync::Mutex::new(script));
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let sink = Arc::clone(&sink);
                let script = Arc::clone(&script);
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut buf: Vec<u8> = Vec::new();
                    let mut chunk = [0u8; 4096];
                    let head_end = loop {
                        match socket.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                        if let Some(at) = find_subslice(&buf, b"\r\n\r\n") {
                            break at + 4;
                        }
                        if buf.len() > 1 << 20 {
                            return;
                        }
                    };
                    let head = String::from_utf8_lossy(&buf[..head_end]).to_lowercase();
                    let len: usize = head
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length:"))
                        .and_then(|v| v.trim().parse().ok())
                        .unwrap_or(0);
                    while buf.len() < head_end + len {
                        match socket.read(&mut chunk).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    sink.lock().unwrap().push(buf[head_end..].to_vec());
                    let status = {
                        let mut script = script.lock().unwrap();
                        if script.len() > 1 {
                            script.remove(0)
                        } else {
                            script.first().copied().unwrap_or(200)
                        }
                    };
                    let response = format!(
                        "HTTP/1.1 {status} STATUS\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.flush().await;
                });
            }
        });
        TestCollector {
            url: format!("http://{addr}/v1/logs"),
            received,
        }
    }

    fn test_ctx(config_dir: &Path, enabled: bool) -> ExportContext {
        ExportContext {
            config_dir: config_dir.to_path_buf(),
            app_version: "1.2.3".into(),
            os: "linux".into(),
            os_version: "22.04".into(),
            arch: "aarch64".into(),
            cpu_cores: 10,
            channel: "release".into(),
            session_id: "99999999-8888-4777-8666-555555555555".into(),
            gate: Arc::new(AtomicBool::new(enabled)),
        }
    }

    fn test_exporter(config_dir: &Path, url: &str, enabled: bool) -> Exporter {
        Exporter {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .no_proxy() // a developer's HTTP_PROXY must not break the test
                .build()
                .unwrap(),
            config: ExporterConfig {
                endpoint: url.to_string(),
                headers: Vec::new(),
            },
            ctx: test_ctx(config_dir, enabled),
            spool: Spool::new(config_dir),
            backoff: Backoff::new(),
            dropped_batches: 0,
        }
    }

    fn one_event() -> Vec<Event> {
        vec![Event::new(
            "recording_completed",
            &[("duration_bucket", "5-15m".into())],
        )]
    }

    #[tokio::test]
    async fn a_failed_export_spools_and_a_later_success_drains_it() {
        let dir = tempfile::tempdir().unwrap();
        let collector = spawn_collector(vec![503, 200]).await;
        let mut ex = test_exporter(dir.path(), &collector.url, true);

        let mut pending = one_event();
        ex.flush(&mut pending).await;
        assert_eq!(
            ex.spool.stats().0,
            1,
            "a retryable failure must persist the batch, not drop it"
        );

        assert_eq!(ex.drain_spool().await, DrainResult::Progress);
        assert!(
            ex.spool.batches().is_empty(),
            "a later success must clear the spool"
        );
        assert_eq!(collector.request_count(), 2);
    }

    #[tokio::test]
    async fn rate_limited_and_server_errors_are_retried() {
        for status in [429u16, 500, 502, 503] {
            let dir = tempfile::tempdir().unwrap();
            let collector = spawn_collector(vec![status]).await;
            let mut ex = test_exporter(dir.path(), &collector.url, true);
            ex.flush(&mut one_event()).await;
            assert_eq!(
                ex.spool.stats().0,
                1,
                "{status} must be spooled for a later retry"
            );
        }
    }

    #[tokio::test]
    async fn non_retryable_statuses_discard_instead_of_spooling_forever() {
        for status in [400u16, 401, 403, 404, 422] {
            let dir = tempfile::tempdir().unwrap();
            let collector = spawn_collector(vec![status]).await;
            let mut ex = test_exporter(dir.path(), &collector.url, true);
            ex.flush(&mut one_event()).await;
            assert!(
                ex.spool.batches().is_empty(),
                "{status} means the payload or the auth is wrong; retrying it \
                 forever would fill the disk and hammer the collector"
            );
            assert_eq!(collector.request_count(), 1);
        }
    }

    #[tokio::test]
    async fn a_non_retryable_status_also_clears_a_spooled_batch() {
        let dir = tempfile::tempdir().unwrap();
        let collector = spawn_collector(vec![503, 401]).await;
        let mut ex = test_exporter(dir.path(), &collector.url, true);
        ex.flush(&mut one_event()).await;
        assert_eq!(ex.spool.stats().0, 1);
        // Credentials were revoked while we were offline: the spool must
        // empty itself rather than retry a doomed batch until the cap.
        assert_eq!(ex.drain_spool().await, DrainResult::Progress);
        assert!(ex.spool.batches().is_empty());
    }

    #[tokio::test]
    async fn opting_out_purges_the_spool_and_nothing_spooled_is_sent_afterwards() {
        let dir = tempfile::tempdir().unwrap();
        // The collector is healthy from the second request onwards, so if the
        // opt-out leaked anything at all this test would see it.
        let collector = spawn_collector(vec![503, 200]).await;
        let mut ex = test_exporter(dir.path(), &collector.url, true);

        ex.flush(&mut one_event()).await;
        assert_eq!(ex.spool.stats().0, 1, "precondition: something is spooled");
        let sent_before_opt_out = collector.request_count();

        // The user flips the Settings toggle off.
        ex.ctx.gate.store(false, Ordering::Relaxed);

        assert_eq!(ex.drain_spool().await, DrainResult::Empty);
        assert!(
            ex.spool.batches().is_empty(),
            "opting out must delete spooled batches from disk"
        );
        assert!(!ex.spool.dir.exists());

        // Anything emitted after the toggle is discarded, not queued to disk.
        let mut later = one_event();
        ex.flush(&mut later).await;
        assert!(later.is_empty());
        assert!(ex.spool.batches().is_empty());
        assert_eq!(
            collector.request_count(),
            sent_before_opt_out,
            "no batch may reach the collector after the user opts out"
        );
    }

    #[tokio::test]
    async fn startup_drains_a_spool_left_behind_by_a_previous_run() {
        let dir = tempfile::tempdir().unwrap();
        let collector = spawn_collector(vec![200]).await;

        // Simulate a previous, offline run that could not deliver its batch.
        let spool = Spool::new(dir.path());
        let payload = build_export_payload(&test_resource(), &one_event());
        spool.push(&serde_json::to_vec(&payload).unwrap());
        assert_eq!(spool.stats().0, 1);

        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let config = ExporterConfig {
            endpoint: collector.url.clone(),
            headers: Vec::new(),
        };
        let worker = tokio::spawn(export_worker(config, test_ctx(dir.path(), true), rx));
        // Closing the queue makes the worker flush and return, so the test
        // observes only the startup drain.
        drop(tx);
        worker.await.unwrap();

        assert!(
            spool.batches().is_empty(),
            "events must survive an app restart, not just a network blip"
        );
        assert_eq!(collector.request_count(), 1);
    }

    #[tokio::test]
    async fn a_corrupt_or_unreadable_spool_file_is_skipped_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let collector = spawn_collector(vec![200]).await;
        let spool = Spool::new(dir.path());
        std::fs::create_dir_all(&spool.dir).unwrap();
        // Truncated JSON, an empty file, and a stray partial write — none of
        // which may stop the one good batch behind them from going out.
        std::fs::write(spool.dir.join("00000000000000000001-0.json"), b"{ not json").unwrap();
        std::fs::write(spool.dir.join("00000000000000000002-0.json"), b"").unwrap();
        std::fs::write(spool.dir.join("00000000000000000003-0.json.tmp"), b"half").unwrap();
        let good =
            serde_json::to_vec(&build_export_payload(&test_resource(), &one_event())).unwrap();
        std::fs::write(spool.dir.join("00000000000000000004-0.json"), &good).unwrap();

        let mut ex = test_exporter(dir.path(), &collector.url, true);
        assert_eq!(ex.drain_spool().await, DrainResult::Progress);
        assert!(spool.batches().is_empty(), "bad files must be discarded");
        assert_eq!(
            collector.request_count(),
            1,
            "only the one readable batch may be sent"
        );
        // The `.tmp` partial write is ignored, never read back as a batch.
        assert!(spool.dir.join("00000000000000000003-0.json.tmp").exists());
    }

    #[tokio::test]
    async fn batches_lost_to_the_spool_bounds_are_reported_not_hidden() {
        let dir = tempfile::tempdir().unwrap();
        let collector = spawn_collector(vec![503, 503, 200]).await;
        let mut ex = test_exporter(dir.path(), &collector.url, true);
        ex.spool = Spool::with_limits(dir.path(), 1, SPOOL_MAX_BYTES);

        ex.flush(&mut one_event()).await;
        ex.flush(&mut one_event()).await;
        assert_eq!(ex.spool.stats().0, 1, "the count cap must hold");
        assert_eq!(ex.dropped_batches, 1);

        ex.flush(&mut one_event()).await; // collector is healthy again
        let body = collector.last_body();
        assert!(
            body.contains("telemetry_spool_dropped") && body.contains("\"dropped\""),
            "a gap in the data must be reported, not silently invisible: {body}"
        );
        assert_eq!(ex.dropped_batches, 0);
    }

    #[tokio::test]
    async fn spooled_payloads_on_disk_contain_no_content() {
        let dir = tempfile::tempdir().unwrap();
        let collector = spawn_collector(vec![503]).await;
        let mut ex = test_exporter(dir.path(), &collector.url, true);
        ex.flush(&mut vec![Event::new(
            "summary_generated",
            &[
                ("transcript_length_bucket", "5k-20k".into()),
                ("meeting_title", "Q3 board meeting — LAYOFFS".into()),
                ("transcript", "we decided to...".into()),
            ],
        )])
        .await;

        let files = ex.spool.batches();
        assert_eq!(files.len(), 1);
        let on_disk = String::from_utf8(std::fs::read(&files[0]).unwrap()).unwrap();
        assert!(!on_disk.contains("LAYOFFS"));
        assert!(!on_disk.contains("meeting_title"));
        assert!(!on_disk.contains("we decided to"));
        assert!(on_disk.contains("transcript_length_bucket"));
    }
}
