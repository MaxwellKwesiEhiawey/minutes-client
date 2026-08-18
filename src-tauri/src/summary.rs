//! AI summary client for the Minutes server. The server owns the Fireworks AI
//! key, prompt, and schema enforcement; the client just posts the transcript
//! and receives the structured `SummaryContent`.

use crate::models::SummaryContent;
use serde_json::json;

/// Categorized failure modes for a `summarize()` call, so the caller (see
/// `commands::generate_summary` / `crate::error::CategorizedError`) can tell
/// "can't reach the server" apart from "bad token" apart from "server had a
/// problem" instead of collapsing everything into one opaque string.
///
/// NOT verified with a local `cargo build` (see CONTRIBUTING.md).
#[derive(Debug, thiserror::Error)]
pub enum SummaryError {
    #[error("network error contacting the summarization server: {0}")]
    Network(#[from] reqwest::Error),
    #[error("unauthorized")]
    Unauthorized,
    /// The server recognised this device and refuses it. Distinct from
    /// [`Unauthorized`](SummaryError::Unauthorized) because the caller
    /// re-registers on that one, and a revoked device doing so would simply
    /// enrol itself again.
    #[error("device revoked")]
    DeviceRevoked,
    #[error("{0}")]
    Server(String),
}

fn http_base(server_url: &str) -> &str {
    server_url.trim().trim_end_matches('/')
}

/// Combine global (Settings) and per-meeting instructions into one optional payload.
/// Empty/whitespace values are ignored; if both are present they are concatenated.
pub fn merge_instructions(global: &str, per_meeting: Option<&str>) -> Option<String> {
    let global = global.trim();
    let per = per_meeting.map(str::trim).filter(|s| !s.is_empty());

    match (global.is_empty(), per) {
        (true, None) => None,
        (false, None) => Some(global.to_string()),
        (true, Some(p)) => Some(p.to_string()),
        (false, Some(p)) => Some(format!("{global}\n\n{p}")),
    }
}

/// Generate a structured summary by posting the transcript to the Minutes server.
///
/// `summary_language` is the active "Summary language" setting. It is forwarded
/// verbatim so the server can decide how to apply it (empty/`auto` => match the
/// transcript; a language name => force that language).
pub async fn summarize(
    client: &reqwest::Client,
    server_url: &str,
    token: &str,
    model: &str,
    transcript: &str,
    instructions: Option<&str>,
    summary_language: &str,
) -> Result<SummaryContent, SummaryError> {
    let url = format!("{}/v1/summarize", http_base(server_url));
    let mut body = json!({ "transcript": transcript, "model": model });
    if let Some(instr) = instructions.map(str::trim).filter(|s| !s.is_empty()) {
        body["instructions"] = json!(instr);
    }
    let lang = summary_language.trim();
    if !lang.is_empty() {
        body["language"] = json!(lang);
    }

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        // Response bodies may echo transcript or summary content. Keep them
        // out of persistent local logs and record only non-sensitive metadata.
        tracing::warn!("summary request failed ({status})");
        if status.as_u16() == 403 {
            return Err(SummaryError::DeviceRevoked);
        }
        if status.as_u16() == 401 {
            return Err(SummaryError::Unauthorized);
        }
        return Err(SummaryError::Server(format!(
            "The summarization server returned an error ({status}). Please try again, or contact IT if it persists."
        )));
    }

    serde_json::from_str(&text).map_err(|e| {
        tracing::error!("could not parse summary response: {e}");
        SummaryError::Server(
            "The summarization server returned an unexpected response. Please try again, or contact IT if it persists."
                .to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_base_trims_trailing_slashes_and_space() {
        assert_eq!(http_base("  https://x.test/  "), "https://x.test");
        assert_eq!(http_base("https://x.test"), "https://x.test");
    }

    #[test]
    fn merge_instructions_covers_all_combinations() {
        assert_eq!(merge_instructions("", None), None);
        assert_eq!(merge_instructions("  ", Some("  ")), None);
        assert_eq!(
            merge_instructions("global", None).as_deref(),
            Some("global")
        );
        assert_eq!(merge_instructions("", Some("per")).as_deref(), Some("per"));
        assert_eq!(
            merge_instructions("global", Some("per")).as_deref(),
            Some("global\n\nper")
        );
    }
}
