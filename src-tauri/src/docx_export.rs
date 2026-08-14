//! Builds a Word (.docx) document for a meeting using `docx-rs`.
//!
//! All presentation rules shared with the Markdown exports (speaker labels,
//! date/duration formatting, transcript grouping, omit-empty owner/assignee
//! notes) live in [`crate::markdown`] so the formats can't drift apart.

use crate::markdown::{assignment_note, fmt_ts, group_segments, meeting_times, owner_note};
use crate::models::{Meeting, Segment, Summary};
use anyhow::Result;
use docx_rs::*;

const TITLE_SZ: usize = 36; // half-points => 18pt
const H1_SZ: usize = 28; // 14pt
const H2_SZ: usize = 24; // 12pt
const META_SZ: usize = 18; // 9pt

// The app's accent colour (`--accent: #e65100`, the light-theme value in
// src/styles.css). Exports render on white paper, so the light-theme accent is
// the one with the right contrast; the previous "2E6BE6" blue matched nothing
// in the product.
const ACCENT: &str = "E65100";
const MUTED: &str = "777777";

fn heading(text: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(text).bold().size(H1_SZ).color(ACCENT))
        .style("Heading1")
}

fn heading2(text: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(text).bold().size(H2_SZ).color(ACCENT))
        .style("Heading2")
}

fn body(text: &str) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text))
}

fn bullet(text: &str) -> Paragraph {
    // Use a literal bullet glyph to avoid numbering-definition complexity.
    Paragraph::new()
        .add_run(Run::new().add_text(format!("\u{2022}  {text}")))
        .indent(Some(360), None, None, None)
}

/// Decision line: the parenthetical is omitted entirely when there is no
/// owner, instead of printing "(owner: —)".
fn decision_line(text: &str, owner: Option<&str>) -> String {
    match owner_note(owner) {
        Some(note) => format!("{text} ({note})"),
        None => text.to_string(),
    }
}

/// Action-item line: absent assignee/due parts are left out entirely.
fn action_item_line(task: &str, assignee: Option<&str>, due: Option<&str>) -> String {
    match assignment_note(assignee, due) {
        Some(note) => format!("{task} ({note})"),
        None => task.to_string(),
    }
}

/// The metadata line under the title: human date/time range in local time,
/// duration, and — only when the recording did not stop cleanly — the status
/// (e.g. "interrupted"). "completed" is internal bookkeeping a reader never
/// needs, so it is not shown.
fn meta_line(meeting: &Meeting) -> String {
    let times = meeting_times(&meeting.created_at, meeting.ended_at.as_deref());
    let mut meta = times.when;
    if let Some(d) = &times.duration {
        meta.push_str(&format!("  \u{b7}  {d}"));
    }
    if meeting.status != "completed" {
        meta.push_str(&format!("  \u{b7}  Status: {}", meeting.status));
    }
    meta
}

/// One transcript paragraph per speaker group: bold speaker, muted timestamp,
/// then the group's text flowing as a normal paragraph.
fn transcript_paragraph(speaker: Option<&str>, start_ms: Option<i64>, text: &str) -> Paragraph {
    // 6pt after each group so the transcript doesn't read as a wall.
    let mut p = Paragraph::new().line_spacing(LineSpacing::new().after(120));
    let ts = start_ms.map(fmt_ts);
    match (speaker, ts) {
        (Some(sp), Some(ts)) => {
            p = p
                .add_run(Run::new().add_text(sp).bold())
                .add_run(
                    Run::new()
                        .add_text(format!(" [{ts}]"))
                        .size(META_SZ)
                        .color(MUTED),
                )
                .add_run(Run::new().add_text(format!(": {text}")));
        }
        (Some(sp), None) => {
            p = p
                .add_run(Run::new().add_text(sp).bold())
                .add_run(Run::new().add_text(format!(": {text}")));
        }
        (None, Some(ts)) => {
            p = p
                .add_run(
                    Run::new()
                        .add_text(format!("[{ts}] "))
                        .size(META_SZ)
                        .color(MUTED),
                )
                .add_run(Run::new().add_text(text));
        }
        (None, None) => {
            p = p.add_run(Run::new().add_text(text));
        }
    }
    p
}

/// Render the meeting (summary + transcript) and write it to `path`.
pub fn write_docx(
    path: &str,
    meeting: &Meeting,
    segments: &[Segment],
    summary: &Option<Summary>,
    include_transcript: bool,
) -> Result<()> {
    let title = summary
        .as_ref()
        .map(|s| s.content.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| meeting.title.clone());

    // Define real heading styles with outline levels so Word's navigation
    // pane and a table of contents see the document structure.
    let mut docx = Docx::new()
        .add_style(
            Style::new("Heading1", StyleType::Paragraph)
                .name("Heading 1")
                .bold()
                .size(H1_SZ)
                .color(ACCENT)
                .outline_lvl(0),
        )
        .add_style(
            Style::new("Heading2", StyleType::Paragraph)
                .name("Heading 2")
                .bold()
                .size(H2_SZ)
                .color(ACCENT)
                .outline_lvl(1),
        )
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text(title).bold().size(TITLE_SZ)));

    // Metadata line.
    docx = docx.add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text(meta_line(meeting))
                .size(META_SZ)
                .color(MUTED),
        ),
    );

    if let Some(s) = summary {
        let c = &s.content;

        docx = docx.add_paragraph(heading("Summary"));
        if !c.executive_summary.trim().is_empty() {
            docx = docx.add_paragraph(body(&c.executive_summary));
        }

        if !c.key_topics.is_empty() {
            docx = docx.add_paragraph(heading("Key Topics"));
            for t in &c.key_topics {
                docx = docx.add_paragraph(heading2(&t.topic));
                for b in &t.bullets {
                    docx = docx.add_paragraph(bullet(b));
                }
            }
        }

        if !c.decisions.is_empty() {
            docx = docx.add_paragraph(heading("Decisions"));
            for d in &c.decisions {
                docx = docx.add_paragraph(bullet(&decision_line(&d.text, d.owner.as_deref())));
            }
        }

        if !c.action_items.is_empty() {
            docx = docx.add_paragraph(heading("Action Items"));
            for a in &c.action_items {
                docx = docx.add_paragraph(bullet(&action_item_line(
                    &a.task,
                    a.assignee.as_deref(),
                    a.due.as_deref(),
                )));
            }
        }

        if !c.open_questions.is_empty() {
            docx = docx.add_paragraph(heading("Open Questions"));
            for q in &c.open_questions {
                docx = docx.add_paragraph(bullet(q));
            }
        }
    }

    // Start the transcript on a fresh page so the summary reads as a clean
    // standalone document. Omitted entirely — heading included — when the user
    // chose to share the summary alone.
    if include_transcript {
        docx = docx.add_paragraph(heading("Transcript").page_break_before(true));
        if segments.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text("No transcript captured.").italic()),
            );
        } else {
            for g in group_segments(segments) {
                docx = docx.add_paragraph(transcript_paragraph(
                    g.speaker.as_deref(),
                    g.start_ms,
                    &g.text,
                ));
            }
        }
    }

    let file = std::fs::File::create(path)?;
    docx.build().pack(file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting(status: &str, created: &str, ended: Option<&str>) -> Meeting {
        Meeting {
            id: "m1".to_string(),
            title: "Weekly Sync".to_string(),
            status: status.to_string(),
            created_at: created.to_string(),
            ended_at: ended.map(str::to_string),
        }
    }

    #[test]
    fn decision_and_action_lines_omit_empty_placeholders() {
        assert_eq!(decision_line("Ship it", None), "Ship it");
        assert_eq!(
            decision_line("Ship it", Some("Dana")),
            "Ship it (owner: Dana)"
        );
        assert_eq!(action_item_line("Prepare deck", None, None), "Prepare deck");
        assert_eq!(
            action_item_line("Prepare deck", Some("Ade"), None),
            "Prepare deck (assignee: Ade)"
        );
        assert_eq!(
            action_item_line("Prepare deck", None, Some("Friday")),
            "Prepare deck (due: Friday)"
        );
        assert_eq!(
            action_item_line("Prepare deck", Some("Ade"), Some("Friday")),
            "Prepare deck (assignee: Ade, due: Friday)"
        );
    }

    /// Writing to a temp file is the only way to observe what landed in the
    /// document: `docx-rs` builds an opaque zip, so the assertion is on the
    /// bytes of the package rather than on a rendered string.
    #[test]
    fn omitting_the_transcript_keeps_it_out_of_the_document() {
        use crate::models::Segment;
        let secret = "a sentence nobody outside the room should read";
        let segments = vec![Segment {
            id: 1,
            meeting_id: "m1".into(),
            seq: 1,
            text: secret.into(),
            created_at: "2026-08-11T09:31:00+00:00".into(),
            speaker_label: None,
            speaker_name: None,
            start_ms: Some(1000),
            end_ms: Some(2000),
        }];
        let m = meeting("completed", "2026-08-11T09:30:00+00:00", None);

        let dir = std::env::temp_dir().join(format!("minutes-docx-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let with = dir.join("with.docx");
        let without = dir.join("without.docx");
        write_docx(with.to_str().unwrap(), &m, &segments, &None, true).unwrap();
        write_docx(without.to_str().unwrap(), &m, &segments, &None, false).unwrap();

        // Deflated text is not greppable, so compare sizes: the document
        // carrying a transcript must be the larger one, and both must be valid
        // non-empty packages.
        let big = std::fs::metadata(&with).unwrap().len();
        let small = std::fs::metadata(&without).unwrap().len();
        assert!(small > 0 && big > small, "with={big} without={small}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn meta_line_hides_completed_status_but_shows_interrupted() {
        // Note: `meeting_times` formats in the local time zone, so only assert
        // on the status suffix here; the date/duration formatting itself is
        // unit-tested in `crate::markdown` with fixed zones.
        let done = meta_line(&meeting(
            "completed",
            "2026-08-11T09:30:00+00:00",
            Some("2026-08-11T10:01:00+00:00"),
        ));
        assert!(!done.contains("Status:"), "unexpected status in: {done}");
        assert!(done.contains("31 min"), "missing duration in: {done}");

        let interrupted = meta_line(&meeting("interrupted", "2026-08-11T09:30:00+00:00", None));
        assert!(
            interrupted.contains("Status: interrupted"),
            "missing status in: {interrupted}"
        );
    }

    #[test]
    fn meta_line_falls_back_to_raw_timestamp_when_unparseable() {
        let m = meta_line(&meeting("completed", "not-a-date", None));
        assert_eq!(m, "not-a-date");
    }
}
