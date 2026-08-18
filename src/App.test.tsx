import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import App from "./App";
import { api } from "./api";

// Captured so tests can fire backend error events into the app.
const errorHandlers = vi.hoisted(
  () => [] as Array<(payload: { message: string }) => void>,
);

// App talks to the Tauri backend through ./api; give it a quiet fake so the
// UI can render (and start a meeting) inside jsdom.
vi.mock("./api", () => {
  const settings = {
    server_url: "",
    whisper_model: "small",
    transcription_engine: "whisper",
    diarization_enabled: false,
    export_markdown: false,
    anthropic_model: "",
    chunk_secs: 10,
    partial_secs: 1,
    capture_microphone: true,
    input_device: null,
    capture_system_audio: false,
    system_audio_device: null,
    summary_instructions: "",
    transcription_language: "auto",
    summary_language: "auto",
    call_detection_enabled: false,
    call_detection_cooldown_minutes: 5,
    call_detection_poll_interval_secs: 5,
    call_detection_apps: [],
    call_detection_supported: false,
    start_at_login: false,
    onboarding_completed_version: 1,
    server_url_from_env: false,
    server_url_from_build: false,
    server_token_present: false,
    server_token_from_env: false,
    server_token_from_build: false,
  };
  const unlisten = () => {};
  const never = () => Promise.resolve(unlisten);
  return {
    api: {
      getSettings: vi.fn().mockResolvedValue(settings),
      // Already onboarded: the default for every test that is not about setup.
      permissionStatus: vi.fn().mockResolvedValue({
        onboardingRequired: false,
        steps: [],
        completedVersion: 1,
        currentVersion: 1,
        microphone: "granted",
        browsers: [],
        platform: "macos",
      }),
      requestMicrophone: vi.fn().mockResolvedValue("granted"),
      requestBrowserAutomation: vi.fn().mockResolvedValue("granted"),
      openPrivacySettings: vi.fn().mockResolvedValue(undefined),
      completeOnboarding: vi.fn().mockResolvedValue(settings),
      resetOnboarding: vi.fn().mockResolvedValue({
        onboardingRequired: true,
        steps: ["microphone"],
        completedVersion: 0,
        currentVersion: 1,
        microphone: "notDetermined",
        browsers: [],
        platform: "macos",
      }),
      listMeetings: vi.fn().mockResolvedValue([]),
      searchMeetings: vi.fn().mockResolvedValue([]),
      recordingState: vi.fn().mockResolvedValue(null),
      transcriptionStatus: vi.fn().mockResolvedValue({
        model: "small",
        model_ready: true,
        diarization_enabled: false,
      }),
      showNewMeetingPrompt: vi.fn().mockResolvedValue(undefined),
      startRecording: vi.fn().mockResolvedValue({
        id: "m-new",
        title: "Meeting",
        status: "recording",
        created_at: "2026-08-13T09:00:00.000Z",
        ended_at: null,
      }),
      getMeeting: vi.fn().mockResolvedValue({
        meeting: {
          id: "m-new",
          title: "Meeting",
          status: "recording",
          created_at: "2026-08-13T09:00:00.000Z",
          ended_at: null,
        },
        segments: [],
        summary: null,
      }),
    },
    events: {
      onStatus: never,
      onPartial: never,
      onFinal: never,
      onError: (cb: (payload: { message: string }) => void) => {
        errorHandlers.push(cb);
        return Promise.resolve(unlisten);
      },
      onLevel: never,
      onCaptureNotice: never,
      onModelProgress: never,
      onMeetingStarted: never,
    },
  };
});

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
  ask: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  errorHandlers.length = 0;
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("empty-state call to action", () => {
  it("offers a primary Start a meeting button on the home screen", async () => {
    render(<App />);
    const cta = await screen.findByRole("button", { name: /start a meeting/i });
    expect(cta).toBeTruthy();
  });

  it("records immediately, without a confirm prompt, when the button is clicked", async () => {
    render(<App />);
    const cta = await screen.findByRole("button", { name: /start a meeting/i });
    fireEvent.click(cta);

    // Pressing New Meeting is already the instruction to record. Only a
    // system-detected call should raise the floating confirm prompt, so this
    // path must never open it.
    await waitFor(() => {
      expect(vi.mocked(api.startRecording)).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(api.showNewMeetingPrompt)).not.toHaveBeenCalled();
  });
});

describe("toast timing", () => {
  it("a new toast is not cut short by the previous toast's timer", async () => {
    vi.useFakeTimers();
    render(<App />);
    // Flush the initial async load so the error listener is registered.
    await act(async () => {});
    const fireError = (message: string) =>
      act(() => errorHandlers.forEach((h) => h({ message })));

    fireError("first");
    expect(screen.getByText("Transcription: first")).toBeTruthy();

    // 5s in, a second toast replaces the first. With the old code the
    // first toast's timer (due at t=8s) would dismiss it three seconds in.
    act(() => vi.advanceTimersByTime(5000));
    fireError("second");
    act(() => vi.advanceTimersByTime(4000)); // t = 9s, past the stale timer
    expect(screen.getByText("Transcription: second")).toBeTruthy();

    // The second toast's own 8s timer (due at t=13s) still dismisses it.
    act(() => vi.advanceTimersByTime(4000));
    expect(screen.queryByText("Transcription: second")).toBeNull();
  });
});

describe("home screen", () => {
  it("does not leak engine internals into the empty state", async () => {
    render(<App />);
    await screen.findByRole("button", { name: /start a meeting/i });
    // The engine mode is surfaced while recording (see engineLabel.test.ts);
    // the resting home screen should never show the model name.
    expect(screen.queryByText(/whisper/i)).toBeNull();
  });
});

describe("first-run setup gating", () => {
  it("shows the app, not the wizard, once onboarding is done", async () => {
    render(<App />);
    await screen.findByRole("button", { name: /start a meeting/i });
    expect(screen.queryByText(/welcome to minutes/i)).toBeNull();
  });

  it("replaces the whole shell with setup when it is due", async () => {
    vi.mocked(api.permissionStatus).mockResolvedValueOnce({
      onboardingRequired: true,
      steps: ["microphone"],
      completedVersion: 0,
      currentVersion: 1,
      microphone: "notDetermined",
      browsers: [],
      platform: "macos",
    });
    render(<App />);

    await screen.findByText(/welcome to minutes/i);
    // The nav rail and the home CTA must not be reachable behind it: setup is a
    // replacement for the shell, not an overlay on top of it.
    expect(screen.queryByRole("button", { name: /start a meeting/i })).toBeNull();
    expect(screen.queryByRole("navigation")).toBeNull();
  });

  it("opens the app when the permission probe fails", async () => {
    // A wizard that cannot be dismissed because its own status call broke would
    // lock the user out of meetings they already recorded.
    vi.mocked(api.permissionStatus).mockRejectedValueOnce(new Error("nope"));
    render(<App />);
    await screen.findByRole("button", { name: /start a meeting/i });
  });

  it("returns to the app after setup is finished", async () => {
    vi.mocked(api.permissionStatus).mockResolvedValueOnce({
      onboardingRequired: true,
      steps: [],
      completedVersion: 0,
      currentVersion: 1,
      microphone: "granted",
      browsers: [],
      platform: "macos",
    });
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /get started/i }));
    fireEvent.click(
      await screen.findByRole("button", { name: /start using minutes/i }),
    );

    await screen.findByRole("button", { name: /start a meeting/i });
    expect(api.completeOnboarding).toHaveBeenCalledTimes(1);
  });
});
