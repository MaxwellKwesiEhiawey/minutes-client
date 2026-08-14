import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import { api } from "./api";
import type { FinalEvent, MeetingDetail, Segment, StatusEvent, Meeting } from "./types";

/* The full recording lifecycle, driven through the same backend events the Rust
   side emits: meeting started → live segments → stop → status completed. This
   covers the reports that a recording could not be seen or stopped from another
   screen, and that no transcript showed up once the meeting had ended. */

const handlers = vi.hoisted(() => ({
  final: [] as Array<(e: FinalEvent) => void>,
  status: [] as Array<(e: StatusEvent) => void>,
  started: [] as Array<(m: Meeting) => void>,
}));

const state = vi.hoisted(() => ({
  detail: null as MeetingDetail | null,
  settings: {} as Record<string, unknown>,
}));
// The manual path adopts the meeting `start_recording` returns; the mock needs
// it before MEETING is defined, hence the indirection.
const MEETING_REF = vi.hoisted(() => ({ current: null as unknown as Meeting }));

vi.mock("./api", () => {
  const unlisten = () => {};
  const never = () => Promise.resolve(unlisten);
  return {
    api: {
      getSettings: vi.fn(() => Promise.resolve(state.settings)),
      generateSummary: vi.fn((meetingId: string) =>
        Promise.resolve({
          meeting_id: meetingId,
          model: "test-model",
          created_at: "2026-08-13T09:31:00.000Z",
          content: {
            title: "Standup — numbers and shipping",
            executive_summary: "Numbers are up; ship Thursday.",
            key_topics: [],
            decisions: [],
            action_items: [],
            open_questions: [],
          },
        }),
      ),
      listMeetings: vi.fn().mockResolvedValue([]),
      searchMeetings: vi.fn().mockResolvedValue([]),
      recordingState: vi.fn().mockResolvedValue(null),
      getMeeting: vi.fn(() => Promise.resolve(state.detail)),
      stopRecording: vi.fn().mockResolvedValue("m1"),
      transcriptionStatus: vi
        .fn()
        .mockResolvedValue({ model: "small", model_ready: true, diarization_enabled: false }),
      showNewMeetingPrompt: vi.fn().mockResolvedValue(undefined),
      startRecording: vi.fn(() => Promise.resolve(MEETING_REF.current)),
    },
    events: {
      onFinal: (cb: (e: FinalEvent) => void) => {
        handlers.final.push(cb);
        return Promise.resolve(unlisten);
      },
      onStatus: (cb: (e: StatusEvent) => void) => {
        handlers.status.push(cb);
        return Promise.resolve(unlisten);
      },
      onMeetingStarted: (cb: (m: Meeting) => void) => {
        handlers.started.push(cb);
        return Promise.resolve(unlisten);
      },
      onPartial: never,
      onError: never,
      onLevel: never,
      onCaptureNotice: never,
      onModelProgress: never,
    },
  };
});

vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn(), ask: vi.fn() }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));

const MEETING: Meeting = {
  id: "m1",
  title: "Standup",
  status: "recording",
  created_at: "2026-08-13T09:00:00.000Z",
  ended_at: null,
};

function segment(id: number, text: string): Segment {
  return {
    id,
    meeting_id: "m1",
    seq: id,
    text,
    created_at: "2026-08-13T09:01:00.000Z",
    speaker_label: "SPEAKER_0",
    speaker_name: null,
    start_ms: id * 1000,
    end_ms: id * 1000 + 900,
  };
}

MEETING_REF.current = MEETING;

const SETTINGS = {
  server_url: "",
  whisper_model: "small",
  transcription_engine: "whisper",
  diarization_enabled: false,
  export_markdown: false,
  anthropic_model: "",
  chunk_secs: 10,
  partial_secs: 1,
  capture_microphone: true,
  capture_system_audio: false,
  call_detection_supported: false,
  auto_summarize: true,
  server_token_present: true,
};

beforeEach(() => {
  vi.clearAllMocks();
  state.settings = { ...SETTINGS };
  handlers.final.length = 0;
  handlers.status.length = 0;
  handlers.started.length = 0;
  state.detail = { meeting: MEETING, segments: [], summary: null };
});

afterEach(cleanup);

/** Boot the app and put it into a live recording with one captured segment. */
async function startRecordingWithOneSegment() {
  render(<App />);
  await screen.findByRole("button", { name: /start a meeting/i });

  await act(async () => {
    handlers.started.forEach((h) => h(MEETING));
  });
  // The backend also has the segment, so a re-fetch keeps it.
  state.detail = { meeting: MEETING, segments: [segment(1, "Numbers are up this week")], summary: null };
  await act(async () => {
    handlers.final.forEach((h) => h({ meetingId: "m1", segment: segment(1, "Numbers are up this week") }));
  });
}

describe("recording lifecycle", () => {
  it("goes straight to the recording screen when New Meeting is pressed", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /start a meeting/i }));

    // `start_recording` emits no meeting-started event, so the UI has to adopt
    // the meeting it returns — otherwise recording runs with nothing on screen.
    await waitFor(() => expect(screen.getByText("Live transcript")).toBeTruthy());
    expect(screen.getByRole("button", { name: /end meeting/i })).toBeTruthy();
    expect(
      screen.getByRole("button", { name: /end the meeting being recorded/i }),
    ).toBeTruthy();
  });

  it("shows the recording screen with a live transcript once a meeting starts", async () => {
    await startRecordingWithOneSegment();

    expect(screen.getByText("Live transcript")).toBeTruthy();
    expect(screen.getByRole("button", { name: /end meeting/i })).toBeTruthy();
    expect(screen.getByText(/Numbers are up this week/)).toBeTruthy();
  });

  it("keeps the timer and Stop reachable after navigating away from the recording", async () => {
    await startRecordingWithOneSegment();

    fireEvent.click(screen.getByRole("button", { name: "Home" }));
    expect(screen.queryByText("Live transcript")).toBeNull();

    // This is the regression: the old always-visible record bar was replaced by
    // a screen, so a recording could be left running with no way to end it.
    const stop = screen.getByRole("button", { name: /end the meeting being recorded/i });
    expect(stop).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: /go to the meeting being recorded/i }));
    expect(screen.getByText("Live transcript")).toBeTruthy();
  });

  it("shows the transcript on the meeting screen after the meeting has ended", async () => {
    await startRecordingWithOneSegment();

    // Stopping ends capture; the backend then flushes and reports completion.
    fireEvent.click(screen.getByRole("button", { name: /end meeting/i }));
    await waitFor(() => expect(vi.mocked(api.stopRecording)).toHaveBeenCalled());

    const completed: Meeting = { ...MEETING, status: "completed", ended_at: "2026-08-13T09:30:00.000Z" };
    state.detail = {
      meeting: completed,
      segments: [segment(1, "Numbers are up this week"), segment(2, "Ship on Thursday")],
      summary: null,
    };
    await act(async () => {
      handlers.status.forEach((h) => h({ meetingId: "m1", status: "completed" }));
    });

    // No longer recording: the meeting screen and its tabs, not the rec screen.
    await waitFor(() => expect(screen.queryByText("Live transcript")).toBeNull());

    // The whole flushed transcript is there, including segments that only
    // arrived after Stop was pressed.
    fireEvent.click(screen.getByRole("tab", { name: "Transcription" }));
    expect(screen.getByText(/Numbers are up this week/)).toBeTruthy();
    expect(screen.getByText(/Ship on Thursday/)).toBeTruthy();

    // And the recording indicator is gone.
    expect(
      screen.queryByRole("button", { name: /end the meeting being recorded/i }),
    ).toBeNull();
  });

  it("does not drop segments that arrive before the meeting detail has loaded", async () => {
    // getMeeting is slow, so a final segment lands while detail is still null.
    let release: (v: MeetingDetail) => void = () => {};
    vi.mocked(api.getMeeting).mockImplementationOnce(
      () => new Promise<MeetingDetail>((r) => (release = r)),
    );

    render(<App />);
    await screen.findByRole("button", { name: /start a meeting/i });
    await act(async () => {
      handlers.started.forEach((h) => h(MEETING));
    });

    await act(async () => {
      handlers.final.forEach((h) => h({ meetingId: "m1", segment: segment(7, "Early words") }));
    });

    // The fetch that was in flight predates the segment above.
    await act(async () => {
      release({ meeting: MEETING, segments: [], summary: null });
    });

    expect(screen.getByText(/Early words/)).toBeTruthy();
  });
});

describe("automatic summary when a meeting ends", () => {
  /** Finish the meeting: stop, then the backend's completed status event. */
  async function endMeeting(overrides: Partial<Meeting> = {}, minutes = 30) {
    fireEvent.click(screen.getByRole("button", { name: /end meeting/i }));
    await waitFor(() => expect(vi.mocked(api.stopRecording)).toHaveBeenCalled());
    const ended = new Date(
      Date.parse(MEETING.created_at) + minutes * 60_000,
    ).toISOString();
    state.detail = {
      meeting: { ...MEETING, status: "completed", ended_at: ended, ...overrides },
      segments: [segment(1, "Numbers are up this week")],
      summary: null,
    };
    await act(async () => {
      handlers.status.forEach((h) => h({ meetingId: "m1", status: "completed" }));
    });
  }

  it("summarizes a finished meeting without being asked", async () => {
    await startRecordingWithOneSegment();
    await endMeeting();

    await waitFor(() =>
      expect(vi.mocked(api.generateSummary)).toHaveBeenCalledWith("m1", undefined),
    );
    // The generated summary lands on the open meeting, title included.
    expect(await screen.findByText(/Numbers are up; ship Thursday/)).toBeTruthy();
    expect(
      screen.getByRole("heading", { level: 2, name: "Standup — numbers and shipping" }),
    ).toBeTruthy();
  });

  it("does not summarize when the user has turned it off", async () => {
    state.settings = { ...SETTINGS, auto_summarize: false };
    await startRecordingWithOneSegment();
    await endMeeting();

    expect(vi.mocked(api.generateSummary)).not.toHaveBeenCalled();
    // The manual button is still the way to get one.
    expect(
      screen.getByRole("button", { name: /generate summary/i }),
    ).toBeTruthy();
  });

  it("skips meetings under a minute, so a stray recording costs nothing", async () => {
    await startRecordingWithOneSegment();
    await endMeeting({}, 0.5);

    expect(vi.mocked(api.generateSummary)).not.toHaveBeenCalled();
  });

  it("skips a meeting with no transcript", async () => {
    await startRecordingWithOneSegment();
    fireEvent.click(screen.getByRole("button", { name: /end meeting/i }));
    await waitFor(() => expect(vi.mocked(api.stopRecording)).toHaveBeenCalled());
    state.detail = {
      meeting: { ...MEETING, status: "completed", ended_at: "2026-08-13T09:40:00.000Z" },
      segments: [],
      summary: null,
    };
    await act(async () => {
      handlers.status.forEach((h) => h({ meetingId: "m1", status: "completed" }));
    });

    expect(vi.mocked(api.generateSummary)).not.toHaveBeenCalled();
  });

  it("stays quiet when the summarization server is not set up", async () => {
    // The manual path explains this and opens Settings; doing that on its own
    // after every meeting would be obnoxious.
    state.settings = { ...SETTINGS, server_token_present: false };
    await startRecordingWithOneSegment();
    await endMeeting();

    expect(vi.mocked(api.generateSummary)).not.toHaveBeenCalled();
    expect(screen.queryByText(/isn't set up yet/)).toBeNull();
  });

  it("summarizes a meeting only once, however often completion is reported", async () => {
    await startRecordingWithOneSegment();
    await endMeeting();
    await waitFor(() => expect(vi.mocked(api.generateSummary)).toHaveBeenCalledTimes(1));

    await act(async () => {
      handlers.status.forEach((h) => h({ meetingId: "m1", status: "completed" }));
    });
    expect(vi.mocked(api.generateSummary)).toHaveBeenCalledTimes(1);
  });
});

describe("revealing the automatic summary", () => {
  async function endLongMeeting() {
    fireEvent.click(screen.getByRole("button", { name: /end meeting/i }));
    await waitFor(() => expect(vi.mocked(api.stopRecording)).toHaveBeenCalled());
    state.detail = {
      meeting: {
        ...MEETING,
        status: "completed",
        ended_at: "2026-08-13T09:40:00.000Z",
      },
      segments: [segment(1, "Numbers are up this week")],
      summary: null,
    };
    await act(async () => {
      handlers.status.forEach((h) => h({ meetingId: "m1", status: "completed" }));
    });
  }

  it("opens the Summary tab when the summary arrives", async () => {
    await startRecordingWithOneSegment();
    await endLongMeeting();

    // The summary is what the user was waiting for, so it shows itself.
    await waitFor(() =>
      expect(
        screen.getByRole("tab", { name: "Summary" }).getAttribute("aria-selected"),
      ).toBe("true"),
    );
  });

  it("leaves the transcript alone if the user opened it themselves", async () => {
    await startRecordingWithOneSegment();

    fireEvent.click(screen.getByRole("button", { name: /end meeting/i }));
    await waitFor(() => expect(vi.mocked(api.stopRecording)).toHaveBeenCalled());

    // Reading the transcript while the summary is still being written.
    fireEvent.click(screen.getByRole("tab", { name: "Transcription" }));

    state.detail = {
      meeting: {
        ...MEETING,
        status: "completed",
        ended_at: "2026-08-13T09:40:00.000Z",
      },
      segments: [segment(1, "Numbers are up this week")],
      summary: null,
    };
    await act(async () => {
      handlers.status.forEach((h) => h({ meetingId: "m1", status: "completed" }));
    });
    await waitFor(() => expect(vi.mocked(api.generateSummary)).toHaveBeenCalled());

    // Still on the transcript: an arriving summary must not steal the view.
    expect(
      screen.getByRole("tab", { name: "Transcription" }).getAttribute("aria-selected"),
    ).toBe("true");
    expect(screen.getByText(/Numbers are up this week/)).toBeTruthy();
  });
});
