#![allow(dead_code)]

//! Server-proxied batch transcription (fallback). Live recording uses [`crate::remote_stream`].
//!
//! Mirrors the local chunk API: encode PCM as WAV, POST to `/v1/transcribe`,
//! map JSON lines into [`SpeakerLine`]s with absolute meeting timings.

use crate::local_transcribe::SpeakerLine;
use crate::settings;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StatusResponse {
    configured: bool,
    model: String,
}

#[derive(Debug, Deserialize)]
struct LineDto {
    text: String,
    speaker_label: Option<String>,
    start_ms: Option<i64>,
    end_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TranscribeResponse {
    lines: Vec<LineDto>,
}

pub struct RemoteStatus {
    pub configured: bool,
    pub model: String,
}

fn http_base(server_url: &str) -> &str {
    server_url.trim().trim_end_matches('/')
}

fn apply_base_offset(lines: Vec<LineDto>, base_offset_secs: f64) -> Vec<SpeakerLine> {
    let base_ms = (base_offset_secs * 1000.0) as i64;
    lines
        .into_iter()
        .filter_map(|l| {
            let text = l.text.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(SpeakerLine {
                text,
                speaker_label: l.speaker_label,
                start_ms: l.start_ms.map(|ms| base_ms + ms),
                end_ms: l.end_ms.map(|ms| base_ms + ms),
            })
        })
        .collect()
}

/// Whether the server reports Deepgram as configured.
pub async fn fetch_status(
    client: &reqwest::Client,
    server_url: &str,
    token: &str,
) -> Result<RemoteStatus, crate::error::CategorizedError> {
    settings::validate_server_url(server_url)?;
    let url = format!("{}/v1/transcribe/status", http_base(server_url));
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("cannot reach transcription server: {e}"))?;

    if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
        return Err("Minutes server rejected the access token".into());
    }
    if !resp.status().is_success() {
        return Err(crate::error::CategorizedError::from(format!(
            "transcription status check failed ({})",
            resp.status()
        )));
    }

    let body: StatusResponse = resp
        .json()
        .await
        .map_err(|e| format!("invalid transcription status response: {e}"))?;
    Ok(RemoteStatus {
        configured: body.configured,
        model: body.model,
    })
}

/// Transcribe one audio chunk via the Minutes server → Deepgram.
#[allow(clippy::too_many_arguments)]
pub async fn transcribe_samples(
    client: &reqwest::Client,
    server_url: &str,
    token: &str,
    samples: &[f32],
    sample_rate: u32,
    diarization_enabled: bool,
    transcription_language: &str,
    base_offset_secs: f64,
) -> Result<Vec<SpeakerLine>, crate::error::CategorizedError> {
    settings::validate_server_url(server_url)?;

    let samples_16k =
        crate::audio::resample(samples, sample_rate, crate::audio::TARGET_SAMPLE_RATE);
    let wav = crate::audio::encode_wav_16k(&samples_16k, crate::audio::TARGET_SAMPLE_RATE)
        .map_err(|e| e.to_string())?;

    let mut url = reqwest::Url::parse(&format!("{}/v1/transcribe", http_base(server_url)))
        .map_err(|_| "invalid server URL".to_string())?;
    url.query_pairs_mut()
        .append_pair("diarize", &diarization_enabled.to_string());
    let lang = transcription_language.trim();
    if !lang.is_empty() && !lang.eq_ignore_ascii_case("auto") {
        url.query_pairs_mut().append_pair("language", lang);
    }

    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "audio/wav")
        .body(wav)
        .send()
        .await
        .map_err(|e| format!("transcription request failed: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        tracing::warn!("transcription request failed ({status})");
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(crate::error::CategorizedError::coded(
                "error.serverRejectedToken",
                "Minutes server rejected the access token",
            ));
        }
        if status.as_u16() == 503 {
            return Err(crate::error::CategorizedError::coded(
                "error.onlineNotConfiguredOnServer",
                "Online transcription is not configured on the Minutes server (DEEPGRAM_API_KEY).",
            ));
        }
        return Err(crate::error::CategorizedError::from(format!(
            "The transcription server returned an error ({status}). Please try again."
        )));
    }

    let parsed: TranscribeResponse = serde_json::from_str(&text).map_err(|e| {
        tracing::error!("could not parse transcription response: {e}");
        "The transcription server returned an unexpected response.".to_string()
    })?;

    Ok(apply_base_offset(parsed.lines, base_offset_secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_base_offset_shifts_timings() {
        let dto = vec![LineDto {
            text: "hello".into(),
            speaker_label: Some("SPEAKER_0".into()),
            start_ms: Some(100),
            end_ms: Some(500),
        }];
        let lines = apply_base_offset(dto, 10.0);
        assert_eq!(lines[0].start_ms, Some(10_100));
        assert_eq!(lines[0].end_ms, Some(10_500));
    }
}
