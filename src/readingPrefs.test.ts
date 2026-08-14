import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  applyReadingPrefs,
  getReadingPrefs,
  prefersReducedMotion,
  setReadingPrefs,
} from "./readingPrefs";

const STORAGE_KEY = "desksec-reading";

function mockReducedMotion(matches: boolean) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches,
    media: query,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })) as unknown as typeof window.matchMedia;
}

describe("readingPrefs", () => {
  beforeEach(() => {
    localStorage.clear();
    delete document.documentElement.dataset.textScale;
    delete document.documentElement.dataset.lineSpacing;
    delete document.documentElement.dataset.highContrast;
    delete document.documentElement.dataset.reduceMotion;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns sane defaults when nothing is stored", () => {
    expect(getReadingPrefs()).toEqual({
      textScale: "normal",
      lineSpacing: "normal",
      highContrast: false,
      reduceMotion: false,
    });
  });

  it("returns defaults when the stored value is malformed JSON", () => {
    localStorage.setItem(STORAGE_KEY, "{not-json");
    expect(getReadingPrefs()).toEqual({
      textScale: "normal",
      lineSpacing: "normal",
      highContrast: false,
      reduceMotion: false,
    });
  });

  it("falls back per-field for an out-of-range enum value instead of discarding everything", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ textScale: "huge", lineSpacing: "loose", highContrast: true })
    );
    expect(getReadingPrefs()).toEqual({
      textScale: "normal", // invalid -> default
      lineSpacing: "loose", // valid -> kept
      highContrast: true,
      reduceMotion: false,
    });
  });

  it("round-trips valid prefs through localStorage", () => {
    setReadingPrefs({
      textScale: "large",
      lineSpacing: "relaxed",
      highContrast: true,
      reduceMotion: true,
    });
    expect(getReadingPrefs()).toEqual({
      textScale: "large",
      lineSpacing: "relaxed",
      highContrast: true,
      reduceMotion: true,
    });
  });

  it("applyReadingPrefs writes data-* attributes and removes them when falsy", () => {
    applyReadingPrefs({
      textScale: "xlarge",
      lineSpacing: "loose",
      highContrast: true,
      reduceMotion: true,
    });
    const root = document.documentElement;
    expect(root.dataset.textScale).toBe("xlarge");
    expect(root.dataset.lineSpacing).toBe("loose");
    expect(root.dataset.highContrast).toBe("true");
    expect(root.dataset.reduceMotion).toBe("true");

    applyReadingPrefs({
      textScale: "normal",
      lineSpacing: "normal",
      highContrast: false,
      reduceMotion: false,
    });
    expect(root.dataset.highContrast).toBeUndefined();
    expect(root.dataset.reduceMotion).toBeUndefined();
  });

  it("prefersReducedMotion is true when the explicit toggle is set, regardless of OS setting", () => {
    mockReducedMotion(false);
    setReadingPrefs({
      textScale: "normal",
      lineSpacing: "normal",
      highContrast: false,
      reduceMotion: true,
    });
    expect(prefersReducedMotion()).toBe(true);
  });

  it("prefersReducedMotion falls back to the OS media query when the toggle is off", () => {
    setReadingPrefs({
      textScale: "normal",
      lineSpacing: "normal",
      highContrast: false,
      reduceMotion: false,
    });
    mockReducedMotion(true);
    expect(prefersReducedMotion()).toBe(true);
    mockReducedMotion(false);
    expect(prefersReducedMotion()).toBe(false);
  });
});
