import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { SettingsScreen } from "./SettingsScreen";
import type { SettingsView, TranscriptionEngine } from "../types";

// The settings tree touches a spread of backend commands across its tabs, and
// this file is about one <select>. A proxy answers anything not named below
// with an empty resolved promise, so the mock does not need maintaining every
// time an unrelated tab gains a call.
vi.mock("../api", () => {
  const explicit: Record<string, unknown> = {
    transcriptionStatus: vi.fn().mockResolvedValue({
      model: "deepgram",
      model_ready: true,
      diarization_enabled: false,
    }),
    checkServer: vi.fn().mockResolvedValue({
      configured: true,
      reachable: true,
      message: "ok",
    }),
    listAudioDevices: vi.fn().mockResolvedValue({
      platform: "macos",
      devices: [],
      has_loopback: false,
    }),
    listInstalledModels: vi.fn().mockResolvedValue([]),
  };
  const unlisten = () => {};
  return {
    // Also exported from ../api and subscribed to by the tree.
    events: new Proxy({} as Record<string, unknown>, {
      get: (target, prop: string) =>
        prop in target
          ? target[prop]
          : (target[prop] = vi.fn().mockResolvedValue(unlisten)),
    }),
    api: new Proxy(explicit, {
      get: (target, prop: string) =>
        prop in target
          ? target[prop]
          : (target[prop] = vi.fn().mockResolvedValue(undefined)),
    }),
  };
});

afterEach(cleanup);

function settings(
  engine: TranscriptionEngine,
  language: string,
  overrides: Partial<SettingsView> = {},
): SettingsView {
  return {
    server_url: "https://example.test",
    whisper_model: "small",
    transcription_engine: engine,
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
    transcription_language: language,
    summary_language: "auto",
    auto_summarize: false,
    call_detection_enabled: false,
    call_detection_cooldown_minutes: 5,
    call_detection_poll_interval_secs: 5,
    call_detection_apps: [],
    call_detection_supported: false,
    share_supported: false,
    telemetry_enabled: false,
    onboarding_completed_version: 1,
    server_url_from_env: false,
    server_url_from_build: false,
    server_token_present: true,
    server_token_from_env: false,
    server_token_from_build: false,
    device_id: null,
    start_at_login: false,
    ...overrides,
  } as SettingsView;
}

function renderWith(
  engine: TranscriptionEngine,
  language: string,
  overrides: Partial<SettingsView> = {},
) {
  // No i18n provider: the context defaults to the English dictionary, which is
  // what every other component test relies on.
  const view = render(
    <SettingsScreen
      current={settings(engine, language, overrides)}
      onClose={() => {}}
      onSaved={() => {}}
      onRerunOnboarding={() => {}}
    />,
  );
  return view;
}

/** The screen opens on the UI-language tab; rows live on their own tabs. */
function openTab(name: RegExp) {
  fireEvent.click(screen.getByRole("tab", { name }));
}

function languageSelect(): HTMLSelectElement {
  openTab(/transcription/i);
  return screen.getByLabelText(/spoken language/i) as HTMLSelectElement;
}

function startAtLoginSwitch(): HTMLElement {
  openTab(/advanced/i);
  return screen.getByRole("switch", { name: /start at login/i });
}

/**
 * Only Whisper detects the spoken language. Deepgram ignores an absent
 * language and falls back to English, and the failure mode is silence — no
 * error, no transcript — so the option must not be offered there.
 */
describe("spoken-language options", () => {
  it("offers auto-detect on the on-device engine", () => {
    renderWith("whisper", "");
    const options = [...languageSelect().options].map((o) => o.value);
    expect(options).toContain("");
  });

  it("hides auto-detect on the online engine", () => {
    renderWith("deepgram", "en");
    const options = [...languageSelect().options].map((o) => o.value);
    expect(options).not.toContain("");
  });

  it("shows a legacy empty value as English online, rather than blank", () => {
    // Installs from before this change still hold "", which behaves as English
    // on the server. Leaving the select on a value no option matches would
    // render blank and misreport what the app is doing.
    renderWith("deepgram", "");
    expect(languageSelect().value).toBe("en");
  });

  it("still shows auto-detect selected for that same value on Whisper", () => {
    // The stored value is left alone, so switching engine back restores it.
    renderWith("whisper", "");
    expect(languageSelect().value).toBe("");
  });
});

/**
 * Auto-detection only runs while the process does, so the login item is what
 * makes it catch a meeting nobody thought to prepare for. It stays opt-in:
 * registering one unasked is the kind of thing users resent.
 */
describe("start at login", () => {
  it("is off unless the setting says otherwise", () => {
    renderWith("deepgram", "en");
    expect(startAtLoginSwitch().getAttribute("aria-checked")).toBe("false");
  });

  it("reflects an install that already opted in", () => {
    renderWith("deepgram", "en", { start_at_login: true });
    expect(startAtLoginSwitch().getAttribute("aria-checked")).toBe("true");
  });

  it("promises detection only where detection exists", () => {
    renderWith("deepgram", "en", { call_detection_supported: true });
    openTab(/advanced/i);
    expect(screen.getByText(/meetings are detected/i)).toBeTruthy();
  });

  it("says so plainly where it does not", () => {
    // Linux keeps the app resident but detects nothing, so the hint must not
    // imply a feature those users will not get.
    renderWith("deepgram", "en", { call_detection_supported: false });
    openTab(/advanced/i);
    expect(screen.getByText(/not available on this platform/i)).toBeTruthy();
  });
});
