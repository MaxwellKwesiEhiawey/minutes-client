import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { RecordingScreen } from "./RecordingScreen";
import type { MeetingDetail, Segment } from "../types";

afterEach(cleanup);

function segment(id: number, text: string, speaker: string | null): Segment {
  return {
    id,
    meeting_id: "m1",
    seq: id,
    text,
    created_at: "2026-08-13T09:0" + id + ":00.000Z",
    speaker_label: null,
    speaker_name: speaker,
    start_ms: id * 1000,
    end_ms: id * 1000 + 900,
  };
}

function detail(segments: Segment[]): MeetingDetail {
  return {
    meeting: {
      id: "m1",
      title: "Standup",
      status: "recording",
      created_at: "2026-08-13T09:00:00.000Z",
      ended_at: null,
    },
    segments,
    summary: null,
  };
}

function props(overrides: Partial<Parameters<typeof RecordingScreen>[0]> = {}) {
  return {
    detail: detail([]),
    elapsed: "00:01:00",
    level: 0.2,
    partialText: "",
    engineMode: null,
    busy: false,
    onStop: vi.fn(),
    onBack: vi.fn(),
    ...overrides,
  };
}

describe("live transcript layout stability", () => {
  /* Interim text arrives and clears several times a second. Anything that
     appears, disappears or resizes as that happens moves every line under it,
     which is what made this screen twitch. */

  it("keeps the interim row mounted when there is no interim text", () => {
    const { container } = render(<RecordingScreen {...props()} />);
    expect(container.querySelector(".live-line.partial")).not.toBeNull();
  });

  it("keeps the same interim row when interim text arrives and clears", () => {
    const { container, rerender } = render(<RecordingScreen {...props()} />);
    const before = container.querySelectorAll(".live-line").length;

    rerender(<RecordingScreen {...props({ partialText: "and then we should" })} />);
    expect(container.querySelectorAll(".live-line").length).toBe(before);

    // Finalized: interim text clears, but no row is added or removed.
    rerender(
      <RecordingScreen
        {...props({
          partialText: "",
          detail: detail([segment(1, "and then we should ship", "Ama")]),
        })}
      />,
    );
    expect(container.querySelectorAll(".live-line").length).toBe(before + 1);
    expect(container.querySelector(".live-line.partial")).not.toBeNull();
  });

  it("scrolls the transcript in its own container, not the page", () => {
    const { container } = render(
      <RecordingScreen {...props({ detail: detail([segment(1, "hello", "Ama")]) })} />,
    );
    // The page scroller is `.main`, shared with the timer and waveform; the
    // transcript must own its own scroll area instead.
    const scroller = container.querySelector(".live-scroll");
    expect(scroller).not.toBeNull();
    expect(scroller!.querySelector(".live-list")).not.toBeNull();
  });

  it("attributes each speaker's block once, with stable initials", () => {
    const { container } = render(
      <RecordingScreen
        {...props({
          detail: detail([
            segment(1, "Numbers are up", "Ama Boateng"),
            segment(2, "Agreed", "Kwesi Mensah"),
          ]),
        })}
      />,
    );
    const avatars = [...container.querySelectorAll(".live-line:not(.partial) .live-avatar")];
    expect(avatars.map((a) => a.textContent)).toEqual(["AB", "KM"]);
  });
});
