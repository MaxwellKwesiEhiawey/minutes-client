//! A categorized, frontend-visible error type.
//!
//! Every other Tauri command in `commands.rs` returns `Result<T, String>` —
//! a bare string the frontend can only display verbatim. That's fine for
//! most commands, but `generate_summary` talks to an external HTTP server,
//! and "can't reach the server" / "bad token" / "server had a problem" call
//! for different user-facing copy and different retry affordances (see
//! `src/components/MeetingView.tsx`'s `summaryErrorCopy`, which switches on
//! this exact shape). Rather than reworking all 20 commands' shared
//! `CmdResult<T> = Result<T, String>` alias — a much larger, harder-to-verify
//! change — this type is scoped to just `generate_summary` /
//! `summary::summarize`.
//!
//! `CategorizedError` serializes to `{ "kind": "...", "message": "..." }`,
//! which Tauri delivers as the rejection value of the `invoke()` promise.
//! This matches the frontend's `AppError` shape (see `src/utils/errors.ts`)
//! field-for-field and case-for-case, so `normalizeError()` on the frontend
//! recognizes it as already-structured instead of falling back to treating
//! it as an opaque string.
//!
//! NOT verified with a local `cargo build` (see CONTRIBUTING.md) — please
//! run `cargo check` after pulling this change.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorKind {
    Network,
    Timeout,
    Auth,
    Server,
    Internal,
}

impl ErrorKind {
    /// Stable lowercase category name, matching both the serde serialization
    /// above and the frontend's `ErrorKind` union (`src/utils/errors.ts`).
    /// Telemetry sends **only** this category, never the message (messages
    /// can embed URLs, model names, or other environment details).
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::Network => "network",
            ErrorKind::Timeout => "timeout",
            ErrorKind::Auth => "auth",
            ErrorKind::Server => "server",
            ErrorKind::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CategorizedError {
    pub kind: ErrorKind,
    pub message: String,
    /// Stable translation key for a message the user is meant to read, e.g.
    /// `error.alreadyRecording`. The frontend translates it and ignores
    /// `message`; when it is `None`, `message` is shown as-is.
    ///
    /// This exists because the backend cannot know the UI language — it is not
    /// told, deliberately, since the language is a device-local preference — so
    /// anything a user is supposed to read has to travel as an identifier the UI
    /// can look up. `message` stays populated regardless, as the English
    /// fallback and as what ends up in logs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<&'static str>,
}

impl CategorizedError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
        }
    }

    /// A message the user reads: a translation key plus the English wording as
    /// the fallback and the log line.
    pub fn coded(code: &'static str, english: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            message: english.into(),
            code: Some(code),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }
}

/// Lets `generate_summary`'s existing `String`-returning helpers (e.g.
/// `settings::validate_server_url`, `db::list_segments(..).map_err(|e|
/// e.to_string())`) keep working with the `?` operator unchanged after the
/// command's return type moves to `CategorizedError` — they classify as
/// `Internal` since a plain string carries no kind information, which is a
/// strict improvement over today's completely uncategorized string.
impl From<String> for CategorizedError {
    fn from(message: String) -> Self {
        Self::internal(message)
    }
}

/// Same reasoning as the `String` impl, for the many `Err("literal")` sites.
/// An uncoded literal is diagnostic text, not something a user is asked to act
/// on; the ones users do read carry a code (see [`CategorizedError::coded`]).
impl From<&str> for CategorizedError {
    fn from(message: &str) -> Self {
        Self::internal(message.to_string())
    }
}

impl From<crate::summary::SummaryError> for CategorizedError {
    fn from(e: crate::summary::SummaryError) -> Self {
        use crate::summary::SummaryError;
        match e {
            SummaryError::Network(err) => {
                if err.is_timeout() {
                    CategorizedError::new(ErrorKind::Timeout, err.to_string())
                } else {
                    CategorizedError::new(ErrorKind::Network, err.to_string())
                }
            }
            SummaryError::Unauthorized => CategorizedError::new(
                ErrorKind::Auth,
                "The summarization server rejected the request (unauthorized). Check your Minutes access token in Settings.",
            ),
            SummaryError::Server(msg) => CategorizedError::new(ErrorKind::Server, msg),
        }
    }
}
