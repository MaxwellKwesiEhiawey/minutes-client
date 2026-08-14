//! Builds a PDF document for a meeting, with no external crate.
//!
//! All presentation rules shared with the Markdown and Word exports (speaker
//! labels, date/duration formatting, transcript grouping, omit-empty
//! owner/assignee notes) live in [`crate::markdown`] so the formats can't drift
//! apart.
//!
//! # Why hand-written
//!
//! A meeting export is styled text on A4 — no images, tables or vector art. That
//! is squarely inside what PDF's base feature set does with the 14 standard
//! fonts, which every reader is required to have, so nothing needs embedding and
//! nothing needs compressing. The alternative (`printpdf`) pulls roughly twenty
//! transitive crates including a font shaper and image codecs, and still leaves
//! line breaking to the caller because it exposes no metrics for the standard
//! fonts. For this document that is a large supply-chain cost for no capability
//! we use — see `.cargo/audit.toml` for the posture this repo takes on deps.
//!
//! Layout is therefore ours: [`WIDTHS`] holds the real Helvetica advance widths
//! from Adobe's AFM metrics, so [`wrap`] breaks lines exactly where the reader
//! will, rather than guessing from character counts.

use crate::markdown::{assignment_note, fmt_ts, group_segments, meeting_times, owner_note};
use crate::models::{Meeting, Segment, Summary};
use anyhow::Result;
use std::fmt::Write as _;

/* ---------- Page geometry, in PostScript points (72 per inch) ---------- */

const PAGE_W: f64 = 595.28; // A4
const PAGE_H: f64 = 841.89;
const MARGIN_X: f64 = 56.0;
const MARGIN_TOP: f64 = 64.0;
const MARGIN_BOTTOM: f64 = 56.0;
const CONTENT_W: f64 = PAGE_W - 2.0 * MARGIN_X;

const TITLE_SIZE: f64 = 20.0;
const H1_SIZE: f64 = 13.5;
const H2_SIZE: f64 = 11.5;
const BODY_SIZE: f64 = 10.5;
const META_SIZE: f64 = 9.0;

/// The app's accent (`--accent: #ff5a00`). Exports print on white paper.
const ACCENT: (f64, f64, f64) = (1.0, 0.353, 0.0);
const TEXT: (f64, f64, f64) = (0.031, 0.157, 0.231); // brand navy #08283B
const MUTED: (f64, f64, f64) = (0.42, 0.45, 0.47);

/// Font resource names declared in the page resource dictionary.
const F_REGULAR: &str = "F1";
const F_BOLD: &str = "F2";
const F_ITALIC: &str = "F3";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Font {
    Regular,
    Bold,
    Italic,
}

impl Font {
    fn resource(self) -> &'static str {
        match self {
            Font::Regular => F_REGULAR,
            Font::Bold => F_BOLD,
            Font::Italic => F_ITALIC,
        }
    }
}

/// Helvetica advance widths, in 1/1000 em, for the printable ASCII range
/// (space, 0x20, through tilde, 0x7E), taken from Adobe's Helvetica.afm. The
/// bold face differs, but only slightly, and using the regular widths for bold
/// text would under-measure headings and let them run into the margin — so the
/// bold table is carried too.
///
/// Helvetica-Oblique shares the regular widths, and any character outside the
/// range falls back to the width of `n`, which is close to the average.
#[rustfmt::skip]
const WIDTHS: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556,
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556,
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

#[rustfmt::skip]
const WIDTHS_BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611,
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556,
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611,
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

/// Width of `text` at `size`, in points.
fn text_width(text: &str, size: f64, font: Font) -> f64 {
    let table = if font == Font::Bold {
        &WIDTHS_BOLD
    } else {
        &WIDTHS
    };
    let fallback = table[('n' as usize) - 0x20];
    let mille: u32 = text
        .chars()
        .map(|c| {
            let i = c as usize;
            if (0x20..=0x7E).contains(&i) {
                u32::from(table[i - 0x20])
            } else {
                u32::from(fallback)
            }
        })
        .sum();
    f64::from(mille) * size / 1000.0
}

/// Greedy line breaking on whitespace. A single word wider than `max_w` is
/// emitted on its own line rather than dropped or truncated — a URL in a
/// transcript should overflow visibly, not disappear.
fn wrap(text: &str, size: f64, font: Font, max_w: f64) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if text_width(&candidate, size, font) <= max_w || line.is_empty() {
            line = candidate;
        } else {
            lines.push(std::mem::take(&mut line));
            line = word.to_string();
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Escape a string for a PDF literal string, and drop the characters a
/// WinAnsi-encoded standard font cannot represent.
///
/// Backslash and both parens would otherwise terminate or unbalance the string
/// and corrupt the file. Non-Latin-1 characters (emoji, CJK) are replaced with
/// `?`: the standard fonts have no glyphs for them, and a raw multi-byte
/// sequence would render as mojibake.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            // Curly quotes, dashes and the interpunct that our own copy uses:
            // map to ASCII rather than lose them.
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201C}' | '\u{201D}' => out.push('"'),
            '\u{2013}' | '\u{2014}' => out.push('-'),
            '\u{2022}' => out.push('-'),
            '\u{00B7}' => out.push('-'),
            '\u{2026}' => out.push_str("..."),
            c if (c as u32) < 0x20 => out.push(' '),
            c if (c as u32) <= 0x7E => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}

/// Accumulates content streams, breaking to a new page when the cursor runs out
/// of vertical room.
struct Pages {
    finished: Vec<String>,
    current: String,
    y: f64,
}

impl Pages {
    fn new() -> Self {
        Self {
            finished: Vec::new(),
            current: String::new(),
            y: PAGE_H - MARGIN_TOP,
        }
    }

    /// Reserve `height` points of vertical space, starting a page if needed.
    fn reserve(&mut self, height: f64) {
        if self.y - height < MARGIN_BOTTOM {
            self.page_break();
        }
        self.y -= height;
    }

    fn page_break(&mut self) {
        self.finished.push(std::mem::take(&mut self.current));
        self.y = PAGE_H - MARGIN_TOP;
    }

    fn space(&mut self, height: f64) {
        // Trailing space at a page boundary is dropped rather than pushing an
        // empty band onto the next page.
        if self.y - height >= MARGIN_BOTTOM {
            self.y -= height;
        }
    }

    /// Draw one already-wrapped line at the current cursor.
    fn line(&mut self, text: &str, size: f64, font: Font, color: (f64, f64, f64), x: f64) {
        let leading = size * 1.42;
        self.reserve(leading);
        let _ = writeln!(
            self.current,
            "BT /{} {:.2} Tf {:.3} {:.3} {:.3} rg 1 0 0 1 {:.2} {:.2} Tm ({}) Tj ET",
            font.resource(),
            size,
            color.0,
            color.1,
            color.2,
            x,
            self.y,
            escape(text),
        );
    }

    /// Wrap and draw a paragraph, optionally hanging-indented (for bullets).
    fn paragraph(
        &mut self,
        text: &str,
        size: f64,
        font: Font,
        color: (f64, f64, f64),
        indent: f64,
    ) {
        for line in wrap(text, size, font, CONTENT_W - indent) {
            self.line(&line, size, font, color, MARGIN_X + indent);
        }
    }

    fn heading(&mut self, text: &str) {
        self.space(10.0);
        self.paragraph(text, H1_SIZE, Font::Bold, ACCENT, 0.0);
        self.space(3.0);
    }

    fn subheading(&mut self, text: &str) {
        self.space(6.0);
        self.paragraph(text, H2_SIZE, Font::Bold, TEXT, 0.0);
    }

    fn body(&mut self, text: &str) {
        self.paragraph(text, BODY_SIZE, Font::Regular, TEXT, 0.0);
        self.space(4.0);
    }

    /// Bullet glyph in the margin, text hanging beside it.
    fn bullet(&mut self, text: &str) {
        let indent = 14.0;
        let lines = wrap(text, BODY_SIZE, Font::Regular, CONTENT_W - indent);
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                // Reserve first so the glyph and its first line can never be
                // split across a page boundary.
                let leading = BODY_SIZE * 1.42;
                self.reserve(leading);
                self.y += leading;
                self.line("-", BODY_SIZE, Font::Regular, ACCENT, MARGIN_X);
                let y = self.y;
                self.y += leading;
                self.line(line, BODY_SIZE, Font::Regular, TEXT, MARGIN_X + indent);
                self.y = y;
            } else {
                self.line(line, BODY_SIZE, Font::Regular, TEXT, MARGIN_X + indent);
            }
        }
        self.space(2.0);
    }

    fn into_streams(mut self) -> Vec<String> {
        self.finished.push(self.current);
        self.finished
    }
}

/// Decision line: the parenthetical is omitted entirely when there is no owner.
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

/// Metadata line under the title. Matches the Word export: "completed" is
/// internal bookkeeping a reader never needs, so only other statuses show.
fn meta_line(meeting: &Meeting) -> String {
    let times = meeting_times(&meeting.created_at, meeting.ended_at.as_deref());
    let mut meta = times.when;
    if let Some(d) = &times.duration {
        meta.push_str(&format!("  -  {d}"));
    }
    if meeting.status != "completed" {
        meta.push_str(&format!("  -  Status: {}", meeting.status));
    }
    meta
}

/// Lay the meeting out into one content stream per page.
fn build_streams(
    meeting: &Meeting,
    segments: &[Segment],
    summary: &Option<Summary>,
    include_transcript: bool,
) -> Vec<String> {
    let title = summary
        .as_ref()
        .map(|s| s.content.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| meeting.title.clone());

    let mut p = Pages::new();
    p.paragraph(&title, TITLE_SIZE, Font::Bold, TEXT, 0.0);
    p.space(2.0);
    p.paragraph(&meta_line(meeting), META_SIZE, Font::Regular, MUTED, 0.0);

    if let Some(s) = summary {
        let c = &s.content;

        p.heading("Summary");
        if !c.executive_summary.trim().is_empty() {
            p.body(&c.executive_summary);
        }

        if !c.key_topics.is_empty() {
            p.heading("Key Topics");
            for t in &c.key_topics {
                p.subheading(&t.topic);
                for b in &t.bullets {
                    p.bullet(b);
                }
            }
        }

        if !c.decisions.is_empty() {
            p.heading("Decisions");
            for d in &c.decisions {
                p.bullet(&decision_line(&d.text, d.owner.as_deref()));
            }
        }

        if !c.action_items.is_empty() {
            p.heading("Action Items");
            for a in &c.action_items {
                p.bullet(&action_item_line(
                    &a.task,
                    a.assignee.as_deref(),
                    a.due.as_deref(),
                ));
            }
        }

        if !c.open_questions.is_empty() {
            p.heading("Open Questions");
            for q in &c.open_questions {
                p.bullet(q);
            }
        }
    }

    if !include_transcript {
        return p.into_streams();
    }

    // Transcript starts on its own page, so the summary reads as a clean
    // standalone document — same rule as the Word export.
    p.page_break();
    p.heading("Transcript");
    if segments.is_empty() {
        p.paragraph(
            "No transcript captured.",
            BODY_SIZE,
            Font::Italic,
            MUTED,
            0.0,
        );
    } else {
        for g in group_segments(segments) {
            let ts = g.start_ms.map(fmt_ts);
            let prefix = match (g.speaker.as_deref(), ts.as_deref()) {
                (Some(sp), Some(ts)) => format!("{sp} [{ts}]: "),
                (Some(sp), None) => format!("{sp}: "),
                (None, Some(ts)) => format!("[{ts}] "),
                (None, None) => String::new(),
            };
            if prefix.is_empty() {
                p.body(&g.text);
            } else {
                // Speaker attribution in bold on its own line keeps long
                // paragraphs readable without measuring mixed-font runs.
                p.paragraph(prefix.trim_end(), BODY_SIZE, Font::Bold, TEXT, 0.0);
                p.paragraph(&g.text, BODY_SIZE, Font::Regular, TEXT, 0.0);
                p.space(5.0);
            }
        }
    }

    p.into_streams()
}

/// Serialize page content streams into a complete PDF 1.4 file.
///
/// Object layout: 1 = catalog, 2 = page tree, 3..5 = fonts, then a (page,
/// content) pair per page. The cross-reference table needs each object's exact
/// byte offset, so the body is assembled as bytes and offsets recorded as it
/// grows.
///
/// Assembling bytes rather than a `String` is load-bearing: the binary marker
/// below is four high Latin-1 bytes, which a Rust `String` holds as eight UTF-8
/// ones. Measuring offsets on the string and narrowing to Latin-1 afterwards
/// shifted every entry in the table by four bytes, and a reader that trusts the
/// table rejects the file. Object text is pure ASCII by construction (see
/// [`escape`]), so `as_bytes` on those is exact.
fn serialize(streams: &[String]) -> Vec<u8> {
    let font_obj = |n: usize, base: &str| {
        format!(
            "{n} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /{base} /Encoding /WinAnsiEncoding >>\nendobj\n"
        )
    };

    let page_count = streams.len();
    let first_page_obj = 6;
    let kids: Vec<String> = (0..page_count)
        .map(|i| format!("{} 0 R", first_page_obj + i * 2))
        .collect();

    let mut objects: Vec<String> = Vec::with_capacity(5 + page_count * 2);
    objects.push("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string());
    objects.push(format!(
        "2 0 obj\n<< /Type /Pages /Count {} /Kids [{}] >>\nendobj\n",
        page_count,
        kids.join(" ")
    ));
    objects.push(font_obj(3, "Helvetica"));
    objects.push(font_obj(4, "Helvetica-Bold"));
    objects.push(font_obj(5, "Helvetica-Oblique"));

    for (i, stream) in streams.iter().enumerate() {
        let page_obj = first_page_obj + i * 2;
        let content_obj = page_obj + 1;
        objects.push(format!(
            "{page_obj} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W:.2} {PAGE_H:.2}] \
             /Resources << /Font << /{F_REGULAR} 3 0 R /{F_BOLD} 4 0 R /{F_ITALIC} 5 0 R >> >> \
             /Contents {content_obj} 0 R >>\nendobj\n"
        ));
        objects.push(format!(
            "{content_obj} 0 obj\n<< /Length {} >>\nstream\n{stream}endstream\nendobj\n",
            stream.len()
        ));
    }

    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    // Binary marker: tells tools the file carries binary data so they do not
    // mangle it in text mode.
    out.extend_from_slice(&[b'%', 0xE2, 0xE3, 0xCF, 0xD3, b'\n']);

    let mut offsets = Vec::with_capacity(objects.len());
    for obj in &objects {
        offsets.push(out.len());
        out.extend_from_slice(obj.as_bytes());
    }

    let xref_at = out.len();
    let total = objects.len() + 1; // +1 for the free object 0
    let mut tail = String::new();
    let _ = write!(tail, "xref\n0 {total}\n0000000000 65535 f \n");
    for off in &offsets {
        let _ = writeln!(tail, "{off:010} 00000 n ");
    }
    let _ = write!(
        tail,
        "trailer\n<< /Size {total} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n"
    );
    out.extend_from_slice(tail.as_bytes());
    out
}

/// Render the meeting (summary + transcript) and write it to `path`.
pub fn write_pdf(
    path: &str,
    meeting: &Meeting,
    segments: &[Segment],
    summary: &Option<Summary>,
    include_transcript: bool,
) -> Result<()> {
    let bytes = serialize(&build_streams(
        meeting,
        segments,
        summary,
        include_transcript,
    ));
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting(status: &str) -> Meeting {
        Meeting {
            id: "m1".to_string(),
            title: "Weekly Sync".to_string(),
            status: status.to_string(),
            created_at: "2026-08-11T09:30:00+00:00".to_string(),
            ended_at: Some("2026-08-11T10:01:00+00:00".to_string()),
        }
    }

    fn segment(id: i64, text: &str, speaker: Option<&str>) -> Segment {
        Segment {
            id,
            meeting_id: "m1".to_string(),
            seq: id,
            text: text.to_string(),
            created_at: "2026-08-11T09:31:00+00:00".to_string(),
            speaker_label: None,
            speaker_name: speaker.map(str::to_string),
            start_ms: Some(id * 1000),
            end_ms: Some(id * 1000 + 500),
        }
    }

    #[test]
    fn escapes_the_characters_that_would_corrupt_a_pdf_string() {
        // Unescaped, each of these ends or unbalances the literal string and
        // the file stops being readable.
        assert_eq!(escape("a(b)c"), "a\\(b\\)c");
        assert_eq!(escape("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn maps_typographic_punctuation_our_own_copy_uses() {
        assert_eq!(
            escape("it\u{2019}s \u{201C}on\u{201D} \u{2013} now"),
            "it's \"on\" - now"
        );
        assert_eq!(escape("wait\u{2026}"), "wait...");
    }

    #[test]
    fn replaces_glyphs_the_standard_fonts_cannot_render() {
        // Emoji and CJK have no glyph in Helvetica; a raw byte run would render
        // as mojibake, so they become '?' rather than corrupting the page.
        assert_eq!(escape("hi \u{1F600} \u{4F60}\u{597D}"), "hi ? ??");
        // Control characters would break the stream.
        assert_eq!(escape("a\nb\tc"), "a b c");
    }

    #[test]
    fn wraps_on_real_font_metrics_and_never_exceeds_the_measure() {
        let text = "The quick brown fox jumps over the lazy dog again and again and again, \
                    and then keeps going well past the right margin of an A4 page so that \
                    line breaking actually has something to do.";
        let lines = wrap(text, BODY_SIZE, Font::Regular, CONTENT_W);
        assert!(lines.len() > 1, "expected the text to wrap: {lines:?}");
        for line in &lines {
            assert!(
                text_width(line, BODY_SIZE, Font::Regular) <= CONTENT_W,
                "line overflows the measure: {line:?}"
            );
        }
        // Nothing is lost or duplicated by wrapping.
        assert_eq!(lines.join(" "), text);
    }

    #[test]
    fn a_word_wider_than_the_measure_gets_its_own_line_instead_of_vanishing() {
        let long = "x".repeat(400);
        let lines = wrap(
            &format!("see {long} end"),
            BODY_SIZE,
            Font::Regular,
            CONTENT_W,
        );
        assert_eq!(lines, vec!["see", &long, "end"]);
    }

    #[test]
    fn bold_is_measured_with_bold_widths() {
        // Using the regular table for bold would under-measure and let headings
        // run past the right margin.
        let s = "Action Items and Decisions";
        assert!(text_width(s, H1_SIZE, Font::Bold) > text_width(s, H1_SIZE, Font::Regular));
    }

    #[test]
    fn meta_line_hides_completed_status_but_shows_interrupted() {
        assert!(!meta_line(&meeting("completed")).contains("Status:"));
        assert!(meta_line(&meeting("interrupted")).contains("Status: interrupted"));
    }

    #[test]
    fn omitting_the_transcript_drops_the_page_and_the_heading() {
        // The point of the switch is that the verbatim record is not in the
        // file at all — not merely that its heading is hidden.
        let streams = build_streams(
            &meeting("completed"),
            &[segment(
                1,
                "a sentence nobody outside the room should read",
                None,
            )],
            &None,
            false,
        );
        assert_eq!(streams.len(), 1, "no transcript page");
        let all = streams.join("");
        assert!(!all.contains("Transcript"), "transcript heading survived");
        assert!(
            !all.contains("nobody outside the room"),
            "transcript text survived: {all}"
        );
    }

    #[test]
    fn transcript_starts_on_its_own_page() {
        let streams = build_streams(
            &meeting("completed"),
            &[segment(1, "hello", Some("Ama"))],
            &None,
            true,
        );
        assert_eq!(streams.len(), 2, "summary page + transcript page");
        assert!(streams[1].contains("Transcript"));
    }

    #[test]
    fn long_transcripts_paginate() {
        // group_segments merges consecutive same-speaker segments, so alternate
        // speakers to get many groups rather than one long paragraph.
        let speakers = ["Ama", "Kwesi", "Naa"];
        let segments: Vec<Segment> = (1..=120)
            .map(|i| {
                segment(
                    i,
                    "A reasonably long line of transcript text to fill the page.",
                    Some(speakers[(i as usize) % 3]),
                )
            })
            .collect();
        let streams = build_streams(&meeting("completed"), &segments, &None, true);
        assert!(
            streams.len() > 3,
            "expected several pages, got {}",
            streams.len()
        );
    }

    #[test]
    fn writes_a_structurally_complete_pdf() {
        let streams = build_streams(
            &meeting("completed"),
            &[segment(1, "hello", None)],
            &None,
            true,
        );
        let bytes = serialize(&streams);

        assert!(bytes.starts_with(b"%PDF-1.4\n"), "missing header");
        assert!(bytes.ends_with(b"%%EOF\n"), "missing trailer");

        // Offsets must be compared against raw bytes: the binary marker is not
        // valid UTF-8, so any lossy string view shifts every index after it —
        // exactly the class of bug this test exists to catch.
        let find = |needle: &[u8]| {
            bytes
                .windows(needle.len())
                .position(|w| w == needle)
                .unwrap_or_else(|| panic!("missing {:?}", String::from_utf8_lossy(needle)))
        };
        find(b"/Type /Catalog");
        find(b"/BaseFont /Helvetica-Bold");
        find(format!("/Count {}", streams.len()).as_bytes());

        let startxref_at = bytes
            .windows(10)
            .rposition(|w| w == b"startxref\n")
            .expect("startxref");
        let after = &bytes[startxref_at + 10..];
        let digits: Vec<u8> = after
            .iter()
            .copied()
            .take_while(|b| b.is_ascii_digit())
            .collect();
        let xref_at: usize = String::from_utf8(digits).unwrap().parse().unwrap();
        assert_eq!(
            &bytes[xref_at..xref_at + 4],
            b"xref",
            "startxref misses the table"
        );

        // Every entry must point at the byte where its object actually starts.
        let table = &bytes[xref_at..];
        let table_text = String::from_utf8(table.to_vec()).expect("the table is ASCII");
        let entries: Vec<&str> = table_text
            .lines()
            .skip(3) // "xref", "0 N", then the free object 0 entry
            .take_while(|l| l.len() == 19 && l.ends_with(" n "))
            .collect();
        assert_eq!(entries.len(), streams.len() * 2 + 5, "one entry per object");
        for (i, entry) in entries.iter().enumerate() {
            let offset: usize = entry[..10].parse().expect("offset digits");
            let expected = format!("{} 0 obj", i + 1);
            assert!(
                bytes[offset..].starts_with(expected.as_bytes()),
                "object {} is indexed at {offset}, which is not {expected:?}",
                i + 1
            );
        }
    }
}

#[cfg(test)]
mod smoke {
    use super::*;

    /// Writes a real file so it can be opened by a PDF reader by hand. Ignored
    /// by default — `cargo test -- --ignored write_a_sample_pdf` and then open
    /// the path it prints.
    #[test]
    #[ignore = "writes a file for manual inspection"]
    fn write_a_sample_pdf() {
        use crate::models::{ActionItem, Decision, KeyTopic, Summary, SummaryContent};
        let meeting = Meeting {
            id: "m1".into(),
            title: "Q3 planning".into(),
            status: "completed".into(),
            created_at: "2026-08-11T09:30:00+00:00".into(),
            ended_at: Some("2026-08-11T10:14:00+00:00".into()),
        };
        let summary = Some(Summary {
            meeting_id: "m1".into(),
            model: "test".into(),
            created_at: "2026-08-11T10:20:00+00:00".into(),
            content: SummaryContent {
                title: "Q3 planning \u{2014} redesign and migration".into(),
                executive_summary:
                    "The team agreed to ship the redesign behind a flag in week two, \
                                    defer folders to Q4, and rehearse the migration on a copy of \
                                    production before touching the real database. Ama owns the \
                                    rollout comms; Kwesi owns the rehearsal."
                        .into(),
                key_topics: vec![KeyTopic {
                    topic: "Redesign rollout".into(),
                    bullets: vec![
                        "Ship behind a feature flag in week two".into(),
                        "Dogfood internally for five days before the wider release".into(),
                    ],
                }],
                decisions: vec![Decision {
                    text: "Ship the redesign behind a feature flag".into(),
                    owner: Some("Ama".into()),
                }],
                action_items: vec![ActionItem {
                    task: "Rehearse the migration on a production copy".into(),
                    assignee: Some("Kwesi".into()),
                    due: Some("Aug 20".into()),
                }],
                open_questions: vec!["Who owns on-call during the migration window?".into()],
            },
        });
        let segments: Vec<Segment> = (1..=40)
            .map(|i| Segment {
                id: i,
                meeting_id: "m1".into(),
                seq: i,
                text: "Let us start with the redesign \u{2014} I think we can ship it behind a flag \
                       in week two if nothing else lands on top of it, and the \u{201C}migration\u{201D} \
                       rehearsal has to happen first."
                    .into(),
                created_at: "2026-08-11T09:31:00+00:00".into(),
                speaker_label: None,
                speaker_name: Some(
                    ["Ama Boateng", "Kwesi Mensah", "Naa Adjei"][(i as usize) % 3].into(),
                ),
                start_ms: Some(i * 12_000),
                end_ms: Some(i * 12_000 + 8_000),
            })
            .collect();

        let path = std::env::temp_dir().join("minutes-sample.pdf");
        write_pdf(path.to_str().unwrap(), &meeting, &segments, &summary, true).unwrap();
        println!("wrote {}", path.display());
    }
}
