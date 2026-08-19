use crate::audio::{self, CaptureSession, CaptureSource};
use crate::db;
use crate::local_transcribe::{self, SpeakerLine};
use crate::locking::MutexExt;
use crate::models::{Meeting, MeetingStatus};
use crate::remote_stream;
use crate::settings::{self, Settings};
use crate::state::AppState;
use minutes_core::config::Config;
use rusqlite::Connection;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::UnboundedReceiver;

/// Event names emitted to the frontend.
pub const EV_STATUS: &str = "recording-status";
pub const EV_PARTIAL: &str = "transcript-partial";
pub const EV_FINAL: &str = "transcript-final";
pub const EV_ERROR: &str = "transcript-error";
pub const EV_LEVEL: &str = "audio-level";
/// Non-fatal capture news: a device was swapped mid-recording, or a source
/// couldn't be opened. Distinct from `EV_ERROR`, which means the transcript
/// itself is failing.
pub const EV_CAPTURE: &str = "capture-notice";

/// Tracks the live recording so it can be torn down on stop.
pub struct RecordingSession {
    pub meeting_id: String,
    capture: Option<CaptureSession>,
    processing: Option<tauri::async_runtime::JoinHandle<()>>,
}

/// Begin a new recording: open the audio device, create the meeting record, and
/// spawn the transcription pipeline. Returns the freshly created meeting.
pub fn start(app: &AppHandle, title: Option<String>) -> Result<Meeting, String> {
    let state = app.state::<AppState>();

    // Atomically claim the start slot before doing any work. If two starts
    // race (e.g. main window + floating prompt, or a double-pressed Enter),
    // exactly one gets the claim; the other fails here — before opening a
    // device or inserting a meeting row. The claim is an RAII guard: every
    // early `?` return below drops it and frees the slot again.
    let claim = state.try_claim_recording_slot()?;

    let settings = state.settings.lock_safe().clone();

    let cfg = if settings::is_whisper_engine(&settings.transcription_engine) {
        let cfg = local_transcribe::build_config(
            &settings.whisper_model,
            &settings.transcription_language,
            settings.diarization_enabled,
        );
        if !local_transcribe::model_likely_present(&cfg) {
            let model_path = local_transcribe::model_file(&cfg);
            if model_path.exists() {
                return Err(format!(
                    "The on-device transcription model '{}' looks corrupted. Open Settings, download it again, then start recording.",
                    settings.whisper_model
                ));
            }
            return Err(format!(
                "The on-device transcription model '{}' isn't downloaded yet. Open Settings and download it, then start recording.",
                settings.whisper_model
            ));
        }
        Some(cfg)
    } else {
        if settings.server_token().is_none() {
            return Err(
                "Minutes server token is not configured. Set DESKSEC_TOKEN in .env or Settings before recording with online transcription.".into(),
            );
        }
        settings::validate_server_url(&settings.server_url())?;
        None
    };

    if let Some(ref warm_cfg) = cfg {
        let warm_cfg = warm_cfg.clone();
        tauri::async_runtime::spawn(async move {
            let _ = tauri::async_runtime::spawn_blocking(move || {
                local_transcribe::preload_whisper(&warm_cfg);
            })
            .await;
        });
    }

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<f32>>();

    // macOS ties the microphone grant to the bundle identity, so a rename or a
    // re-signed build silently drops it. CoreAudio then still opens the device
    // and still delivers buffers — all zeroes — which reads downstream as a
    // recording that simply transcribes to nothing. Record the real
    // authorization state here so that case is legible in the log instead of
    // being inferred from empty output an hour later.
    if settings.capture_microphone {
        let mic = crate::permissions::microphone_state();
        tracing::info!("microphone permission at record start: {mic:?}");
        if matches!(
            mic,
            crate::permissions::PermissionState::Denied
                | crate::permissions::PermissionState::NotDetermined
        ) {
            tracing::warn!(
                "microphone is not authorised; captured audio will be silent until access is granted"
            );
        }
    }

    let sources = capture_sources(&settings)?;

    // Capture notices can fire before the meeting row exists (during the initial
    // device open) and long after (on a mid-recording device swap), so the id is
    // shared through a slot the closure reads each time rather than captured.
    let notice_meeting: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let notify: audio::NoticeFn = {
        let app = app.clone();
        let slot = notice_meeting.clone();
        Arc::new(move |notice: audio::CaptureNotice| {
            let meeting_id = slot.lock_safe().clone();
            let _ = app.emit(
                EV_CAPTURE,
                json!({ "meetingId": meeting_id, "message": notice.message() }),
            );
        })
    };

    let capture: CaptureSession =
        audio::start_capture(sources, tx, notify).map_err(|e| e.to_string())?;
    // Every source resamples to this rate before it reaches us, which is what
    // lets a device be swapped mid-recording without the pipeline noticing.
    let sample_rate = audio::TARGET_SAMPLE_RATE;

    let title = title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(default_title);

    let meeting = {
        let conn = state.db.lock_safe();
        db::create_meeting(&conn, &title).map_err(|e| e.to_string())?
    };
    *notice_meeting.lock_safe() = Some(meeting.id.clone());

    // Matching on `cfg` rather than re-testing `is_whisper_engine` — the two were
    // the same predicate 75 lines apart, holding only because both happened to
    // read the same unchanged `settings`, with an `.expect()` as the penalty for
    // ever drifting. The Option already carries that decision.
    let processing = if let Some(whisper_cfg) = cfg {
        let http = state.http.clone();
        tauri::async_runtime::spawn(run_pipeline_chunk(
            app.clone(),
            state.db.clone(),
            meeting.id.clone(),
            rx,
            sample_rate,
            settings,
            whisper_cfg,
            http,
        ))
    } else {
        tauri::async_runtime::spawn(run_pipeline_live(
            app.clone(),
            state.db.clone(),
            meeting.id.clone(),
            rx,
            sample_rate,
            settings,
        ))
    };

    claim.commit(RecordingSession {
        meeting_id: meeting.id.clone(),
        capture: Some(capture),
        processing: Some(processing),
    });

    let _ = app.emit(
        EV_STATUS,
        json!({ "meetingId": meeting.id, "status": "recording" }),
    );

    Ok(meeting)
}

/// Stop the active recording, flush any buffered audio, and finalize the record.
pub async fn stop(app: &AppHandle) -> Result<String, String> {
    let (session, db) = {
        let state = app.state::<AppState>();
        let db = state.db.clone();
        let taken = state.session.lock_safe().take();
        (taken, db)
    };
    let mut session = session.ok_or_else(|| "no active recording".to_string())?;

    let meeting_id = session.meeting_id.clone();

    if let Some(cap) = session.capture.take() {
        cap.stop();
    }

    if let Some(handle) = session.processing.take() {
        let app_bg = app.clone();
        let meeting_id_bg = meeting_id.clone();
        tauri::async_runtime::spawn(async move {
            let _ = handle.await;
            {
                let conn = db.lock_safe();
                if let Err(e) =
                    db::finalize_meeting(&conn, &meeting_id_bg, MeetingStatus::Completed)
                {
                    tracing::error!("finalize meeting failed: {e}");
                }
            }
            crate::vault_export::export_meeting(&app_bg, &meeting_id_bg);
            let _ = app_bg.emit(
                EV_STATUS,
                json!({ "meetingId": meeting_id_bg, "status": "completed" }),
            );
        });
    } else {
        let state = app.state::<AppState>();
        let conn = state.db.lock_safe();
        db::finalize_meeting(&conn, &meeting_id, MeetingStatus::Completed)
            .map_err(|e| e.to_string())?;
        crate::vault_export::export_meeting(app, &meeting_id);
        let _ = app.emit(
            EV_STATUS,
            json!({ "meetingId": meeting_id, "status": "completed" }),
        );
    }

    Ok(meeting_id)
}

/// Which sources this recording should capture from.
///
/// The microphone is the base source and system audio is additive, so "record my
/// voice and the far side of the call" is both sources enabled — it is not
/// expressed by replacing the microphone with a loopback device.
fn capture_sources(settings: &Settings) -> Result<Vec<CaptureSource>, String> {
    let mut sources = Vec::new();
    if settings.capture_microphone {
        sources.push(CaptureSource::microphone(settings.input_device.clone()));
    }
    if settings.capture_system_audio {
        sources.push(CaptureSource::system_audio(
            settings.system_audio_device.clone(),
        ));
    }
    if sources.is_empty() {
        return Err(
            "No capture source is enabled. Open Settings and turn on the microphone, system audio, or both."
                .into(),
        );
    }
    Ok(sources)
}

fn default_title() -> String {
    let now = chrono::Local::now();
    format!("Meeting {}", now.format("%Y-%m-%d %H:%M"))
}

struct FinalizeJob {
    buffer: Vec<f32>,
    base_offset_secs: f64,
}

async fn run_pipeline_live(
    app: AppHandle,
    db: Arc<Mutex<Connection>>,
    meeting_id: String,
    rx: UnboundedReceiver<Vec<f32>>,
    sample_rate: u32,
    settings: Settings,
) {
    remote_stream::run_live_stream(app, db, meeting_id, rx, sample_rate, settings).await;
}

#[allow(clippy::too_many_arguments)]
async fn run_pipeline_chunk(
    app: AppHandle,
    db: Arc<Mutex<Connection>>,
    meeting_id: String,
    rx: UnboundedReceiver<Vec<f32>>,
    sample_rate: u32,
    settings: Settings,
    whisper_cfg: Config,
    http: reqwest::Client,
) {
    let whisper_cfg = Some(whisper_cfg);
    run_pipeline_chunk_inner(
        app,
        db,
        meeting_id,
        rx,
        sample_rate,
        settings,
        whisper_cfg,
        http,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn run_pipeline_chunk_inner(
    app: AppHandle,
    db: Arc<Mutex<Connection>>,
    meeting_id: String,
    mut rx: UnboundedReceiver<Vec<f32>>,
    sample_rate: u32,
    settings: Settings,
    whisper_cfg: Option<Config>,
    http: reqwest::Client,
) {
    let chunk_samples = ((settings.chunk_secs.max(1.0)) * sample_rate as f32) as usize;
    let partials_enabled = settings.partial_secs > 0.0;
    let partial_samples = if partials_enabled {
        ((settings.partial_secs.max(0.5)) * sample_rate as f32) as usize
    } else {
        usize::MAX
    };

    let (final_tx, mut final_rx) = tokio::sync::mpsc::unbounded_channel::<FinalizeJob>();

    let app_worker = app.clone();
    let db_worker = db.clone();
    let meeting_worker = meeting_id.clone();
    let settings_worker = settings.clone();
    let whisper_cfg_worker = whisper_cfg.clone();
    let http_worker = http.clone();
    let final_worker = tauri::async_runtime::spawn(async move {
        while let Some(job) = final_rx.recv().await {
            finalize_chunk(
                &app_worker,
                &db_worker,
                &meeting_worker,
                &settings_worker,
                whisper_cfg_worker.as_ref(),
                &http_worker,
                &job.buffer,
                sample_rate,
                job.base_offset_secs,
            )
            .await;
        }
    });

    let partial_gen = Arc::new(AtomicU64::new(0));

    let mut buffer: Vec<f32> = Vec::with_capacity(chunk_samples + sample_rate as usize);
    let mut samples_since_partial = 0usize;
    let mut processed_samples: u64 = 0;
    let mut last_level = std::time::Instant::now();

    while let Some(batch) = rx.recv().await {
        if last_level.elapsed() >= std::time::Duration::from_millis(120) {
            let _ = app.emit(
                EV_LEVEL,
                json!({ "meetingId": meeting_id, "level": audio::rms(&batch) }),
            );
            last_level = std::time::Instant::now();
        }
        samples_since_partial += batch.len();
        buffer.extend_from_slice(&batch);

        if buffer.len() >= chunk_samples {
            let base_offset = processed_samples as f64 / sample_rate as f64;
            let chunk_len = buffer.len() as u64;
            let _ = final_tx.send(FinalizeJob {
                buffer: std::mem::take(&mut buffer),
                base_offset_secs: base_offset,
            });
            buffer = Vec::with_capacity(chunk_samples + sample_rate as usize);
            processed_samples += chunk_len;
            samples_since_partial = 0;
        } else if partials_enabled && samples_since_partial >= partial_samples {
            spawn_partial(
                partial_gen.clone(),
                app.clone(),
                meeting_id.clone(),
                settings.clone(),
                whisper_cfg.clone(),
                http.clone(),
                buffer.clone(),
                sample_rate,
                processed_samples,
            );
            samples_since_partial = 0;
        }
    }

    if !buffer.is_empty() {
        let base_offset = processed_samples as f64 / sample_rate as f64;
        let _ = final_tx.send(FinalizeJob {
            buffer,
            base_offset_secs: base_offset,
        });
    }

    drop(final_tx);
    let _ = final_worker.await;
}

async fn transcribe_chunk(
    settings: &Settings,
    whisper_cfg: Option<&Config>,
    _http: &reqwest::Client,
    buffer: &[f32],
    sample_rate: u32,
    base_offset_secs: f64,
) -> Result<Vec<SpeakerLine>, crate::error::CategorizedError> {
    let cfg = whisper_cfg
        .ok_or_else(|| crate::error::CategorizedError::internal("whisper configuration missing"))?;
    // On-device transcription failures are diagnostic, not something a user can
    // act on, so they stay uncoded and convert through `From<String>`.
    local_transcribe::transcribe_samples(
        cfg,
        buffer,
        sample_rate,
        settings.diarization_enabled,
        base_offset_secs,
    )
    .await
    .map_err(crate::error::CategorizedError::from)
}

#[allow(clippy::too_many_arguments)]
fn spawn_partial(
    partial_gen: Arc<AtomicU64>,
    app: AppHandle,
    meeting_id: String,
    settings: Settings,
    whisper_cfg: Option<Config>,
    http: reqwest::Client,
    buffer: Vec<f32>,
    sample_rate: u32,
    processed_samples: u64,
) {
    if audio::is_mostly_silent(&buffer) {
        return;
    }
    let gen = partial_gen.fetch_add(1, Ordering::Relaxed) + 1;
    tauri::async_runtime::spawn(async move {
        let base_offset = processed_samples as f64 / sample_rate as f64;
        match transcribe_chunk(
            &settings,
            whisper_cfg.as_ref(),
            &http,
            &buffer,
            sample_rate,
            base_offset,
        )
        .await
        {
            Ok(lines) if !lines.is_empty() => {
                if partial_gen.load(Ordering::Acquire) != gen {
                    return;
                }
                let text = lines
                    .iter()
                    .map(|l| l.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !text.trim().is_empty() {
                    let _ = app.emit(EV_PARTIAL, json!({ "meetingId": meeting_id, "text": text }));
                }
            }
            _ => {}
        }
    });
}

#[allow(clippy::too_many_arguments)]
async fn finalize_chunk(
    app: &AppHandle,
    db: &Arc<Mutex<Connection>>,
    meeting_id: &str,
    settings: &Settings,
    whisper_cfg: Option<&Config>,
    http: &reqwest::Client,
    buffer: &[f32],
    sample_rate: u32,
    base_offset_secs: f64,
) {
    if audio::is_mostly_silent(buffer) {
        return;
    }

    let lines = match transcribe_chunk(
        settings,
        whisper_cfg,
        http,
        buffer,
        sample_rate,
        base_offset_secs,
    )
    .await
    {
        Ok(lines) => lines,
        Err(e) => {
            // `code` when the backend means the user to act on it, `message`
            // as the English fallback — same contract as command errors.
            let _ = app.emit(
                EV_ERROR,
                json!({
                    "meetingId": meeting_id,
                    "code": e.code,
                    "message": e.message,
                }),
            );
            return;
        }
    };

    for line in lines {
        if line.text.is_empty() {
            continue;
        }
        let segment = {
            let conn = db.lock_safe();
            db::insert_segment_full(
                &conn,
                meeting_id,
                &line.text,
                line.speaker_label.as_deref(),
                line.speaker_label.as_deref(),
                line.start_ms,
                line.end_ms,
            )
        };
        match segment {
            Ok(seg) => {
                let _ = app.emit(EV_FINAL, json!({ "meetingId": meeting_id, "segment": seg }));
            }
            Err(e) => {
                let _ = app.emit(
                    EV_ERROR,
                    json!({ "meetingId": meeting_id, "message": e.to_string() }),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an [`AppState`] backed by an in-memory database, mirroring the
    /// construction in `lib.rs` without needing a Tauri `AppHandle`.
    fn test_state() -> AppState {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        db::init_schema(&conn).expect("init schema");
        AppState {
            db: Arc::new(Mutex::new(conn)),
            http: reqwest::Client::new(),
            config_dir: std::env::temp_dir(),
            settings: Mutex::new(Settings::default()),
            session: Mutex::new(None),
            starting: std::sync::atomic::AtomicBool::new(false),
            prompt: crate::prompt_window::PromptState::default(),
        }
    }

    fn stub_session(meeting_id: &str) -> RecordingSession {
        RecordingSession {
            meeting_id: meeting_id.to_string(),
            capture: None,
            processing: None,
        }
    }

    /// The core double-recording regression test: many threads race through the
    /// same claim → (slow work) → create meeting row → commit sequence that
    /// `start()` performs. Exactly one may win, and exactly one meeting row may
    /// exist afterwards.
    #[test]
    fn concurrent_starts_yield_exactly_one_recording_and_one_meeting_row() {
        const ATTEMPTS: usize = 8;
        let state = Arc::new(test_state());
        let barrier = Arc::new(std::sync::Barrier::new(ATTEMPTS));

        let handles: Vec<_> = (0..ATTEMPTS)
            .map(|_| {
                let state = Arc::clone(&state);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    match state.try_claim_recording_slot() {
                        Ok(claim) => {
                            // Simulate the slow window in start() (device open,
                            // pipeline spawn) during which the old code let a
                            // second start slip through.
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            let meeting = {
                                let conn = state.db.lock_safe();
                                db::create_meeting(&conn, "racing meeting").expect("create meeting")
                            };
                            claim.commit(stub_session(&meeting.id));
                            true
                        }
                        Err(_) => false,
                    }
                })
            })
            .collect();

        let successes = handles
            .into_iter()
            .map(|h| h.join().expect("thread panicked"))
            .filter(|started| *started)
            .count();

        assert_eq!(successes, 1, "exactly one concurrent start may succeed");
        assert!(state.is_recording());

        let meetings = {
            let conn = state.db.lock_safe();
            db::list_meetings(&conn).expect("list meetings")
        };
        assert_eq!(meetings.len(), 1, "exactly one meeting row may be created");
        assert_eq!(
            state.recording_meeting_id().as_deref(),
            Some(meetings[0].meeting.id.as_str()),
            "the live session must point at the single created meeting"
        );
    }

    /// A start that fails partway (any early `?` return) must release the slot
    /// so the user can try again — a permanently stuck "starting" flag would be
    /// worse than the original bug.
    #[test]
    fn failed_start_releases_the_slot_for_the_next_attempt() {
        let state = test_state();

        let claim = state.try_claim_recording_slot().expect("first claim");
        assert!(state.is_recording(), "starting counts as recording");
        assert!(
            state.try_claim_recording_slot().is_err(),
            "second start must be rejected while the first is in flight"
        );
        // Simulates an early `?` return in start(): dropped without commit.
        drop(claim);

        assert!(!state.is_recording());
        assert!(
            state.try_claim_recording_slot().is_ok(),
            "slot must be free again after a failed start"
        );
    }

    /// An active session keeps blocking new starts, and stop (which takes the
    /// session) frees the slot.
    #[test]
    fn active_session_blocks_new_claims_until_stopped() {
        let state = test_state();

        let claim = state.try_claim_recording_slot().expect("claim");
        claim.commit(stub_session("m1"));

        assert!(state.is_recording());
        assert_eq!(state.recording_meeting_id().as_deref(), Some("m1"));
        assert!(state.try_claim_recording_slot().is_err());

        // What stop() does: take the session out of the slot.
        assert!(state.session.lock_safe().take().is_some());

        assert!(!state.is_recording());
        assert!(state.try_claim_recording_slot().is_ok());
    }
}
