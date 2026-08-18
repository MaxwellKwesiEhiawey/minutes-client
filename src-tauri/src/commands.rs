use crate::db;
use crate::error::CategorizedError;
use crate::locking::MutexExt;
use crate::models::*;
use crate::permissions::PermissionsReport;
use crate::recorder;
use crate::settings::{self, SettingsView, VALID_TRANSCRIPTION_ENGINES, VALID_WHISPER_MODELS};
use crate::state::AppState;
use crate::summary;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

/// Every command's error type.
///
/// Was `Result<T, String>`. It carries an optional translation `code` now, so a
/// message the user is meant to read travels as an identifier the UI can look up
/// — the backend is never told the UI language, since that is a device-local
/// preference. Uncoded errors still work unchanged through `From<String>` /
/// `From<&str>` and remain diagnostic English, which is all they ever were.
type CmdResult<T> = Result<T, CategorizedError>;

#[tauri::command]
pub async fn start_recording(app: AppHandle, title: Option<String>) -> CmdResult<Meeting> {
    let result = recorder::start(&app, title);
    emit_recording_start_telemetry(&app, "manual", result.is_ok());
    result.map_err(CategorizedError::from)
}

/// Fire-and-forget telemetry for a recording-start attempt. Metadata only:
/// trigger + configuration flags on success, an error category on failure —
/// never the title or the error message (see docs/TELEMETRY.md).
pub fn emit_recording_start_telemetry(app: &AppHandle, trigger: &'static str, ok: bool) {
    if !ok {
        crate::telemetry::event(
            "recording_start_failed",
            &[
                ("trigger", trigger.into()),
                ("error.type", "internal".into()),
            ],
        );
        return;
    }
    let s = app.state::<AppState>().settings.lock_safe().clone();
    crate::telemetry::event(
        "recording_started",
        &[
            ("trigger", trigger.into()),
            ("engine", s.transcription_engine.as_str().into()),
            ("diarization", s.diarization_enabled.into()),
            ("capture_microphone", s.capture_microphone.into()),
            ("capture_system_audio", s.capture_system_audio.into()),
        ],
    );
}

#[tauri::command]
pub async fn stop_recording(app: AppHandle) -> CmdResult<String> {
    let meeting_id = recorder::stop(&app).await?;
    // Telemetry only after a successful stop; duration leaves the device as a
    // coarse bucket, never a raw value. Any failure here is swallowed — it
    // must not turn a successful stop into a user-visible error.
    let state = app.state::<AppState>();
    let engine = state.settings.lock_safe().transcription_engine.clone();
    let duration_secs = {
        let conn = state.db.lock_safe();
        db::get_meeting(&conn, &meeting_id)
            .ok()
            .flatten()
            .and_then(|m| meeting_duration_secs(&m))
    };
    if let Some(secs) = duration_secs {
        crate::telemetry::event(
            "recording_completed",
            &[
                ("engine", engine.as_str().into()),
                (
                    "duration_bucket",
                    crate::telemetry::duration_bucket_secs(secs).into(),
                ),
            ],
        );
    }
    Ok(meeting_id)
}

/// Wall-clock duration of a finished meeting, from its stored RFC3339 stamps.
fn meeting_duration_secs(meeting: &Meeting) -> Option<u64> {
    let start = chrono::DateTime::parse_from_rfc3339(&meeting.created_at).ok()?;
    let end = chrono::DateTime::parse_from_rfc3339(meeting.ended_at.as_deref()?).ok()?;
    u64::try_from((end - start).num_seconds()).ok()
}

#[tauri::command]
pub fn recording_state(state: State<AppState>) -> CmdResult<Option<String>> {
    Ok(state.recording_meeting_id())
}

#[tauri::command]
pub fn list_audio_devices() -> CmdResult<AudioDevicesResponse> {
    Ok(crate::audio::list_input_devices())
}

#[tauri::command]
pub fn list_meetings(state: State<AppState>) -> CmdResult<Vec<MeetingListItem>> {
    let conn = state.db.lock_safe();
    db::list_meetings(&conn).map_err(|e| CategorizedError::internal(e.to_string()))
}

#[tauri::command]
pub fn get_meeting(state: State<AppState>, meeting_id: String) -> CmdResult<MeetingDetail> {
    let conn = state.db.lock_safe();
    let meeting = db::get_meeting(&conn, &meeting_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| CategorizedError::coded("error.meetingNotFound", "meeting not found"))?;
    let segments = db::list_segments(&conn, &meeting_id).map_err(|e| e.to_string())?;
    let summary = db::get_summary(&conn, &meeting_id).map_err(|e| e.to_string())?;
    Ok(MeetingDetail {
        meeting,
        segments,
        summary,
    })
}

/// Full-text search across meeting titles and transcript segments. Returns the
/// matching meetings (newest first) with a short transcript snippet when the
/// match was found inside the transcript rather than the title.
#[tauri::command]
pub fn search_meetings(state: State<AppState>, query: String) -> CmdResult<Vec<MeetingSearchHit>> {
    let conn = state.db.lock_safe();
    db::search_meetings(&conn, &query).map_err(|e| CategorizedError::internal(e.to_string()))
}

#[tauri::command]
pub fn delete_meeting(state: State<AppState>, meeting_id: String) -> CmdResult<()> {
    if state.recording_meeting_id().as_deref() == Some(meeting_id.as_str()) {
        return Err(CategorizedError::coded(
            "error.deleteWhileRecording",
            "cannot delete a meeting that is currently recording",
        ));
    }
    let conn = state.db.lock_safe();
    db::delete_meeting(&conn, &meeting_id).map_err(|e| CategorizedError::internal(e.to_string()))
}

#[tauri::command]
pub fn rename_meeting(state: State<AppState>, meeting_id: String, title: String) -> CmdResult<()> {
    let conn = state.db.lock_safe();
    db::rename_meeting(&conn, &meeting_id, &title)
        .map_err(|e| CategorizedError::internal(e.to_string()))
}

/// Generate (or regenerate) the AI summary for a meeting via the Minutes server
/// (Fireworks AI).
///
/// Unlike every other command here, this returns `CategorizedError` instead
/// of the shared `CmdResult<T> = Result<T, String>` alias: this is the one
/// path that talks to an external server over the network, so it's the one
/// place the frontend actually benefits from telling "can't reach the
/// server" apart from "bad token" apart from "server had a problem" (see
/// `src/components/MeetingView.tsx`'s `summaryErrorCopy` and
/// `src/utils/errors.ts`). All the `String`-returning helpers below
/// (`db::list_segments`, `settings::validate_server_url`, ...) still work
/// unchanged with `?` because `CategorizedError: From<String>` (see
/// `error.rs`) — they just get classified as `Internal`.
///
/// NOT verified with a local `cargo build` (see CONTRIBUTING.md) — please
/// run `cargo check` after pulling this change.
#[tauri::command]
pub async fn generate_summary(
    app: AppHandle,
    meeting_id: String,
    instructions: Option<String>,
) -> Result<Summary, CategorizedError> {
    let state = app.state::<AppState>();

    // Pick up any `.env` edits without requiring an app restart.
    settings::reload_env_keys();

    let (settings, segments) = {
        let s = state.settings.lock_safe().clone();
        let conn = state.db.lock_safe();
        let segs = db::list_segments(&conn, &meeting_id).map_err(|e| e.to_string())?;
        (s, segs)
    };

    if segments.is_empty() {
        return Err("no transcript available to summarize".to_string().into());
    }
    let server_url = settings.server_url();
    // Never send the token + transcript over cleartext to a remote host.
    settings::validate_server_url(&server_url)?;
    // Registers this install on first use; afterwards this is a keychain read.
    let server_token = crate::device::auth_token(&state.http, &settings).await?;

    // Include speaker labels in the transcript we send for summarization so the
    // AI can attribute decisions and action items to the right person.
    let transcript = segments
        .iter()
        .map(|s| match &s.speaker_label {
            Some(sp) if !sp.is_empty() => format!("{sp}: {}", s.text),
            _ => s.text.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n");

    let merged_instructions =
        summary::merge_instructions(&settings.summary_instructions, instructions.as_deref());

    // The `From` impl in error.rs converts `summary::SummaryError` ->
    // `CategorizedError`, preserving the network/auth/server distinction
    // instead of collapsing it into a string with `.map_err(|e| e.to_string())`.
    // The match (rather than `?`) lets telemetry record the error *category* —
    // only the category; the message can embed URLs or other environment
    // details and is never sent.
    let started = std::time::Instant::now();
    let mut attempt = summary::summarize(
        &state.http,
        &server_url,
        &server_token,
        &settings.anthropic_model,
        &transcript,
        merged_instructions.as_deref(),
        &settings.summary_language,
    )
    .await;

    // A device token that has stopped being accepted usually means the server's
    // device registry was lost or restored from a backup. Registering again
    // turns a permanent failure for every client into one retried request.
    if matches!(attempt, Err(summary::SummaryError::Unauthorized)) {
        tracing::warn!("device token rejected during summary; re-registering");
        if let Ok(token) = crate::device::reregister(&state.http, &settings).await {
            attempt = summary::summarize(
                &state.http,
                &server_url,
                &token,
                &settings.anthropic_model,
                &transcript,
                merged_instructions.as_deref(),
                &settings.summary_language,
            )
            .await;
        }
    }

    let content = match attempt {
        Ok(content) => content,
        Err(e) => {
            let err = CategorizedError::from(e);
            crate::telemetry::event(
                "error",
                &[
                    ("area", "summary".into()),
                    ("error.type", err.kind.as_str().into()),
                ],
            );
            return Err(err);
        }
    };
    // `summarize_duration_bucket` alongside `transcript_length_bucket` is what
    // shows whether a slow summary tracks transcript size or is slow anyway.
    crate::telemetry::event(
        "summary_generated",
        &[
            ("engine", settings.transcription_engine.as_str().into()),
            (
                "transcript_length_bucket",
                crate::telemetry::transcript_length_bucket(transcript.chars().count()).into(),
            ),
            (
                "summarize_duration_bucket",
                crate::telemetry::latency_bucket_ms(started.elapsed().as_millis()).into(),
            ),
        ],
    );

    let saved = {
        let conn = state.db.lock_safe();
        let saved = db::upsert_summary(&conn, &meeting_id, &content, &settings.anthropic_model)
            .map_err(|e| e.to_string())?;
        // Adopt the AI-generated title for the meeting (nicer than the timestamp).
        let trimmed = content.title.trim();
        if !trimmed.is_empty() {
            let _ = db::rename_meeting(&conn, &meeting_id, trimmed);
        }
        saved
    };
    Ok(saved)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CmdResult<SettingsView> {
    // This is the first thing App.tsx asks the backend for once the webview
    // is running, so it is our "the UI is ready" moment for the startup
    // timing event. Only the first call ever reports; the rest are no-ops.
    crate::telemetry::mark_ui_ready();
    // Pick up any edits to `.env` without requiring a full app restart.
    settings::reload_env_keys();
    let s = state.settings.lock_safe();
    Ok(s.to_view())
}

/// Lightweight connectivity probe for the Settings UI.
#[derive(serde::Serialize)]
pub struct ServerStatus {
    pub configured: bool,
    pub reachable: bool,
    pub message: String,
}

#[tauri::command]
pub async fn check_server(state: State<'_, AppState>) -> CmdResult<ServerStatus> {
    settings::reload_env_keys();
    let settings = state.settings.lock_safe().clone();

    if settings.server_token().is_none() {
        return Ok(ServerStatus {
            configured: false,
            reachable: false,
            message: "Not configured — contact your IT team.".into(),
        });
    }

    let base = settings.server_url().trim_end_matches('/').to_string();
    let url = format!("{base}/healthz");

    match state.http.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => Ok(ServerStatus {
            configured: true,
            reachable: true,
            message: "Connected".into(),
        }),
        Ok(resp) => Ok(ServerStatus {
            configured: true,
            reachable: false,
            message: format!("Server error ({})", resp.status()),
        }),
        Err(e) => Ok(ServerStatus {
            configured: true,
            reachable: false,
            message: format!("Cannot reach server — is it running? ({e})"),
        }),
    }
}

#[derive(Deserialize)]
pub struct SettingsInput {
    pub server_url: Option<String>,
    pub server_token: Option<String>,
    pub whisper_model: Option<String>,
    pub transcription_engine: Option<String>,
    pub diarization_enabled: Option<bool>,
    pub export_markdown: Option<bool>,
    pub anthropic_model: Option<String>,
    pub chunk_secs: Option<f32>,
    pub partial_secs: Option<f32>,
    pub capture_microphone: Option<bool>,
    pub input_device: Option<String>,
    pub capture_system_audio: Option<bool>,
    pub system_audio_device: Option<String>,
    pub summary_instructions: Option<String>,
    pub transcription_language: Option<String>,
    pub summary_language: Option<String>,
    pub auto_summarize: Option<bool>,
    pub call_detection_enabled: Option<bool>,
    pub call_detection_cooldown_minutes: Option<u64>,
    pub call_detection_poll_interval_secs: Option<u64>,
    pub telemetry_enabled: Option<bool>,
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, input: SettingsInput) -> CmdResult<SettingsView> {
    if let Some(ref engine) = input.transcription_engine {
        let e = engine.trim();
        if VALID_TRANSCRIPTION_ENGINES.contains(&e) && state.recording_meeting_id().is_some() {
            let current = state.settings.lock_safe().transcription_engine.clone();
            if !e.eq_ignore_ascii_case(&current) {
                return Err(CategorizedError::coded(
                    "error.stopBeforeEngineChange",
                    "Stop recording before changing the transcription engine.",
                ));
            }
        }
    }

    let mut guard = state.settings.lock_safe();
    let mut next = guard.clone();

    if let Some(u) = input.server_url {
        if settings::embedded_api_url().is_some() {
            tracing::debug!("ignoring server_url save — configured at CI build time");
        } else if !u.trim().is_empty() {
            // Reject cleartext remote URLs before persisting (token + transcript
            // are sent to this host).
            settings::validate_server_url(u.trim())?;
            // settings.json (written unconditionally below) is the source of
            // truth for this value now — also mirror it to the OS credential
            // store for backward compat, but don't let a keychain write
            // failure (e.g. a locked/inaccessible keychain) block persisting
            // the setting the user actually asked to save.
            if let Err(e) = crate::secrets::set_api_url(u.trim()) {
                tracing::warn!("failed to mirror server URL to OS store: {e}");
            }
            next.server_url = u.trim().to_string();
        }
    }
    // Empty token means "leave unchanged" so we never clobber a stored token
    // just because the masked field came back blank.
    if let Some(t) = input.server_token {
        if settings::embedded_token().is_some() {
            tracing::debug!("ignoring server_token save — configured at CI build time");
        } else if !t.trim().is_empty() {
            crate::secrets::set_token(t.trim()).map_err(|e| e.to_string())?;
            next.server_token = t.trim().to_string();
        }
    }
    if let Some(m) = input.whisper_model {
        let m = m.trim();
        if VALID_WHISPER_MODELS.contains(&m) && m != guard.whisper_model {
            crate::local_transcribe::invalidate_whisper_runtime_cache();
        }
        if VALID_WHISPER_MODELS.contains(&m) {
            next.whisper_model = m.to_string();
        }
    }
    if let Some(e) = input.transcription_engine {
        let e = settings::normalize_transcription_engine(e.trim());
        next.transcription_engine = e;
    }
    if let Some(d) = input.diarization_enabled {
        next.diarization_enabled = d;
    }
    if let Some(x) = input.export_markdown {
        next.export_markdown = x;
    }
    if let Some(m) = input.anthropic_model {
        if !m.trim().is_empty() {
            next.anthropic_model = m.trim().to_string();
        }
    }
    if let Some(c) = input.chunk_secs {
        next.chunk_secs = c.clamp(2.0, 60.0);
    }
    if let Some(p) = input.partial_secs {
        next.partial_secs = if p <= 0.0 { 0.0 } else { p.clamp(1.0, 30.0) };
    }
    if let Some(on) = input.capture_microphone {
        next.capture_microphone = on;
    }
    if let Some(d) = input.input_device {
        // Empty string means "system default".
        next.input_device = if d.trim().is_empty() { None } else { Some(d) };
    }
    if let Some(on) = input.capture_system_audio {
        next.capture_system_audio = on;
    }
    if let Some(d) = input.system_audio_device {
        next.system_audio_device = if d.trim().is_empty() { None } else { Some(d) };
    }
    // A recording needs at least one source; refuse a config that has none
    // rather than failing at start_recording time with a confusing message.
    if !next.capture_microphone && !next.capture_system_audio {
        return Err(CategorizedError::coded(
            "error.noCaptureSource",
            "Enable the microphone, system audio, or both.",
        ));
    }
    // Always assignable, including clearing it back to empty.
    if let Some(instr) = input.summary_instructions {
        next.summary_instructions = instr.trim().to_string();
    }
    // Language selections are always assignable (empty = provider/model default).
    if let Some(lang) = input.transcription_language {
        if lang.trim() != guard.transcription_language {
            crate::local_transcribe::invalidate_whisper_runtime_cache();
        }
        next.transcription_language = lang.trim().to_string();
    }
    if let Some(lang) = input.summary_language {
        next.summary_language = lang.trim().to_string();
    }
    if let Some(on) = input.auto_summarize {
        next.auto_summarize = on;
    }
    if let Some(on) = input.call_detection_enabled {
        next.call_detection_enabled = on;
    }
    if let Some(m) = input.call_detection_cooldown_minutes {
        next.call_detection_cooldown_minutes = m.clamp(0, 120);
    }
    if let Some(s) = input.call_detection_poll_interval_secs {
        next.call_detection_poll_interval_secs = s.clamp(1, 30);
    }
    if let Some(on) = input.telemetry_enabled {
        if on != next.telemetry_enabled {
            if on {
                crate::telemetry::set_enabled(true);
            } else {
                // Opting out: stop all emission immediately, delete anything
                // already spooled to disk (otherwise "off" would still leak
                // the moment the collector came back), then delete the
                // pseudonymous install id so a later opt-in starts with a
                // fresh id, unlinkable to anything sent before.
                crate::telemetry::set_enabled(false);
                crate::telemetry::purge_spool(&state.config_dir);
                crate::telemetry::reset_install_id(&state.config_dir);
            }
        }
        next.telemetry_enabled = on;
    }

    settings::save(&state.config_dir, &next).map_err(|e| e.to_string())?;
    *guard = next;
    Ok(guard.to_view())
}

/// Load everything the export and share paths render from, in one lock.
///
/// Extracted because `export_markdown`, `export_docx`, `export_pdf` and
/// `share_meeting` all need exactly this triple; four copies of the same three
/// queries is how one of them ends up subtly different.
fn load_for_export(
    state: &State<AppState>,
    meeting_id: &str,
) -> CmdResult<(Meeting, Vec<Segment>, Option<Summary>)> {
    let conn = state.db.lock_safe();
    let meeting = db::get_meeting(&conn, meeting_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| CategorizedError::coded("error.meetingNotFound", "meeting not found"))?;
    let segments = db::list_segments(&conn, meeting_id).map_err(|e| e.to_string())?;
    let summary = db::get_summary(&conn, meeting_id).map_err(|e| e.to_string())?;
    Ok((meeting, segments, summary))
}

/// Write a meeting to `path` in `format`. Shared by the save-to-device commands
/// and by sharing, so the three formats behave identically down both routes.
fn render_to_path(
    path: &std::path::Path,
    format: crate::share::ShareFormat,
    meeting: &Meeting,
    segments: &[Segment],
    summary: &Option<Summary>,
    include_transcript: bool,
) -> Result<(), String> {
    use crate::share::ShareFormat;
    let path_str = path.to_str().ok_or("the export path is not valid UTF-8")?;
    match format {
        ShareFormat::Pdf => {
            crate::pdf_export::write_pdf(path_str, meeting, segments, summary, include_transcript)
                .map_err(|e| e.to_string())
        }
        ShareFormat::Docx => {
            crate::docx_export::write_docx(path_str, meeting, segments, summary, include_transcript)
                .map_err(|e| e.to_string())
        }
        ShareFormat::Md => {
            let md = render_markdown(meeting, segments, summary, include_transcript);
            std::fs::write(path, md).map_err(|e| e.to_string())
        }
    }
}

/// Write a meeting to a managed temporary file and hand it to the OS share
/// picker, so it can go straight into Mail, Messages, AirDrop or any app with a
/// share extension.
///
/// The path is chosen entirely in `share.rs` — the webview never supplies one —
/// and the staging directory is swept at startup and before each share, so a
/// shared transcript does not linger on disk.
#[tauri::command]
pub fn share_meeting(
    app: AppHandle,
    state: State<AppState>,
    meeting_id: String,
    format: crate::share::ShareFormat,
    include_transcript: bool,
) -> CmdResult<()> {
    if !crate::share::supported() {
        return Err(CategorizedError::coded(
            "error.shareUnsupported",
            "Sharing to another app isn't available on this platform — save the file instead.",
        ));
    }

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("could not resolve the cache directory: {e}"))?;
    crate::share::purge_stale(&cache_dir);

    let (meeting, segments, summary) = load_for_export(&state, &meeting_id)?;
    if segments.is_empty() && summary.is_none() {
        return Err(CategorizedError::coded(
            "error.nothingToShare",
            "there is nothing to share for this meeting yet",
        ));
    }

    let path = crate::share::stage_path(&cache_dir, &meeting.title, format)
        .map_err(|e| format!("{e:#}"))?;
    // Create with owner-only permissions first, then let the writers fill it —
    // otherwise the file would briefly exist with the process umask.
    drop(crate::share::create_staged_file(&path).map_err(|e| format!("{e:#}"))?);
    render_to_path(
        &path,
        format,
        &meeting,
        &segments,
        &summary,
        include_transcript,
    )?;

    let window = app.get_webview_window("main").ok_or_else(|| {
        CategorizedError::coded(
            "error.noWindowToShare",
            "the main window is not available to share from",
        )
    })?;
    crate::share::present_for_window(&window, path).map_err(|e| format!("{e:#}"))?;

    crate::telemetry::event(
        "share_opened",
        &[
            ("format", format.as_str().into()),
            ("include_transcript", include_transcript.into()),
        ],
    );
    Ok(())
}

/// Render a meeting as Markdown for sharing/export.
///
/// `include_transcript` is the user's choice in the share dialog: a summary is
/// safe to pass around, while the verbatim transcript is the sensitive part, so
/// it can be left out of the file.
#[tauri::command]
pub fn export_markdown(
    state: State<AppState>,
    meeting_id: String,
    include_transcript: bool,
) -> CmdResult<String> {
    let (meeting, segments, summary) = load_for_export(&state, &meeting_id)?;
    Ok(render_markdown(
        &meeting,
        &segments,
        &summary,
        include_transcript,
    ))
}

/// Validate a user-supplied export path. The path originates from the native
/// save dialog, but this command is reachable from the webview, so we constrain
/// it to an absolute path with an allowed document extension. This turns a raw
/// arbitrary-file-write primitive into "write a document to a chosen location"
/// and prevents clobbering shell rc files, launch agents, binaries, etc.
fn validate_export_path(path: &str, allowed_exts: &[&str]) -> CmdResult<()> {
    let p = std::path::Path::new(path);
    if !p.is_absolute() {
        return Err(CategorizedError::coded(
            "error.exportPathNotAbsolute",
            "export path must be an absolute path",
        ));
    }
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext {
        Some(e) if allowed_exts.contains(&e.as_str()) => Ok(()),
        _ => Err(CategorizedError::coded(
            "error.exportExtension",
            format!(
                "refusing to write a file without an allowed extension ({})",
                allowed_exts.join(", ")
            ),
        )),
    }
}

/// Write text to a user-chosen path (the frontend obtains the path via the
/// native save dialog). Done in Rust to avoid fs-plugin scope configuration.
/// Restricted to Markdown/plain-text document extensions (see
/// [`validate_export_path`]).
#[tauri::command]
pub fn write_text_file(path: String, contents: String) -> CmdResult<()> {
    validate_export_path(&path, &["md", "markdown", "txt"])?;
    std::fs::write(&path, contents).map_err(|e| e.to_string())?;
    // Telemetry: the export *format* only — never the path or the contents.
    let format: &'static str = if path.to_ascii_lowercase().ends_with(".txt") {
        "txt"
    } else {
        "md"
    };
    crate::telemetry::event("export_completed", &[("format", format.into())]);
    Ok(())
}

/// Build a Word (.docx) document for a meeting (summary + transcript) and write
/// it to the user-chosen `path`.
#[tauri::command]
pub fn export_docx(
    state: State<AppState>,
    meeting_id: String,
    path: String,
    include_transcript: bool,
) -> CmdResult<()> {
    validate_export_path(&path, &["docx"])?;
    let (meeting, segments, summary) = load_for_export(&state, &meeting_id)?;
    crate::docx_export::write_docx(&path, &meeting, &segments, &summary, include_transcript)
        .map_err(|e| e.to_string())?;
    crate::telemetry::event(
        "export_completed",
        &[
            ("format", "docx".into()),
            ("include_transcript", include_transcript.into()),
        ],
    );
    Ok(())
}

/// Build a PDF for a meeting (summary + transcript) and write it to the
/// user-chosen `path`.
#[tauri::command]
pub fn export_pdf(
    state: State<AppState>,
    meeting_id: String,
    path: String,
    include_transcript: bool,
) -> CmdResult<()> {
    validate_export_path(&path, &["pdf"])?;
    let (meeting, segments, summary) = load_for_export(&state, &meeting_id)?;
    crate::pdf_export::write_pdf(&path, &meeting, &segments, &summary, include_transcript)
        .map_err(|e| e.to_string())?;
    crate::telemetry::event(
        "export_completed",
        &[
            ("format", "pdf".into()),
            ("include_transcript", include_transcript.into()),
        ],
    );
    Ok(())
}

/// Status of the on-device transcription models for the Settings UI.
#[derive(serde::Serialize)]
pub struct TranscriptionStatus {
    pub model: String,
    pub model_ready: bool,
    pub diarization_enabled: bool,
}

/// Resolve which whisper model to probe or download. When `model` is omitted,
/// use the saved setting. The Settings dropdown passes the in-progress selection
/// so status/download work before Save.
fn resolve_whisper_model(
    settings: &settings::Settings,
    model: Option<&str>,
) -> Result<String, String> {
    let name = model
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(settings.whisper_model.as_str());
    if VALID_WHISPER_MODELS.contains(&name) {
        Ok(name.to_string())
    } else {
        Err(format!("unsupported whisper model: {name}"))
    }
}

fn transcription_status_for_whisper(
    settings: &settings::Settings,
    model: &str,
) -> TranscriptionStatus {
    let cfg = crate::local_transcribe::build_config(
        model,
        &settings.transcription_language,
        settings.diarization_enabled,
    );
    TranscriptionStatus {
        model: model.to_string(),
        model_ready: crate::local_transcribe::model_likely_present(&cfg),
        diarization_enabled: settings.diarization_enabled,
    }
}

async fn transcription_status_for_deepgram(
    http: &reqwest::Client,
    settings: &settings::Settings,
) -> TranscriptionStatus {
    // A bootstrap token is enough to *be able* to reach the server: the device
    // token is minted on demand from it.
    let token_ok = settings.server_token().is_some();
    let mut model = "deepgram".to_string();
    let mut configured = false;

    if token_ok {
        if let Ok(token) = crate::device::auth_token(http, settings).await {
            if let Ok(status) =
                crate::remote_transcribe::fetch_status(http, &settings.server_url(), &token).await
            {
                configured = status.configured;
                model = status.model;
            }
        }
    }

    TranscriptionStatus {
        model,
        model_ready: token_ok && configured,
        diarization_enabled: settings.diarization_enabled,
    }
}

/// Report transcription readiness for the active engine.
#[tauri::command]
pub async fn transcription_status(
    state: State<'_, AppState>,
    model: Option<String>,
) -> CmdResult<TranscriptionStatus> {
    settings::reload_env_keys();
    let settings = state.settings.lock_safe().clone();

    if settings::is_whisper_engine(&settings.transcription_engine) {
        let model = resolve_whisper_model(&settings, model.as_deref())?;
        Ok(transcription_status_for_whisper(&settings, &model))
    } else {
        Ok(transcription_status_for_deepgram(&state.http, &settings).await)
    }
}

/// Download the on-device transcription (whisper), VAD, and — when enabled —
/// diarization models. This is Minutes's one-click equivalent of
/// `minutes setup --model <model> [--diarization]`. Progress is streamed to the
/// frontend via the `model-download-progress` event.
///
/// Pass `model` from the Settings dropdown to download a selection before Save.
#[tauri::command]
pub async fn download_models(
    app: AppHandle,
    state: State<'_, AppState>,
    model: Option<String>,
) -> CmdResult<TranscriptionStatus> {
    let settings = state.settings.lock_safe().clone();

    if !settings::is_whisper_engine(&settings.transcription_engine) {
        return Ok(transcription_status_for_deepgram(&state.http, &settings).await);
    }

    let model = resolve_whisper_model(&settings, model.as_deref())?;
    let cfg = crate::local_transcribe::build_config(
        &model,
        &settings.transcription_language,
        settings.diarization_enabled,
    );
    // A model download that fails or takes forever is an activation blocker:
    // the user cannot transcribe anything until it finishes. Outcome plus a
    // duration bucket only — never the URL, the path, or the file name.
    let started = std::time::Instant::now();
    let result =
        crate::local_transcribe::ensure_models(&app, &cfg, settings.diarization_enabled).await;
    crate::telemetry::event(
        "model_download_completed",
        &[
            ("whisper_model", model.as_str().into()),
            (
                "outcome",
                if result.is_ok() { "success" } else { "failed" }.into(),
            ),
            (
                "download_duration_bucket",
                crate::telemetry::latency_bucket_ms(started.elapsed().as_millis()).into(),
            ),
        ],
    );
    result?;
    Ok(transcription_status_for_whisper(&settings, &model))
}

fn installed_models_for(
    settings: &settings::Settings,
) -> crate::local_transcribe::InstalledModelsInfo {
    let cfg = crate::local_transcribe::build_config(
        &settings.whisper_model,
        &settings.transcription_language,
        settings.diarization_enabled,
    );
    crate::local_transcribe::list_installed_models(
        &cfg,
        &settings.whisper_model,
        settings.diarization_enabled,
    )
}

/// List on-device model files the user can delete to reclaim disk space.
#[tauri::command]
pub fn list_installed_models(
    state: State<AppState>,
) -> CmdResult<crate::local_transcribe::InstalledModelsInfo> {
    let settings = state.settings.lock_safe().clone();
    Ok(installed_models_for(&settings))
}

/// Delete one installed model group (`tiny`…`large-v3`, `vad`, or `diarization`).
#[tauri::command]
pub fn delete_installed_model(
    state: State<AppState>,
    model_id: String,
) -> CmdResult<crate::local_transcribe::InstalledModelsInfo> {
    let settings = state.settings.lock_safe().clone();
    let cfg = crate::local_transcribe::build_config(
        &settings.whisper_model,
        &settings.transcription_language,
        settings.diarization_enabled,
    );
    if state.recording_meeting_id().is_some() {
        return Err(CategorizedError::coded(
            "error.stopBeforeDeletingModels",
            "Stop recording before deleting models.",
        ));
    }
    crate::local_transcribe::delete_installed_model(
        &cfg,
        model_id.trim(),
        &settings.whisper_model,
        // The command already refused above; the helper keeps its own guard for
        // its own tests and for any future caller.
        false,
    )?;
    Ok(installed_models_for(&settings))
}

// ---------------------------------------------------------------------------
// First-run permission onboarding (see permissions.rs)
// ---------------------------------------------------------------------------

/// Everything onboarding covers, plus which steps this install should see.
///
/// A pure read — it never prompts — so the UI can call it on mount and after
/// each grant without side effects.
#[tauri::command]
pub async fn permission_status(state: State<'_, AppState>) -> CmdResult<PermissionsReport> {
    let completed = state.settings.lock_safe().onboarding_completed_version;
    let preexisting = settings::was_preexisting_install();
    // Off the UI thread: each probe shells out to `open -Ra` and `osascript`
    // once per installed browser.
    let report = tauri::async_runtime::spawn_blocking(move || {
        crate::permissions::report(preexisting, completed)
    })
    .await
    .map_err(|e| CategorizedError::internal(format!("permission probe failed: {e}")))?;

    // Nothing to ask on this platform / already-granted upgrade: stamp it now so
    // this is not recomputed on every launch.
    if !report.onboarding_required && report.completed_version < report.current_version {
        persist_onboarding_version(&state, report.current_version)?;
    }
    Ok(report)
}

/// Raise the microphone prompt. Only does anything when it has never been asked.
#[tauri::command]
pub async fn request_microphone() -> CmdResult<crate::permissions::PermissionState> {
    // Blocking on purpose: the TCC dialog is modal and we want the real answer
    // rather than a stale status. `spawn_blocking` keeps the window responsive
    // behind it.
    tauri::async_runtime::spawn_blocking(crate::permissions::request_microphone)
        .await
        .map_err(|e| CategorizedError::internal(format!("microphone request failed: {e}")))
}

/// Ask for Automation consent for one browser.
#[tauri::command]
pub async fn request_browser_automation(
    app_name: String,
) -> CmdResult<crate::permissions::PermissionState> {
    // The app name is interpolated into an AppleScript body, so it must be one
    // of ours and never an arbitrary string from the webview.
    if !crate::permissions::is_known_browser(&app_name) {
        return Err(CategorizedError::coded(
            "error.unknownBrowser",
            "That browser is not one Minutes can detect meetings in.",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || {
        crate::permissions::request_browser_automation(&app_name)
    })
    .await
    .map_err(|e| CategorizedError::internal(format!("automation request failed: {e}")))
}

/// Open the OS privacy pane for a permission we cannot grant ourselves.
///
/// Takes an enum, not a URL: the mapping lives in Rust so a webview string can
/// never become something the app opens.
#[tauri::command]
pub async fn open_privacy_settings(
    app: AppHandle,
    pane: crate::permissions::PrivacyPane,
) -> CmdResult<()> {
    let Some(url) = pane.url() else {
        return Err(CategorizedError::coded(
            "error.noPrivacyPane",
            "This system has no settings page for that permission.",
        ));
    };
    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_url(url, None::<&str>)
        .map_err(|e| CategorizedError::internal(format!("could not open settings: {e}")))
}

/// Mark onboarding done. Called on Finish *and* on Skip — someone who declined
/// has made a choice, and re-asking every launch is the ambush this replaces.
#[tauri::command]
pub async fn complete_onboarding(state: State<'_, AppState>) -> CmdResult<SettingsView> {
    persist_onboarding_version(&state, crate::permissions::ONBOARDING_VERSION)?;
    Ok(state.settings.lock_safe().to_view())
}

/// Reset the marker so setup can be run again from Settings.
#[tauri::command]
pub async fn reset_onboarding(state: State<'_, AppState>) -> CmdResult<PermissionsReport> {
    persist_onboarding_version(&state, 0)?;
    let report = tauri::async_runtime::spawn_blocking(move || {
        // Deliberately reported as a fresh install so re-running setup walks the
        // full sequence rather than only what is still outstanding — the user
        // asked to see it again.
        crate::permissions::report(false, 0)
    })
    .await
    .map_err(|e| CategorizedError::internal(format!("permission probe failed: {e}")))?;
    Ok(report)
}

/// Write the onboarding marker through the normal settings path.
///
/// Kept separate from `save_settings` because that command merges a partial view
/// from the UI; this writes one backend-owned field and must not be reachable as
/// a UI-supplied value.
fn persist_onboarding_version(state: &State<AppState>, version: u32) -> CmdResult<()> {
    let mut guard = state.settings.lock_safe();
    if guard.onboarding_completed_version == version {
        return Ok(());
    }
    let mut next = guard.clone();
    next.onboarding_completed_version = version;
    settings::save(&state.config_dir, &next).map_err(|e| e.to_string())?;
    *guard = next;
    Ok(())
}

fn render_markdown(
    meeting: &Meeting,
    segments: &[Segment],
    summary: &Option<Summary>,
    include_transcript: bool,
) -> String {
    let mut out = String::new();
    let title = summary
        .as_ref()
        .map(|s| s.content.title.clone())
        .unwrap_or_else(|| meeting.title.clone());

    out.push_str(&format!("# {title}\n\n"));
    let times = crate::markdown::meeting_times(&meeting.created_at, meeting.ended_at.as_deref());
    out.push_str(&format!("- **Date:** {}\n", times.when));
    if let Some(d) = &times.duration {
        out.push_str(&format!("- **Duration:** {d}\n"));
    }
    // "completed" is internal bookkeeping; only surface an abnormal status.
    if meeting.status != "completed" {
        out.push_str(&format!("- **Status:** {}\n", meeting.status));
    }
    out.push('\n');

    if let Some(s) = summary {
        let c = &s.content;
        out.push_str("## Summary\n\n");
        out.push_str(&format!("{}\n\n", c.executive_summary));

        crate::markdown::push_key_topics(&mut out, &c.key_topics);
        if !c.decisions.is_empty() {
            out.push_str("### Decisions\n\n");
            for d in &c.decisions {
                // The parenthetical is omitted entirely when there is no owner.
                match crate::markdown::owner_note(d.owner.as_deref()) {
                    Some(note) => out.push_str(&format!("- {} _({note})_\n", d.text)),
                    None => out.push_str(&format!("- {}\n", d.text)),
                }
            }
            out.push('\n');
        }
        if !c.action_items.is_empty() {
            out.push_str("### Action Items\n\n");
            for a in &c.action_items {
                match crate::markdown::assignment_note(a.assignee.as_deref(), a.due.as_deref()) {
                    Some(note) => out.push_str(&format!("- [ ] {} _({note})_\n", a.task)),
                    None => out.push_str(&format!("- [ ] {}\n", a.task)),
                }
            }
            out.push('\n');
        }
        crate::markdown::push_open_questions(&mut out, &c.open_questions);
    }

    if include_transcript {
        crate::markdown::push_transcript(&mut out, segments);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    mod share_flag {
        use super::*;
        use crate::models::{Meeting, Segment, Summary, SummaryContent};

        const SECRET: &str = "a sentence nobody outside the room should read";

        fn fixture() -> (Meeting, Vec<Segment>, Option<Summary>) {
            let meeting = Meeting {
                id: "m1".into(),
                title: "Weekly sync".into(),
                status: "completed".into(),
                created_at: "2026-08-11T09:30:00+00:00".into(),
                ended_at: Some("2026-08-11T10:01:00+00:00".into()),
            };
            let segments = vec![Segment {
                id: 1,
                meeting_id: "m1".into(),
                seq: 1,
                text: SECRET.into(),
                created_at: "2026-08-11T09:31:00+00:00".into(),
                speaker_label: None,
                speaker_name: Some("Ama".into()),
                start_ms: Some(1000),
                end_ms: Some(2000),
            }];
            let summary = Some(Summary {
                meeting_id: "m1".into(),
                model: "test".into(),
                created_at: "2026-08-11T10:20:00+00:00".into(),
                content: SummaryContent {
                    title: "Weekly sync".into(),
                    executive_summary: "We agreed to ship on Thursday.".into(),
                    key_topics: vec![],
                    decisions: vec![],
                    action_items: vec![],
                    open_questions: vec![],
                },
            });
            (meeting, segments, summary)
        }

        #[test]
        fn including_the_transcript_puts_it_in_the_markdown() {
            let (m, segs, sum) = fixture();
            let md = render_markdown(&m, &segs, &sum, true);
            assert!(md.contains("## Transcript"), "missing heading: {md}");
            assert!(md.contains(SECRET));
        }

        #[test]
        fn omitting_the_transcript_leaves_the_summary_intact() {
            let (m, segs, sum) = fixture();
            let md = render_markdown(&m, &segs, &sum, false);
            // The verbatim record is gone, heading and all…
            assert!(!md.contains("Transcript"), "transcript survived: {md}");
            assert!(!md.contains(SECRET), "transcript text survived: {md}");
            // …and everything the summary contributes is still there.
            assert!(md.contains("# Weekly sync"));
            assert!(md.contains("We agreed to ship on Thursday."));
            assert!(md.contains("**Date:**"));
        }

        #[test]
        fn omitting_the_transcript_without_a_summary_yields_only_the_header() {
            // The UI forces the transcript back in when there is no summary,
            // precisely because this is all that would be left.
            let (m, segs, _) = fixture();
            let md = render_markdown(&m, &segs, &None, false);
            assert!(md.contains("# Weekly sync"));
            assert!(!md.contains(SECRET));
        }
    }

    /// `Path::is_absolute` is platform-specific: on Windows a leading `/` is
    /// drive-*relative*, so fixtures need a drive letter to get past the
    /// absolute-path check and actually exercise the extension filter.
    #[cfg(windows)]
    const ABS_PREFIX: &str = r"C:\temp\";
    #[cfg(not(windows))]
    const ABS_PREFIX: &str = "/tmp/";

    fn abs(rest: &str) -> String {
        format!("{ABS_PREFIX}{rest}")
    }

    #[test]
    fn export_path_requires_absolute_and_allowed_extension() {
        // Allowed extensions on an absolute path succeed.
        assert!(validate_export_path(&abs("meeting.md"), &["md", "txt"]).is_ok());
        assert!(validate_export_path(&abs("meeting.MD"), &["md", "txt"]).is_ok()); // case-insensitive
        assert!(validate_export_path(&abs("report.docx"), &["docx"]).is_ok());
    }

    #[test]
    fn export_path_rejects_relative_and_dangerous_targets() {
        // Relative paths are refused.
        assert!(validate_export_path("meeting.md", &["md"]).is_err());
        // A leading separator with no drive letter is not absolute on Windows.
        #[cfg(windows)]
        assert!(validate_export_path("/tmp/meeting.md", &["md"]).is_err());
        // Disallowed / missing extensions are refused (no clobbering rc files, binaries, plists).
        assert!(validate_export_path(&abs(".zshrc"), &["md", "txt"]).is_err());
        assert!(validate_export_path(&abs("bin/tool"), &["md"]).is_err());
        assert!(validate_export_path(&abs("evil.sh"), &["md", "docx"]).is_err());
    }
}
