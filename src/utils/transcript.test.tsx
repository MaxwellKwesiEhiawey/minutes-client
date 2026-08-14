import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import type { Segment } from "../types";
import {
  groupSegments,
  humanizeSpeakerLabel,
  mergeSegments,
  speakerDisplayName,
  SILENCE_SPLIT_MS,
} from "./transcript";
import { highlight } from "./highlight";

let nextId = 1;

/** Build a segment with sensible defaults; only override what a test cares about. */
function seg(overrides: Partial<Segment> = {}): Segment {
  const id = nextId++;
  return {
    id,
    meeting_id: "m1",
    seq: id,
    text: `text ${id}`,
    created_at: "2026-08-11T09:49:19Z",
    speaker_label: null,
    speaker_name: null,
    start_ms: null,
    end_ms: null,
    ...overrides,
  };
}

describe("humanizeSpeakerLabel", () => {
  it("maps zero-indexed raw labels to one-indexed friendly names", () => {
    expect(humanizeSpeakerLabel("SPEAKER_0")).toBe("Speaker 1");
    expect(humanizeSpeakerLabel("SPEAKER_1")).toBe("Speaker 2");
    expect(humanizeSpeakerLabel("SPEAKER_12")).toBe("Speaker 13");
  });

  it("returns null for missing or empty labels", () => {
    expect(humanizeSpeakerLabel(null)).toBeNull();
    expect(humanizeSpeakerLabel(undefined)).toBeNull();
    expect(humanizeSpeakerLabel("")).toBeNull();
  });

  it("leaves anything that is not a raw diarization label unchanged", () => {
    expect(humanizeSpeakerLabel("Alice")).toBe("Alice");
    expect(humanizeSpeakerLabel("SPEAKER_")).toBe("SPEAKER_");
    expect(humanizeSpeakerLabel("speaker_1")).toBe("speaker_1");
    expect(humanizeSpeakerLabel("SPEAKER_1B")).toBe("SPEAKER_1B");
  });

  it("tolerates surrounding whitespace", () => {
    expect(humanizeSpeakerLabel(" SPEAKER_0 ")).toBe("Speaker 1");
  });
});

describe("speakerDisplayName", () => {
  it("prefers the real human name when present", () => {
    expect(
      speakerDisplayName({ speaker_name: "Alice", speaker_label: "SPEAKER_0" }),
    ).toBe("Alice");
  });

  it("falls back to the humanized label", () => {
    expect(
      speakerDisplayName({ speaker_name: null, speaker_label: "SPEAKER_0" }),
    ).toBe("Speaker 1");
  });

  it("returns null when the segment has neither", () => {
    expect(
      speakerDisplayName({ speaker_name: null, speaker_label: null }),
    ).toBeNull();
  });
});

describe("groupSegments", () => {
  it("returns an empty list for no segments", () => {
    expect(groupSegments([])).toEqual([]);
  });

  it("keeps a single segment as a single group", () => {
    const s = seg({ text: "hello", speaker_label: "SPEAKER_0" });
    const groups = groupSegments([s]);
    expect(groups).toHaveLength(1);
    expect(groups[0]).toMatchObject({
      key: s.id,
      speaker: "Speaker 1",
      startedAt: s.created_at,
      text: "hello",
      segmentCount: 1,
    });
  });

  it("merges consecutive segments from the same speaker into one paragraph", () => {
    const groups = groupSegments([
      seg({ text: "awesome", speaker_label: "SPEAKER_1" }),
      seg({ text: "thank you one note on token usage", speaker_label: "SPEAKER_1" }),
      seg({ text: "you have here the projects", speaker_label: "SPEAKER_1" }),
    ]);
    expect(groups).toHaveLength(1);
    expect(groups[0].speaker).toBe("Speaker 2");
    expect(groups[0].text).toBe(
      "awesome thank you one note on token usage you have here the projects",
    );
    expect(groups[0].segmentCount).toBe(3);
  });

  it("starts a new group when the speaker changes", () => {
    const groups = groupSegments([
      seg({ text: "a", speaker_label: "SPEAKER_0" }),
      seg({ text: "b", speaker_label: "SPEAKER_0" }),
      seg({ text: "c", speaker_label: "SPEAKER_1" }),
      seg({ text: "d", speaker_label: "SPEAKER_0" }),
    ]);
    expect(groups.map((g) => [g.speaker, g.text])).toEqual([
      ["Speaker 1", "a b"],
      ["Speaker 2", "c"],
      ["Speaker 1", "d"],
    ]);
  });

  it("groups unlabeled segments together and keeps a null speaker", () => {
    const groups = groupSegments([seg({ text: "a" }), seg({ text: "b" })]);
    expect(groups).toHaveLength(1);
    expect(groups[0].speaker).toBeNull();
    expect(groups[0].text).toBe("a b");
  });

  it("keeps real names and does not merge a named speaker with a raw label", () => {
    const groups = groupSegments([
      seg({ text: "a", speaker_name: "Alice", speaker_label: "SPEAKER_0" }),
      seg({ text: "b", speaker_name: "Alice", speaker_label: "SPEAKER_0" }),
      seg({ text: "c", speaker_name: null, speaker_label: "SPEAKER_0" }),
    ]);
    expect(groups.map((g) => [g.speaker, g.text])).toEqual([
      ["Alice", "a b"],
      ["Speaker 1", "c"],
    ]);
  });

  it("splits on a silence gap longer than the threshold (audio offsets)", () => {
    const groups = groupSegments([
      seg({ text: "a", speaker_label: "SPEAKER_0", start_ms: 0, end_ms: 1000 }),
      seg({
        text: "b",
        speaker_label: "SPEAKER_0",
        start_ms: 1000 + SILENCE_SPLIT_MS + 1,
        end_ms: 1000 + SILENCE_SPLIT_MS + 2000,
      }),
    ]);
    expect(groups).toHaveLength(2);
  });

  it("splits on a silence gap using wall-clock time when offsets are missing", () => {
    const groups = groupSegments([
      seg({
        text: "a",
        speaker_label: "SPEAKER_0",
        created_at: "2026-08-11T09:49:19Z",
      }),
      seg({
        text: "b",
        speaker_label: "SPEAKER_0",
        created_at: "2026-08-11T09:50:19Z",
      }),
    ]);
    expect(groups).toHaveLength(2);
  });

  it("does not split on a short pause", () => {
    const groups = groupSegments([
      seg({ text: "a", speaker_label: "SPEAKER_0", start_ms: 0, end_ms: 1000 }),
      seg({ text: "b", speaker_label: "SPEAKER_0", start_ms: 5000, end_ms: 6000 }),
    ]);
    expect(groups).toHaveLength(1);
  });

  it("skips empty segment texts without adding stray spaces", () => {
    const groups = groupSegments([
      seg({ text: "a", speaker_label: "SPEAKER_0" }),
      seg({ text: "   ", speaker_label: "SPEAKER_0" }),
      seg({ text: "b", speaker_label: "SPEAKER_0" }),
    ]);
    expect(groups[0].text).toBe("a b");
    expect(groups[0].segmentCount).toBe(3);
  });

  it("uses the first segment's id and timestamp for the group", () => {
    const first = seg({
      text: "a",
      speaker_label: "SPEAKER_0",
      created_at: "2026-08-11T09:49:19Z",
    });
    const groups = groupSegments([
      first,
      seg({
        text: "b",
        speaker_label: "SPEAKER_0",
        created_at: "2026-08-11T09:49:24Z",
      }),
    ]);
    expect(groups[0].key).toBe(first.id);
    expect(groups[0].startedAt).toBe("2026-08-11T09:49:19Z");
  });
});

describe("search highlighting over grouped text", () => {
  it("marks matches inside a joined paragraph, including across the boundary between two segments", () => {
    const groups = groupSegments([
      seg({ text: "one note on token usage", speaker_label: "SPEAKER_1" }),
      seg({ text: "i just wanna show you guys", speaker_label: "SPEAKER_1" }),
    ]);
    expect(groups).toHaveLength(1);

    // "usage i" spans the join between the two original segments.
    const { container } = render(<>{highlight(groups[0].text, "usage i")}</>);
    const marks = container.querySelectorAll("mark");
    expect(marks).toHaveLength(1);
    expect(marks[0].textContent).toBe("usage i");
    expect(container.textContent).toBe(
      "one note on token usage i just wanna show you guys",
    );
  });

  it("marks every occurrence within the grouped text", () => {
    const groups = groupSegments([
      seg({ text: "budget review", speaker_label: "SPEAKER_0" }),
      seg({ text: "the budget is final", speaker_label: "SPEAKER_0" }),
    ]);
    const { container } = render(
      <>{highlight(groups[0].text, "budget")}</>,
    );
    expect(container.querySelectorAll("mark")).toHaveLength(2);
  });
});

describe("mergeSegments", () => {
  it("returns the other list when one side is empty", () => {
    const a = [seg({ id: 1, seq: 1 })];
    expect(mergeSegments(a, [])).toBe(a);
    expect(mergeSegments([], a)).toBe(a);
  });

  it("keeps one copy of a segment present in both lists", () => {
    const live = seg({ id: 5, seq: 5, text: "hello" });
    const fetched = seg({ id: 5, seq: 5, text: "hello" });
    expect(mergeSegments([live], [fetched])).toHaveLength(1);
  });

  it("lets the second list win, so a re-fetch can add a speaker label", () => {
    const live = seg({ id: 5, seq: 5, speaker_label: null });
    const fetched = seg({ id: 5, seq: 5, speaker_label: "SPEAKER_1" });
    const [merged] = mergeSegments([live], [fetched]);
    expect(merged.speaker_label).toBe("SPEAKER_1");
  });

  it("returns transcript order regardless of which side a segment came from", () => {
    const fetched = [seg({ id: 1, seq: 1 }), seg({ id: 3, seq: 3 })];
    const live = [seg({ id: 4, seq: 4 }), seg({ id: 2, seq: 2 })];
    expect(mergeSegments(fetched, live).map((s) => s.seq)).toEqual([1, 2, 3, 4]);
  });

  it("orders by id when two segments share a seq", () => {
    const a = [seg({ id: 9, seq: 1 })];
    const b = [seg({ id: 4, seq: 1 })];
    expect(mergeSegments(a, b).map((s) => s.id)).toEqual([4, 9]);
  });
});
