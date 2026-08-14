//! Shared export-rendering helpers.
//!
//! The Markdown export renderer ([`crate::commands`]), the vault-mirror
//! renderer ([`crate::vault_export`]) and the Word export
//! ([`crate::docx_export`]) format the same meeting data for different
//! consumers. Everything presentation-related that must stay consistent across
//! them — speaker labels, date/duration formatting, transcript grouping,
//! omit-when-empty owner/assignee notes, and the shared Markdown sections —
//! lives here so the renderers can't drift apart.

use crate::models::{KeyTopic, Segment};
use chrono::{DateTime, Local, TimeZone};
use std::fmt::Write;

// ---- Speaker labels -------------------------------------------------------

/// Resolve the display name for a segment's speaker.
///
/// A resolved human name always wins. Otherwise raw diarization labels like
/// `SPEAKER_0` are rendered as `Speaker 1` (zero-indexed to one-indexed) —
/// the same rule the on-screen transcript applies, so the file and the screen
/// agree. Unrecognized labels pass through untouched. Returns `None` when
/// neither a name nor a label is present.
pub fn display_speaker(speaker_name: Option<&str>, speaker_label: Option<&str>) -> Option<String> {
    if let Some(name) = non_empty(speaker_name) {
        return Some(name.to_string());
    }
    non_empty(speaker_label).map(humanize_speaker_label)
}

/// `SPEAKER_0` → `Speaker 1`. Mirrors the `^SPEAKER_(\d+)$` rule in
/// `humanizeSpeakerLabel` (`src/utils/transcript.ts`): only an all-digit
/// suffix is rewritten, so anything else — including a real name stored in
/// the label column — passes through untouched.
fn humanize_speaker_label(label: &str) -> String {
    let n = label
        .strip_prefix("SPEAKER_")
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|n| n.parse::<u32>().ok());
    match n {
        Some(n) => format!("Speaker {}", u64::from(n) + 1),
        None => label.to_string(),
    }
}

fn non_empty(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|s| !s.is_empty())
}

// ---- Meeting date / duration ----------------------------------------------

/// Human-readable meeting times for document headers.
pub struct MeetingTimes {
    /// e.g. `11 August 2026, 09:30 – 10:01` (end gets its own date when the
    /// meeting crosses midnight). Falls back to the raw stored string when the
    /// timestamp doesn't parse, so we never drop information.
    pub when: String,
    /// e.g. `31 min` / `1 h 5 min`. `None` when there is no (sane) end time.
    pub duration: Option<String>,
}

/// Format the stored RFC 3339 timestamps in the reader's local time zone.
pub fn meeting_times(created_at: &str, ended_at: Option<&str>) -> MeetingTimes {
    meeting_times_in(created_at, ended_at, &Local)
}

fn meeting_times_in<Tz: TimeZone>(created_at: &str, ended_at: Option<&str>, tz: &Tz) -> MeetingTimes
where
    Tz::Offset: std::fmt::Display,
{
    let Ok(start_raw) = DateTime::parse_from_rfc3339(created_at) else {
        return MeetingTimes {
            when: created_at.to_string(),
            duration: None,
        };
    };
    let end_raw = ended_at.and_then(|e| DateTime::parse_from_rfc3339(e).ok());

    let start = start_raw.with_timezone(tz);
    let mut when = start.format("%-d %B %Y, %H:%M").to_string();
    let mut duration = None;
    if let Some(end_raw) = end_raw {
        let end = end_raw.with_timezone(tz);
        if end.date_naive() == start.date_naive() {
            let _ = write!(when, " – {}", end.format("%H:%M"));
        } else {
            let _ = write!(when, " – {}", end.format("%-d %B %Y, %H:%M"));
        }
        let secs = (end_raw - start_raw).num_seconds();
        if secs >= 0 {
            duration = Some(format_duration(secs));
        }
    }
    MeetingTimes { when, duration }
}

fn format_duration(secs: i64) -> String {
    let mins = (secs + 30) / 60; // round to the nearest minute
    if mins < 1 {
        "under 1 min".to_string()
    } else if mins < 60 {
        format!("{mins} min")
    } else if mins % 60 == 0 {
        format!("{} h", mins / 60)
    } else {
        format!("{} h {} min", mins / 60, mins % 60)
    }
}

// ---- Owner / assignee notes (omitted when empty) ---------------------------

/// `owner: Dana` for a decision, or `None` when there is no owner — so
/// exports never print placeholder noise like `(owner: —)`.
pub fn owner_note(owner: Option<&str>) -> Option<String> {
    non_empty(owner).map(|o| format!("owner: {o}"))
}

/// `assignee: Ade, due: Friday` for an action item, with absent parts left
/// out entirely; `None` when neither is present.
pub fn assignment_note(assignee: Option<&str>, due: Option<&str>) -> Option<String> {
    let parts: Vec<String> = [
        non_empty(assignee).map(|a| format!("assignee: {a}")),
        non_empty(due).map(|d| format!("due: {d}")),
    ]
    .into_iter()
    .flatten()
    .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

// ---- Transcript grouping ---------------------------------------------------

/// Consecutive transcript segments from one speaker, merged into a single
/// flowing paragraph (raw diarization emits 3–9 word fragments per segment).
pub struct SpeakerGroup {
    /// Display speaker (see [`display_speaker`]); `None` when unknown.
    pub speaker: Option<String>,
    /// Start offset of the first segment in the group, in milliseconds.
    pub start_ms: Option<i64>,
    /// The segments' text joined with single spaces.
    pub text: String,
}

/// Silence between consecutive segments that starts a new paragraph, in
/// milliseconds. Mirrors `SILENCE_SPLIT_MS` in `src/utils/transcript.ts`.
pub const SILENCE_SPLIT_MS: i64 = 30_000;

/// Milliseconds of silence between two consecutive segments. Prefers the
/// precise audio offsets when both sides have them; falls back to wall-clock
/// timestamps; 0 (never split) when neither is usable.
fn gap_ms(prev: &Segment, next: &Segment) -> i64 {
    if let (Some(end), Some(start)) = (prev.end_ms, next.start_ms) {
        return start - end;
    }
    match (
        DateTime::parse_from_rfc3339(&prev.created_at),
        DateTime::parse_from_rfc3339(&next.created_at),
    ) {
        (Ok(p), Ok(n)) => (n - p).num_milliseconds(),
        _ => 0,
    }
}

/// Merge consecutive segments from one speaker into flowing paragraphs. A new
/// group starts when the speaker changes or after more than
/// [`SILENCE_SPLIT_MS`] of silence, so a long monologue still breaks at its
/// natural pauses instead of becoming one unreadable block. Empty segments are
/// skipped; each group keeps its first segment's start offset.
pub fn group_segments(segments: &[Segment]) -> Vec<SpeakerGroup> {
    let mut groups: Vec<SpeakerGroup> = Vec::new();
    let mut current_identity: Option<String> = None;
    let mut prev: Option<&Segment> = None;

    for seg in segments {
        let text = seg.text.trim();
        if text.is_empty() {
            continue;
        }
        // Group identity compares the raw stored values, not the rendered
        // label, so two different speakers can never merge through display
        // formatting — a person actually named "Speaker 1" must not absorb
        // SPEAKER_0. Same rule as `groupSegments` in transcript.ts.
        let identity = non_empty(seg.speaker_name.as_deref())
            .or(non_empty(seg.speaker_label.as_deref()))
            .map(str::to_string);
        let split_on_gap = prev.is_some_and(|p| gap_ms(p, seg) > SILENCE_SPLIT_MS);

        if groups.is_empty() || identity != current_identity || split_on_gap {
            groups.push(SpeakerGroup {
                speaker: display_speaker(seg.speaker_name.as_deref(), seg.speaker_label.as_deref()),
                start_ms: seg.start_ms,
                text: text.to_string(),
            });
            current_identity = identity;
        } else if let Some(last) = groups.last_mut() {
            last.text.push(' ');
            last.text.push_str(text);
        }
        prev = Some(seg);
    }
    groups
}

/// `m:ss` transcript offset, e.g. `3:05`.
pub fn fmt_ts(ms: i64) -> String {
    let total_secs = ms / 1000;
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{m}:{s:02}")
}

// ---- Shared Markdown sections ----------------------------------------------

/// Append the "Key Topics" section (nothing if there are no topics).
pub fn push_key_topics(out: &mut String, topics: &[KeyTopic]) {
    if topics.is_empty() {
        return;
    }
    out.push_str("### Key Topics\n\n");
    for t in topics {
        let _ = writeln!(out, "**{}**", t.topic);
        for b in &t.bullets {
            let _ = writeln!(out, "- {b}");
        }
        out.push('\n');
    }
}

/// Append the "Open Questions" section (nothing if there are none).
pub fn push_open_questions(out: &mut String, questions: &[String]) {
    if questions.is_empty() {
        return;
    }
    out.push_str("### Open Questions\n\n");
    for q in questions {
        let _ = writeln!(out, "- {q}");
    }
    out.push('\n');
}

/// Append the "Transcript" section: grouped by speaker, one timestamped
/// paragraph per group.
pub fn push_transcript(out: &mut String, segments: &[Segment]) {
    out.push_str("## Transcript\n\n");
    if segments.is_empty() {
        out.push_str("_No transcript captured._\n");
        return;
    }
    for g in group_segments(segments) {
        let ts = g.start_ms.map(fmt_ts);
        match (&g.speaker, ts) {
            (Some(sp), Some(ts)) => {
                let _ = writeln!(out, "**{sp}** [{ts}]: {}", g.text);
            }
            (Some(sp), None) => {
                let _ = writeln!(out, "**{sp}**: {}", g.text);
            }
            (None, Some(ts)) => {
                let _ = writeln!(out, "[{ts}] {}", g.text);
            }
            (None, None) => {
                let _ = writeln!(out, "{}", g.text);
            }
        }
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    fn seg(
        seq: i64,
        text: &str,
        label: Option<&str>,
        name: Option<&str>,
        start_ms: Option<i64>,
    ) -> Segment {
        Segment {
            id: seq,
            meeting_id: "m1".to_string(),
            seq,
            text: text.to_string(),
            created_at: "2026-08-11T09:30:00+00:00".to_string(),
            speaker_label: label.map(str::to_string),
            speaker_name: name.map(str::to_string),
            start_ms,
            end_ms: None,
        }
    }

    // ---- display_speaker ----

    #[test]
    fn diarization_labels_map_zero_indexed_to_one_indexed() {
        assert_eq!(
            display_speaker(None, Some("SPEAKER_0")),
            Some("Speaker 1".to_string())
        );
        assert_eq!(
            display_speaker(None, Some("SPEAKER_1")),
            Some("Speaker 2".to_string())
        );
        // Two-digit labels must not be truncated or mis-parsed.
        assert_eq!(
            display_speaker(None, Some("SPEAKER_10")),
            Some("Speaker 11".to_string())
        );
    }

    #[test]
    fn real_speaker_name_wins_over_label() {
        assert_eq!(
            display_speaker(Some("Dana K."), Some("SPEAKER_0")),
            Some("Dana K.".to_string())
        );
    }

    #[test]
    fn empty_or_missing_name_falls_back_to_label_then_none() {
        assert_eq!(
            display_speaker(Some("  "), Some("SPEAKER_2")),
            Some("Speaker 3".to_string())
        );
        assert_eq!(display_speaker(None, None), None);
        assert_eq!(display_speaker(Some(""), Some("")), None);
    }

    #[test]
    fn unrecognized_labels_pass_through_untouched() {
        assert_eq!(
            display_speaker(None, Some("GUEST_A")),
            Some("GUEST_A".to_string())
        );
        // Not a number after the prefix: leave it alone rather than guess.
        assert_eq!(
            display_speaker(None, Some("SPEAKER_X")),
            Some("SPEAKER_X".to_string())
        );
        // Only an all-digit suffix is rewritten, matching the `^SPEAKER_(\d+)$`
        // regex on the TypeScript side — a signed number is not a real label.
        assert_eq!(
            display_speaker(None, Some("SPEAKER_+5")),
            Some("SPEAKER_+5".to_string())
        );
        assert_eq!(
            display_speaker(None, Some("SPEAKER_")),
            Some("SPEAKER_".to_string())
        );
        // Zero-padded labels behave like their numeric value, as in TS.
        assert_eq!(
            display_speaker(None, Some("SPEAKER_007")),
            Some("Speaker 8".to_string())
        );
    }

    // ---- meeting_times ----

    #[test]
    fn meeting_times_formats_range_and_duration() {
        let t = meeting_times_in(
            "2026-08-11T09:30:26.403056+00:00",
            Some("2026-08-11T10:01:40.821324+00:00"),
            &Utc,
        );
        assert_eq!(t.when, "11 August 2026, 09:30 – 10:01");
        assert_eq!(t.duration.as_deref(), Some("31 min"));
    }

    #[test]
    fn meeting_times_converts_to_the_target_zone() {
        let plus_one = FixedOffset::east_opt(3600).unwrap();
        let t = meeting_times_in(
            "2026-08-11T09:30:00+00:00",
            Some("2026-08-11T10:01:00+00:00"),
            &plus_one,
        );
        assert_eq!(t.when, "11 August 2026, 10:30 – 11:01");
        assert_eq!(t.duration.as_deref(), Some("31 min"));
    }

    #[test]
    fn meeting_times_without_end_shows_start_only() {
        let t = meeting_times_in("2026-08-11T09:30:00+00:00", None, &Utc);
        assert_eq!(t.when, "11 August 2026, 09:30");
        assert_eq!(t.duration, None);
    }

    #[test]
    fn meeting_times_crossing_midnight_dates_both_ends() {
        let t = meeting_times_in(
            "2026-08-11T23:50:00+00:00",
            Some("2026-08-12T00:10:00+00:00"),
            &Utc,
        );
        assert_eq!(t.when, "11 August 2026, 23:50 – 12 August 2026, 00:10");
        assert_eq!(t.duration.as_deref(), Some("20 min"));
    }

    #[test]
    fn meeting_times_falls_back_to_raw_string_when_unparseable() {
        let t = meeting_times_in("not-a-date", Some("also-not-a-date"), &Utc);
        assert_eq!(t.when, "not-a-date");
        assert_eq!(t.duration, None);
    }

    #[test]
    fn meeting_times_omits_negative_duration() {
        // End before start (clock skew / bad data): show the range, skip the duration.
        let t = meeting_times_in(
            "2026-08-11T10:00:00+00:00",
            Some("2026-08-11T09:00:00+00:00"),
            &Utc,
        );
        assert_eq!(t.duration, None);
    }

    #[test]
    fn duration_rounds_and_scales_units() {
        assert_eq!(format_duration(20), "under 1 min");
        assert_eq!(format_duration(45), "1 min"); // rounds up
        assert_eq!(format_duration(31 * 60 + 14), "31 min"); // rounds down
        assert_eq!(format_duration(60 * 60), "1 h");
        assert_eq!(format_duration(65 * 60), "1 h 5 min");
    }

    // ---- owner / assignment notes ----

    #[test]
    fn owner_note_omitted_when_absent_or_blank() {
        assert_eq!(owner_note(None), None);
        assert_eq!(owner_note(Some("")), None);
        assert_eq!(owner_note(Some("  ")), None);
        assert_eq!(owner_note(Some("Dana")), Some("owner: Dana".to_string()));
    }

    #[test]
    fn assignment_note_includes_only_present_parts() {
        assert_eq!(assignment_note(None, None), None);
        assert_eq!(assignment_note(Some(""), Some(" ")), None);
        assert_eq!(
            assignment_note(Some("Ade"), None),
            Some("assignee: Ade".to_string())
        );
        assert_eq!(
            assignment_note(None, Some("Friday")),
            Some("due: Friday".to_string())
        );
        assert_eq!(
            assignment_note(Some("Ade"), Some("Friday")),
            Some("assignee: Ade, due: Friday".to_string())
        );
    }

    // ---- grouping ----

    #[test]
    fn consecutive_segments_from_one_speaker_merge() {
        let segs = vec![
            seg(1, "Good morning", Some("SPEAKER_0"), None, Some(0)),
            seg(
                2,
                "everyone, let's start.",
                Some("SPEAKER_0"),
                None,
                Some(2_000),
            ),
            seg(3, "Thanks.", Some("SPEAKER_1"), None, Some(5_000)),
            seg(4, "Right, so", Some("SPEAKER_0"), None, Some(8_000)),
        ];
        let groups = group_segments(&segs);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].speaker.as_deref(), Some("Speaker 1"));
        assert_eq!(groups[0].text, "Good morning everyone, let's start.");
        assert_eq!(groups[0].start_ms, Some(0)); // first segment's offset
        assert_eq!(groups[1].speaker.as_deref(), Some("Speaker 2"));
        assert_eq!(groups[2].speaker.as_deref(), Some("Speaker 1"));
        assert_eq!(groups[2].start_ms, Some(8_000));
    }

    #[test]
    fn grouping_uses_raw_identity_and_skips_empty_segments() {
        // The same person, however their name reached us, is one group.
        // Whitespace-only segments disappear entirely.
        let segs = vec![
            seg(1, "Hello", Some("SPEAKER_0"), Some("Dana"), Some(0)),
            seg(2, "   ", Some("SPEAKER_0"), Some("Dana"), Some(1_000)),
            seg(3, "world", None, Some("Dana"), Some(2_000)),
        ];
        let groups = group_segments(&segs);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].speaker.as_deref(), Some("Dana"));
        assert_eq!(groups[0].text, "Hello world");
    }

    #[test]
    fn identity_compares_raw_values_not_rendered_labels() {
        // A person literally named "Speaker 1" renders the same as SPEAKER_0,
        // but they are different people and must not merge.
        let segs = vec![
            seg(1, "one", Some("SPEAKER_0"), None, Some(0)),
            seg(2, "two", None, Some("Speaker 1"), Some(1_000)),
        ];
        let groups = group_segments(&segs);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].speaker.as_deref(), Some("Speaker 1"));
        assert_eq!(groups[1].speaker.as_deref(), Some("Speaker 1"));
    }

    #[test]
    fn long_silence_splits_one_speaker_into_separate_paragraphs() {
        // Same speaker either side, but a two-minute pause between them.
        let mut a = seg(1, "Before the break", Some("SPEAKER_0"), None, Some(0));
        a.end_ms = Some(5_000);
        let b = seg(2, "After the break", Some("SPEAKER_0"), None, Some(125_000));
        let groups = group_segments(&[a, b]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].text, "Before the break");
        assert_eq!(groups[1].text, "After the break");
        assert_eq!(groups[1].start_ms, Some(125_000));
    }

    #[test]
    fn short_silence_keeps_one_speaker_in_one_paragraph() {
        let mut a = seg(1, "A brief", Some("SPEAKER_0"), None, Some(0));
        a.end_ms = Some(5_000);
        // 10s pause: under the 30s threshold, so it stays one paragraph.
        let b = seg(2, "pause only", Some("SPEAKER_0"), None, Some(15_000));
        let groups = group_segments(&[a, b]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].text, "A brief pause only");
    }

    #[test]
    fn silence_split_falls_back_to_wall_clock_when_offsets_missing() {
        // No start_ms/end_ms at all: created_at carries the gap.
        let mut a = seg(1, "Before", Some("SPEAKER_0"), None, None);
        a.created_at = "2026-08-11T09:30:00+00:00".to_string();
        let mut b = seg(2, "After", Some("SPEAKER_0"), None, None);
        b.created_at = "2026-08-11T09:35:00+00:00".to_string();
        assert_eq!(group_segments(&[a, b]).len(), 2);
    }

    #[test]
    fn unparseable_timestamps_never_split() {
        let mut a = seg(1, "Before", Some("SPEAKER_0"), None, None);
        a.created_at = "not-a-date".to_string();
        let mut b = seg(2, "After", Some("SPEAKER_0"), None, None);
        b.created_at = "also-not-a-date".to_string();
        let groups = group_segments(&[a, b]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].text, "Before After");
    }

    #[test]
    fn unlabeled_segments_group_together() {
        let segs = vec![
            seg(1, "one", None, None, None),
            seg(2, "two", None, None, None),
        ];
        let groups = group_segments(&segs);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].speaker, None);
        assert_eq!(groups[0].text, "one two");
    }

    #[test]
    fn fmt_ts_pads_seconds_to_two_digits() {
        assert_eq!(fmt_ts(0), "0:00");
        assert_eq!(fmt_ts(5_000), "0:05");
        assert_eq!(fmt_ts(65_000), "1:05");
        assert_eq!(fmt_ts(3_600_000), "60:00");
    }

    // ---- markdown sections ----

    #[test]
    fn key_topics_render_matches_legacy_format() {
        let topics = vec![KeyTopic {
            topic: "Roadmap".to_string(),
            bullets: vec!["Q3 launch".to_string(), "hiring".to_string()],
        }];
        let mut out = String::new();
        push_key_topics(&mut out, &topics);
        assert_eq!(
            out,
            "### Key Topics\n\n**Roadmap**\n- Q3 launch\n- hiring\n\n"
        );
    }

    #[test]
    fn empty_sections_render_nothing() {
        let mut out = String::new();
        push_key_topics(&mut out, &[]);
        push_open_questions(&mut out, &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn open_questions_render_matches_legacy_format() {
        let mut out = String::new();
        push_open_questions(&mut out, &["Who owns billing?".to_string()]);
        assert_eq!(out, "### Open Questions\n\n- Who owns billing?\n\n");
    }

    #[test]
    fn transcript_groups_render_with_speaker_and_timestamp() {
        let segs = vec![
            seg(1, "Good morning", Some("SPEAKER_0"), None, Some(0)),
            seg(2, "everyone.", Some("SPEAKER_0"), None, Some(2_000)),
            seg(3, "Thanks.", Some("SPEAKER_1"), None, Some(65_000)),
        ];
        let mut out = String::new();
        push_transcript(&mut out, &segs);
        assert_eq!(
            out,
            "## Transcript\n\n\
             **Speaker 1** [0:00]: Good morning everyone.\n\n\
             **Speaker 2** [1:05]: Thanks.\n\n"
        );
    }

    #[test]
    fn transcript_without_segments_says_so() {
        let mut out = String::new();
        push_transcript(&mut out, &[]);
        assert_eq!(out, "## Transcript\n\n_No transcript captured._\n");
    }

    #[test]
    fn transcript_without_speaker_or_timestamp_degrades_gracefully() {
        let mut out = String::new();
        push_transcript(&mut out, &[seg(1, "hello", None, None, None)]);
        assert_eq!(out, "## Transcript\n\nhello\n\n");

        let mut out = String::new();
        push_transcript(&mut out, &[seg(1, "hello", None, None, Some(5_000))]);
        assert_eq!(out, "## Transcript\n\n[0:05] hello\n\n");

        let mut out = String::new();
        push_transcript(&mut out, &[seg(1, "hello", Some("SPEAKER_0"), None, None)]);
        assert_eq!(out, "## Transcript\n\n**Speaker 1**: hello\n\n");
    }
}
