import { describe, expect, it } from "vitest";
import { engineModeLabel } from "./engineLabel";
import { translatorFor } from "../i18n";
import type { SettingsView } from "../types";

function make(overrides: Partial<SettingsView>): SettingsView {
  return { whisper_model: "small", ...overrides } as SettingsView;
}

// The real English translator, so the test covers the dictionary too: a key
// renamed in en.ts without updating this call site fails here.
const t = translatorFor("en");

describe("engineModeLabel", () => {
  it("describes on-device transcription in plain language, engine in the tooltip", () => {
    const { label, title } = engineModeLabel(
      make({ transcription_engine: "whisper", whisper_model: "small" }),
      t,
    );
    expect(label).toBe("Private · on this device");
    expect(label).not.toMatch(/whisper/i);
    expect(title).toMatch(/whisper/i);
    expect(title).toMatch(/small/);
  });

  it("says plainly when transcription happens online", () => {
    const { label, title } = engineModeLabel(
      make({ transcription_engine: "deepgram" }),
      t,
    );
    expect(label).toBe("Cloud transcription");
    expect(label).not.toMatch(/deepgram/i);
    expect(title).toMatch(/deepgram/i);
  });
});
