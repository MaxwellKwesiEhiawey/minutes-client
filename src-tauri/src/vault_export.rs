//! Markdown export bridge to `~/meetings/`.
//!
//! This app stores meetings canonically in SQLite, but the vendored Minutes
//! ecosystem (CLI, MCP server, relationship graph, vault sync — see `minutes/`)
//! reads plain markdown from the meetings directory. On each recording stop we
//! mirror the finished meeting out as a markdown file with YAML frontmatter so
//! those tools transparently see recordings made here.
//!
//! This is best-effort: any failure is logged and never blocks the recording
//! lifecycle. Controlled by the `export_markdown` setting (default on).

use crate::db;
use crate::locking::MutexExt;
use crate::models::{Meeting, Segment, Summary};
use crate::state::AppState;
use minutes_core::config::Config;
use tauri::{AppHandle, Manager};

/// Export a completed meeting to `~/meetings/<date>-<slug>.md`, if enabled.
pub fn export_meeting(app: &AppHandle, meeting_id: &str) {
    let state = app.state::<AppState>();

    let export_enabled = state.settings.lock_safe().export_markdown;
    if !export_enabled {
        return;
    }

    let (meeting, segments, summary) = {
        let conn = state.db.lock_safe();
        let Ok(Some(meeting)) = db::get_meeting(&conn, meeting_id) else {
            return;
        };
        let segments = db::list_segments(&conn, meeting_id).unwrap_or_default();
        let summary = db::get_summary(&conn, meeting_id).ok().flatten();
        (meeting, segments, summary)
    };

    // ~/meetings by default (Config::default().output_dir).
    let output_dir = Config::default().output_dir;
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        tracing::error!("could not create meetings dir {output_dir:?}: {e}");
        return;
    }

    let markdown = render_vault_markdown(&meeting, &segments, &summary);
    let path = output_dir.join(file_name(&meeting, &summary));
    if let Err(e) = std::fs::write(&path, markdown) {
        tracing::error!("failed to export meeting markdown to {path:?}: {e}");
    }
}

fn file_name(meeting: &Meeting, summary: &Option<Summary>) -> String {
    let date = meeting
        .created_at
        .split('T')
        .next()
        .unwrap_or(&meeting.created_at);
    let title = summary
        .as_ref()
        .map(|s| s.content.title.as_str())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(meeting.title.as_str());
    format!("{date}-{}.md", slugify(title))
}

fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut prev_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "meeting".to_string()
    } else {
        trimmed
    }
}

fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render markdown with YAML frontmatter compatible with the Minutes vault
/// reader (title, type, date, plus optional action_items / decisions).
fn render_vault_markdown(
    meeting: &Meeting,
    segments: &[Segment],
    summary: &Option<Summary>,
) -> String {
    let title = summary
        .as_ref()
        .map(|s| s.content.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| meeting.title.clone());

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("title: \"{}\"\n", yaml_escape(&title)));
    out.push_str("type: meeting\n");
    out.push_str(&format!("date: {}\n", meeting.created_at));
    if let Some(ended) = &meeting.ended_at {
        out.push_str(&format!("ended: {ended}\n"));
    }
    out.push_str("source: desksec\n");

    if let Some(s) = summary {
        let c = &s.content;
        if !c.action_items.is_empty() {
            out.push_str("action_items:\n");
            for a in &c.action_items {
                out.push_str(&format!("  - task: \"{}\"\n", yaml_escape(&a.task)));
                if let Some(assignee) = &a.assignee {
                    out.push_str(&format!("    assignee: \"{}\"\n", yaml_escape(assignee)));
                }
                if let Some(due) = &a.due {
                    out.push_str(&format!("    due: \"{}\"\n", yaml_escape(due)));
                }
                out.push_str("    status: open\n");
            }
        }
        if !c.decisions.is_empty() {
            out.push_str("decisions:\n");
            for d in &c.decisions {
                out.push_str(&format!("  - text: \"{}\"\n", yaml_escape(&d.text)));
                if let Some(owner) = &d.owner {
                    out.push_str(&format!("    owner: \"{}\"\n", yaml_escape(owner)));
                }
            }
        }
    }
    out.push_str("---\n\n");

    out.push_str(&format!("# {title}\n\n"));

    if let Some(s) = summary {
        let c = &s.content;
        out.push_str("## Summary\n\n");
        if !c.executive_summary.trim().is_empty() {
            out.push_str(&format!("{}\n\n", c.executive_summary));
        }
        crate::markdown::push_key_topics(&mut out, &c.key_topics);
        crate::markdown::push_open_questions(&mut out, &c.open_questions);
    }

    crate::markdown::push_transcript(&mut out, segments);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_and_dashes_non_alnum() {
        assert_eq!(
            slugify("Weekly Sync: Q3 Planning"),
            "weekly-sync-q3-planning"
        );
    }

    #[test]
    fn slugify_collapses_consecutive_separators_to_one_dash() {
        assert_eq!(slugify("a---b   c"), "a-b-c");
    }

    #[test]
    fn slugify_trims_leading_and_trailing_dashes() {
        assert_eq!(
            slugify("  --Leading and trailing--  "),
            "leading-and-trailing"
        );
    }

    #[test]
    fn slugify_falls_back_to_meeting_when_nothing_alphanumeric_remains() {
        // e.g. an emoji-only or punctuation-only meeting title, or an empty string —
        // mirrors the same footgun `sanitizeFilename` on the frontend guards against
        // (see src/utils/format.ts), so a meeting is never exported with an empty
        // or purely-dashes filename.
        assert_eq!(slugify(""), "meeting");
        assert_eq!(slugify("???"), "meeting");
        assert_eq!(slugify("🎉🎉🎉"), "meeting");
    }

    #[test]
    fn slugify_keeps_unicode_letters_intact_by_dropping_non_ascii_alnum() {
        // `is_ascii_alphanumeric` only keeps ASCII letters/digits, so accented
        // characters become separators rather than being preserved — documenting
        // this as current (possibly surprising) behavior rather than silently
        // relying on it.
        assert_eq!(slugify("Café Meeting"), "caf-meeting");
    }

    #[test]
    fn yaml_escape_escapes_backslashes_and_double_quotes() {
        assert_eq!(yaml_escape(r#"say "hi" \ bye"#), r#"say \"hi\" \\ bye"#);
        assert_eq!(yaml_escape("no special chars"), "no special chars");
    }
}
