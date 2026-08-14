//! Live online transcription via WebSocket (`/v1/transcribe/stream` → Deepgram Live).

use crate::db;
use crate::locking::MutexExt;
use crate::settings;
use futures_util::{SinkExt, StreamExt};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

const EV_PARTIAL: &str = "transcript-partial";
const EV_FINAL: &str = "transcript-final";
const EV_ERROR: &str = "transcript-error";
const EV_LEVEL: &str = "audio-level";
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

/// Stream captured audio to the Minutes server and emit partial/final transcript events.
pub async fn run_live_stream(
    app: AppHandle,
    db: Arc<Mutex<Connection>>,
    meeting_id: String,
    mut rx: UnboundedReceiver<Vec<f32>>,
    sample_rate: u32,
    settings: crate::settings::Settings,
) {
    let token = match settings.server_token() {
        Some(t) => t,
        None => {
            let _ = app.emit(
                EV_ERROR,
                json!({
                    "meetingId": meeting_id,
                    // Same contract as `CategorizedError::coded`: the UI
                    // translates `code` and keeps `message` as the fallback.
                    "code": "error.serverTokenMissing",
                    "message": "Minutes server token is not configured",
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

    let mut request = match url.into_client_request() {
        Ok(r) => r,
        Err(e) => {
            let _ = app.emit(
                EV_ERROR,
                json!({ "meetingId": meeting_id, "message": e.to_string() }),
            );
            return;
        }
    };
    request.headers_mut().insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))
            .unwrap_or_else(|_| HeaderValue::from_static("Bearer")),
    );

    // A wss:// handshake panics on the worker thread without a rustls provider,
    // which is silent from here — the transcript just never arrives.
    crate::install_tls_provider();

    let (ws_stream, _resp) = match connect_async(request).await {
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
                                let _ = app_read.emit(
                                    EV_PARTIAL,
                                    json!({ "meetingId": meeting_read, "text": t }),
                                );
                            }
                        }
                        "final" => {
                            if let Some(t) = ev.text {
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
    while let Some(batch) = rx.recv().await {
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
