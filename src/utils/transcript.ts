import type { Segment } from "../types";

// Raw diarization labels arrive from the backend as SPEAKER_0, SPEAKER_1, …
const RAW_SPEAKER_LABEL = /^SPEAKER_(\d+)$/;

/**
 * Turn a raw diarization label into a human-friendly one:
 * `SPEAKER_0` → `"Speaker 1"`, `SPEAKER_1` → `"Speaker 2"` (zero-indexed to
 * one-indexed). Anything that is not a raw `SPEAKER_n` label (e.g. a real
 * name that was stored in the label column) is returned unchanged; empty or
 * missing labels return null. Shared by the transcript view and the file
 * export paths so the display rule can't drift.
 */
export function humanizeSpeakerLabel(
  label: string | null | undefined,
): string | null {
  if (!label) return null;
  const m = RAW_SPEAKER_LABEL.exec(label.trim());
  if (!m) return label;
  return `Speaker ${Number(m[1]) + 1}`;
}

/**
 * Display name for a segment's speaker: a real human name (`speaker_name`)
 * always wins; otherwise the humanized diarization label; null when the
 * segment has neither.
 */
export function speakerDisplayName(
  s: Pick<Segment, "speaker_name" | "speaker_label">,
): string | null {
  return s.speaker_name ?? humanizeSpeakerLabel(s.speaker_label);
}

/**
 * Combine two segment lists into one transcript order, keeping a single copy of
 * any segment present in both.
 *
 * Needed because live `transcript-final` events and the `get_meeting` fetch are
 * two independent views of the same transcript that race each other: a segment
 * emitted while a fetch is in flight is in the event stream but not in the
 * response, and re-fetching after a meeting ends returns segments the UI already
 * appended. Merging on the database id keeps both without duplicates.
 */
export function mergeSegments(a: Segment[], b: Segment[]): Segment[] {
  if (b.length === 0) return a;
  if (a.length === 0) return b;
  const byId = new Map<number, Segment>();
  for (const s of a) byId.set(s.id, s);
  // Later wins: a re-fetched segment is at least as authoritative as the event
  // (it may carry a speaker label the live event did not have yet).
  for (const s of b) byId.set(s.id, s);
  return [...byId.values()].sort((x, y) => x.seq - y.seq || x.id - y.id);
}

/** Silence between consecutive segments that starts a new group. */
export const SILENCE_SPLIT_MS = 30_000;

/** One rendered transcript block: several consecutive segments by the same
 *  speaker, joined into flowing paragraph text. */
export interface TranscriptGroup {
  /** id of the first segment in the group — stable, usable as a React key. */
  key: number;
  /** Display-ready speaker (real name preserved, raw label humanized). */
  speaker: string | null;
  /** created_at of the first segment in the group. */
  startedAt: string;
  /** Segment texts joined with single spaces. */
  text: string;
  /** How many segments were merged into this group. */
  segmentCount: number;
}

/** Milliseconds of silence between two consecutive segments. Prefers the
 *  precise audio offsets when both sides have them; falls back to wall-clock
 *  timestamps; 0 (never split) when neither is usable. */
function gapMs(prev: Segment, next: Segment): number {
  if (prev.end_ms != null && next.start_ms != null) {
    return next.start_ms - prev.end_ms;
  }
  const gap = Date.parse(next.created_at) - Date.parse(prev.created_at);
  return Number.isNaN(gap) ? 0 : gap;
}

/**
 * Group consecutive segments that share a speaker into single paragraph
 * blocks. A new group starts when the speaker changes or after a silence
 * longer than `silenceSplitMs`. Segments must already be in transcript order.
 */
export function groupSegments(
  segments: Segment[],
  silenceSplitMs = SILENCE_SPLIT_MS,
): TranscriptGroup[] {
  const groups: TranscriptGroup[] = [];
  let current: TranscriptGroup | null = null;
  let currentIdentity: string | null = null;
  let prev: Segment | null = null;

  for (const s of segments) {
    // Group identity compares the raw values (name when present, else raw
    // label) so two different speakers can never merge via display formatting.
    const identity = s.speaker_name ?? s.speaker_label;
    const splitOnGap = prev !== null && gapMs(prev, s) > silenceSplitMs;

    if (!current || identity !== currentIdentity || splitOnGap) {
      current = {
        key: s.id,
        speaker: speakerDisplayName(s),
        startedAt: s.created_at,
        text: "",
        segmentCount: 0,
      };
      currentIdentity = identity;
      groups.push(current);
    }

    const text = s.text.trim();
    if (text) {
      current.text = current.text ? `${current.text} ${text}` : text;
    }
    current.segmentCount += 1;
    prev = s;
  }

  return groups;
}
