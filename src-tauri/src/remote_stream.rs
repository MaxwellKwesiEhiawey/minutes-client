//! Live online transcription via WebSocket (`/v1/transcribe/stream` → Deepgram Live).

use crate::db;
use crate::locking::MutexExt;
use crate::settings;
use futures_util::{SinkExt, StreamExt};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

const EV_PARTIAL: &str = "transcript-partial";
const EV_FINAL: &str = "transcript-final";
const EV_ERROR: &str = "transcript-error";
const EV_LEVEL: &str = "audio-level";

/// How much *voiced* audio may go by with nothing transcribed before the user
/// is told. Twenty seconds is long enough not to fire between "hello" and the
/// first result, short enough that nobody records a whole meeting into silence.
const NO_TRANSCRIPT_WARN_MS: u64 = 20_000;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::IntoClientRequest, http::header::AUTHORIZATION, http::HeaderValue, Message,
    },
};

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
    speaker_label: Option<String>,
    start_ms: Option<i64>,
    end_ms: Option<i64>,
    message: Option<String>,
}

pub fn stream_ws_url(server_url: &str, diarize: bool, language: &str) -> Result<String, String> {
    settings::validate_server_url(server_url)?;
    let base = server_url.trim().trim_end_matches('/');
    let ws_base = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        return Err(format!("unsupported server URL scheme: {base}"));
    };
    let mut url = reqwest::Url::parse(&format!("{ws_base}/v1/transcribe/stream"))
        .map_err(|_| "invalid server URL".to_string())?;
    url.query_pairs_mut()
        .append_pair("diarize", &diarize.to_string());
    let lang = language.trim();
    if !lang.is_empty() && !lang.eq_ignore_ascii_case("auto") {
        url.query_pairs_mut().append_pair("language", lang);
    }
    Ok(url.to_string())
}

fn f32_to_linear16_le(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i = (clamped * i16::MAX as f32) as i16;
        out.extend_from_slice(&i.to_le_bytes());
    }
    out
}

fn persist_final_line(
    app: &AppHandle,
    db: &Arc<Mutex<Connection>>,
    meeting_id: &str,
    text: &str,
    speaker_label: Option<String>,
    start_ms: Option<i64>,
    end_ms: Option<i64>,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    let segment = {
        let conn = db.lock_safe();
        db::insert_segment_full(
            &conn,
            meeting_id,
            text,
            speaker_label.as_deref(),
            speaker_label.as_deref(),
            start_ms,
            end_ms,
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

/// Decides when a live stream is silently producing nothing.
///
/// Deepgram returns *no messages at all* when the requested language does not
/// match the speech — English audio on a French model transcribes to nothing,
/// with no error anywhere. The app then looks broken rather than misconfigured,
/// which is exactly how an afternoon disappears. This watches for audio going
/// out with nothing coming back and says so once.
///
/// It counts voiced audio rather than elapsed time on purpose: someone who
/// starts recording and sits quietly for twenty seconds has nothing wrong with
/// their setup, and a warning they learn to dismiss is worse than none.
#[derive(Debug, Default)]
struct SilenceWatch {
    voiced_ms: u64,
    heard: bool,
    warned: bool,
}

impl SilenceWatch {
    /// Note that the server sent back a transcript; disarms the warning.
    fn heard_transcript(&mut self) {
        self.heard = true;
    }

    /// Feed one captured batch. Returns true exactly once, on the batch that
    /// pushes voiced audio past the threshold with nothing heard.
    fn observe(&mut self, batch_ms: u64, voiced: bool) -> bool {
        if self.heard || self.warned {
            return false;
        }
        if voiced {
            self.voiced_ms += batch_ms;
        }
        if self.voiced_ms >= NO_TRANSCRIPT_WARN_MS {
            self.warned = true;
            return true;
        }
        false
    }
}

/// Whether a failed handshake was rejected for authentication rather than, say,
/// an unreachable host — the only case worth re-registering for.
fn handshake_unauthorized<T>(result: &Result<T, tokio_tungstenite::tungstenite::Error>) -> bool {
    matches!(
        result,
        Err(tokio_tungstenite::tungstenite::Error::Http(resp))
            if resp.status() == 401
    )
}

/// Whether the server recognised this device and refused it.
///
/// Kept apart from [`handshake_unauthorized`] because that one triggers a
/// re-registration: a revoked device taking the same path would enrol itself
/// again and undo the revocation.
fn handshake_revoked<T>(result: &Result<T, tokio_tungstenite::tungstenite::Error>) -> bool {
    matches!(
        result,
        Err(tokio_tungstenite::tungstenite::Error::Http(resp))
            if resp.status() == 403
    )
}

/// Stream captured audio to the Minutes server and emit partial/final transcript events.
pub async fn run_live_stream(
    app: AppHandle,
    db: Arc<Mutex<Connection>>,
    meeting_id: String,
    mut rx: UnboundedReceiver<Vec<f32>>,
    sample_rate: u32,
    settings: crate::settings::Settings,
) {
    // Registering needs an HTTP client; this path is otherwise pure WebSocket.
    // Built here rather than reaching into AppState so `run_live_stream` keeps
    // taking a plain `Settings` and stays callable from the tests below.
    let http = reqwest::Client::new();
    let token = match crate::device::auth_token(&http, &settings).await {
        Ok(t) => t,
        Err(e) => {
            let _ = app.emit(
                EV_ERROR,
                json!({
                    "meetingId": meeting_id,
                    // Same contract as `CategorizedError::coded`: the UI
                    // translates `code` and keeps `message` as the fallback.
                    "code": e.code.unwrap_or("error.serverTokenMissing"),
                    "message": e.message,
                }),
            );
            return;
        }
    };

    let url = match stream_ws_url(
        &settings.server_url(),
        settings.diarization_enabled,
        &settings.transcription_language,
    ) {
        Ok(u) => u,
        Err(e) => {
            let _ = app.emit(EV_ERROR, json!({ "meetingId": meeting_id, "message": e }));
            return;
        }
    };

    // Errors are flattened to a String here rather than carried as a
    // tungstenite::Error: that enum is 136 bytes, and clippy's
    // `result_large_err` (denied in CI) rejects returning it by value.
    let build_request = |bearer: &str| -> Result<_, String> {
        let mut request = url
            .clone()
            .into_client_request()
            .map_err(|e| e.to_string())?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {bearer}"))
                .unwrap_or_else(|_| HeaderValue::from_static("Bearer")),
        );
        Ok(request)
    };

    let request = match build_request(&token) {
        Ok(r) => r,
        Err(e) => {
            let _ = app.emit(EV_ERROR, json!({ "meetingId": meeting_id, "message": e }));
            return;
        }
    };

    // A wss:// handshake panics on the worker thread without a rustls provider,
    // which is silent from here — the transcript just never arrives.
    crate::install_tls_provider();

    let mut attempt = connect_async(request).await;

    // Same reasoning as the summary path: a device token the server no longer
    // recognises (registry lost or restored) is recoverable by registering
    // again, so spend one retry on it rather than failing the recording.
    if handshake_unauthorized(&attempt) {
        tracing::warn!("device token rejected by live stream; re-registering");
        if let Ok(fresh) = crate::device::reregister(&http, &settings).await {
            if let Ok(retry) = build_request(&fresh) {
                attempt = connect_async(retry).await;
            }
        }
    }

    if handshake_revoked(&attempt) {
        tracing::warn!("live stream refused: this device has been revoked");
        let _ = app.emit(
            EV_ERROR,
            json!({
                "meetingId": meeting_id,
                "code": "error.deviceRevoked",
                "message": "This device's access has been revoked. Contact your IT team.",
            }),
        );
        return;
    }

    let (ws_stream, _resp) = match attempt {
        Ok(pair) => pair,
        Err(e) => {
            let _ = app.emit(
                EV_ERROR,
                json!({
                    "meetingId": meeting_id,
                    "message": format!("Could not open live transcription stream: {e}"),
                }),
            );
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Set by the read task, watched by the send loop below — the two run in
    // separate tasks, so this cannot be a plain flag on `SilenceWatch`.
    let heard = Arc::new(AtomicBool::new(false));
    let heard_read = Arc::clone(&heard);

    let app_read = app.clone();
    let db_read = db.clone();
    let meeting_read = meeting_id.clone();
    let read_task = tauri::async_runtime::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let Ok(ev) = serde_json::from_str::<StreamEvent>(&text) else {
                        continue;
                    };
                    match ev.kind.as_str() {
                        "partial" => {
                            if let Some(t) = ev.text.filter(|s| !s.trim().is_empty()) {
                                heard_read.store(true, Ordering::Relaxed);
                                let _ = app_read.emit(
                                    EV_PARTIAL,
                                    json!({ "meetingId": meeting_read, "text": t }),
                                );
                            }
                        }
                        "final" => {
                            if let Some(t) = ev.text {
                                heard_read.store(true, Ordering::Relaxed);
                                persist_final_line(
                                    &app_read,
                                    &db_read,
                                    &meeting_read,
                                    &t,
                                    ev.speaker_label,
                                    ev.start_ms,
                                    ev.end_ms,
                                );
                            }
                        }
                        "error" => {
                            let msg = ev
                                .message
                                .unwrap_or_else(|| "Transcription stream error".into());
                            let _ = app_read.emit(
                                EV_ERROR,
                                json!({ "meetingId": meeting_read, "message": msg }),
                            );
                        }
                        _ => {}
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    let _ = app_read.emit(
                        EV_ERROR,
                        json!({ "meetingId": meeting_read, "message": e.to_string() }),
                    );
                    break;
                }
                _ => {}
            }
        }
    });

    let mut last_level = std::time::Instant::now();
    let mut watch = SilenceWatch::default();
    while let Some(batch) = rx.recv().await {
        if heard.load(Ordering::Relaxed) {
            watch.heard_transcript();
        }
        let batch_ms = (batch.len() as u64 * 1000) / sample_rate.max(1) as u64;
        if watch.observe(batch_ms, !crate::audio::is_mostly_silent(&batch)) {
            let language = match settings.transcription_language.trim() {
                "" => "auto".to_string(),
                other => other.to_string(),
            };
            tracing::warn!(
                "no transcript after {NO_TRANSCRIPT_WARN_MS}ms of speech (language={language})"
            );
            let _ = app.emit(
                EV_ERROR,
                json!({
                    "meetingId": meeting_id,
                    "code": "error.noTranscriptCheckLanguage",
                    // Fallback for a build whose dictionary predates this code.
                    // Unlike the translated string it can name the language,
                    // because it is not looked up.
                    "message": format!(
                        "Audio is reaching the server but nothing is being transcribed. Check the transcription language (currently {language}) — it must match the language being spoken."
                    ),
                }),
            );
        }

        if last_level.elapsed() >= std::time::Duration::from_millis(120) {
            let _ = app.emit(
                EV_LEVEL,
                json!({ "meetingId": meeting_id, "level": crate::audio::rms(&batch) }),
            );
            last_level = std::time::Instant::now();
        }

        let pcm = crate::audio::resample(&batch, sample_rate, crate::audio::TARGET_SAMPLE_RATE);
        let bytes = f32_to_linear16_le(&pcm);
        if write.send(Message::Binary(bytes.into())).await.is_err() {
            break;
        }
    }

    let _ = write
        .send(Message::Text(r#"{"type":"close"}"#.into()))
        .await;
    let _ = write.close().await;
    let _ = read_task.await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// One second of audio per call keeps the arithmetic obvious.
    const SEC: u64 = 1000;

    #[test]
    fn silence_alone_never_warns() {
        let mut w = SilenceWatch::default();
        // A quiet room for well past the threshold is not a fault: nobody has
        // spoken yet, so there is nothing to transcribe.
        for _ in 0..120 {
            assert!(!w.observe(SEC, false));
        }
    }

    #[test]
    fn speech_with_nothing_coming_back_warns() {
        let mut w = SilenceWatch::default();
        let threshold_secs = NO_TRANSCRIPT_WARN_MS / SEC;
        for _ in 0..threshold_secs - 1 {
            assert!(!w.observe(SEC, true), "must not warn before the threshold");
        }
        assert!(
            w.observe(SEC, true),
            "should warn as the threshold is crossed"
        );
    }

    #[test]
    fn warns_only_once_per_stream() {
        let mut w = SilenceWatch::default();
        let mut warnings = 0;
        for _ in 0..300 {
            if w.observe(SEC, true) {
                warnings += 1;
            }
        }
        assert_eq!(warnings, 1, "a repeating toast is worse than no toast");
    }

    #[test]
    fn a_transcript_before_the_threshold_disarms_it() {
        let mut w = SilenceWatch::default();
        for _ in 0..5 {
            w.observe(SEC, true);
        }
        w.heard_transcript();
        for _ in 0..300 {
            assert!(
                !w.observe(SEC, true),
                "transcription is working; stay quiet"
            );
        }
    }

    #[test]
    fn a_transcript_after_a_warning_stops_further_ones() {
        let mut w = SilenceWatch::default();
        let mut warnings = 0;
        for _ in 0..NO_TRANSCRIPT_WARN_MS / SEC {
            if w.observe(SEC, true) {
                warnings += 1;
            }
        }
        assert_eq!(warnings, 1);
        // Late results — the language was fixed mid-recording, say.
        w.heard_transcript();
        for _ in 0..300 {
            assert!(!w.observe(SEC, true));
        }
    }

    #[test]
    fn interleaved_silence_does_not_count_toward_the_threshold() {
        let mut w = SilenceWatch::default();
        // Half the threshold in speech, then a long quiet stretch: the quiet
        // must not push it over, or a mostly-silent meeting warns spuriously.
        for _ in 0..NO_TRANSCRIPT_WARN_MS / SEC / 2 {
            assert!(!w.observe(SEC, true));
        }
        for _ in 0..600 {
            assert!(!w.observe(SEC, false));
        }
    }

    #[test]
    fn stream_ws_url_upgrades_scheme_and_carries_options() {
        assert_eq!(
            stream_ws_url("http://localhost:8787", true, "").unwrap(),
            "ws://localhost:8787/v1/transcribe/stream?diarize=true"
        );
        assert_eq!(
            stream_ws_url("https://api.example/", false, "es").unwrap(),
            "wss://api.example/v1/transcribe/stream?diarize=false&language=es"
        );
        // "auto" means let the provider decide, so it is not forwarded.
        assert!(!stream_ws_url("https://api.example", false, "auto")
            .unwrap()
            .contains("language"));
    }

    #[test]
    fn stream_ws_url_encodes_language_as_data() {
        assert_eq!(
            stream_ws_url("https://api.example", true, "en&diarize=false").unwrap(),
            "wss://api.example/v1/transcribe/stream?diarize=true&language=en%26diarize%3Dfalse"
        );
    }

    /// Only an unrecognised token may trigger a re-registration. A revoked
    /// device taking that path would enrol itself again and undo the
    /// revocation — the defect this pair of predicates exists to prevent.
    #[test]
    fn only_an_unknown_token_earns_a_retry() {
        use tokio_tungstenite::tungstenite::{
            http::{Response, StatusCode},
            Error,
        };

        // Returns the error rather than a Result: tungstenite::Error is 136
        // bytes, and clippy's `result_large_err` — denied in CI — rejects
        // handing one back by value.
        fn refusal(code: StatusCode) -> Error {
            Error::Http(Response::builder().status(code).body(None).unwrap())
        }

        let unknown: Result<(), Error> = Err(refusal(StatusCode::UNAUTHORIZED));
        assert!(handshake_unauthorized(&unknown), "401 means re-register");
        assert!(!handshake_revoked(&unknown));

        let revoked: Result<(), Error> = Err(refusal(StatusCode::FORBIDDEN));
        assert!(
            !handshake_unauthorized(&revoked),
            "403 must never trigger a re-registration"
        );
        assert!(handshake_revoked(&revoked));

        // A server that is simply broken is neither.
        let broken: Result<(), Error> = Err(refusal(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(!handshake_unauthorized(&broken));
        assert!(!handshake_revoked(&broken));

        let fine: Result<(), Error> = Ok(());
        assert!(!handshake_unauthorized(&fine));
        assert!(!handshake_revoked(&fine));
    }

    #[test]
    fn f32_to_linear16_le_is_little_endian_and_clamped() {
        assert_eq!(f32_to_linear16_le(&[0.0]), vec![0x00, 0x00]);
        // Above full scale must clamp to i16::MAX rather than wrap.
        assert_eq!(f32_to_linear16_le(&[2.0]), vec![0xFF, 0x7F]);
        assert_eq!(f32_to_linear16_le(&[-2.0]), vec![0x01, 0x80]);
    }

    /// Full online path, real dependencies: the production capture layer feeds
    /// system-audio loopback through the real resampler and linear16 encoder to
    /// the running Minutes server, which relays to Deepgram. A transcript coming
    /// back is the only proof that the captured audio is actually *intelligible*
    /// — sample counts alone would not catch a broken downmix or resample.
    ///
    /// Needs: `npm run dev` in desksec-server with `DEEPGRAM_API_KEY` set, a
    /// `DESKSEC_TOKEN` in the client `.env`, and speech playing on the default
    /// output device for the duration.
    #[tokio::test]
    #[ignore = "requires the server, a Deepgram key, and speech playing on the default output"]
    async fn system_audio_reaches_deepgram_as_intelligible_speech() {
        crate::settings::reload_env_keys();
        let settings = crate::settings::Settings::default();
        let token = settings
            .server_token()
            .expect("DESKSEC_TOKEN must be set for this test");
        let url = stream_ws_url(&settings.server_url(), false, "").expect("valid server url");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<f32>>();
        let notify: crate::audio::NoticeFn = Arc::new(|n: crate::audio::CaptureNotice| {
            println!("capture notice: {}", n.message());
        });
        let session = crate::audio::start_capture(
            vec![crate::audio::CaptureSource::system_audio(None)],
            tx,
            notify,
        )
        .expect("system audio capture should open");

        let mut request = url.into_client_request().expect("client request");
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("header"),
        );
        let (ws_stream, _) = connect_async(request)
            .await
            .expect("server should accept the stream");
        let (mut write, mut read) = ws_stream.split();

        let finals = Arc::new(Mutex::new(Vec::<String>::new()));
        let errors = Arc::new(Mutex::new(Vec::<String>::new()));
        let finals_read = finals.clone();
        let errors_read = errors.clone();
        let reader = tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                let Message::Text(text) = msg else { continue };
                let Ok(ev) = serde_json::from_str::<StreamEvent>(&text) else {
                    continue;
                };
                match ev.kind.as_str() {
                    "final" => {
                        if let Some(t) = ev.text.filter(|s| !s.trim().is_empty()) {
                            println!("final: {t}");
                            finals_read.lock().unwrap().push(t);
                        }
                    }
                    "error" => {
                        let m = ev.message.unwrap_or_default();
                        eprintln!("server error frame: {m}");
                        errors_read.lock().unwrap().push(m);
                    }
                    _ => {}
                }
            }
        });

        let mut sent_samples = 0usize;
        let deadline = Instant::now() + Duration::from_secs(22);
        while Instant::now() < deadline {
            if let Ok(Some(batch)) =
                tokio::time::timeout(Duration::from_millis(250), rx.recv()).await
            {
                sent_samples += batch.len();
                let bytes = f32_to_linear16_le(&batch);
                if write.send(Message::Binary(bytes.into())).await.is_err() {
                    break;
                }
            }
        }
        let _ = write
            .send(Message::Text(r#"{"type":"close"}"#.into()))
            .await;
        let _ = write.close().await;
        let _ = tokio::time::timeout(Duration::from_secs(15), reader).await;
        session.stop();

        let transcript = finals.lock().unwrap().join(" ");
        let errs = errors.lock().unwrap().clone();
        println!(
            "sent {sent_samples} samples ({:.1}s of 16 kHz audio)",
            sent_samples as f32 / crate::audio::TARGET_SAMPLE_RATE as f32
        );
        println!("TRANSCRIPT: {transcript:?}");
        assert!(errs.is_empty(), "server reported errors: {errs:?}");
        assert!(
            sent_samples > crate::audio::TARGET_SAMPLE_RATE as usize,
            "system audio delivered almost nothing — is anything playing? {sent_samples} samples"
        );
        assert!(
            !transcript.trim().is_empty(),
            "no transcript returned for {sent_samples} samples of audio"
        );
    }
}
