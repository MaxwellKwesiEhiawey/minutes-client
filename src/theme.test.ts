import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  applyTheme,
  getThemePreference,
  resolveTheme,
  setThemePreference,
} from "./theme";

const STORAGE_KEY = "desksec-theme";

function mockMatchMedia(matches: boolean) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches,
    media: query,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })) as unknown as typeof window.matchMedia;
}

describe("theme", () => {
  beforeEach(() => {
    localStorage.clear();
    delete document.documentElement.dataset.theme;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("defaults to 'system' when nothing is stored", () => {
    expect(getThemePreference()).toBe("system");
  });

  it("falls back to 'system' for a corrupted/unexpected stored value", () => {
    localStorage.setItem(STORAGE_KEY, "purple");
    expect(getThemePreference()).toBe("system");
  });

  it("round-trips an explicit preference through localStorage", () => {
    setThemePreference("dark");
    expect(getThemePreference()).toBe("dark");
    setThemePreference("light");
    expect(getThemePreference()).toBe("light");
  });

  it("resolveTheme returns the explicit preference for light/dark", () => {
    expect(resolveTheme("light")).toBe("light");
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("resolveTheme falls through to matchMedia for 'system'", () => {
    mockMatchMedia(true);
    expect(resolveTheme("system")).toBe("dark");
    mockMatchMedia(false);
    expect(resolveTheme("system")).toBe("light");
  });

  it("applyTheme writes the resolved theme onto the document root", () => {
    mockMatchMedia(false);
    applyTheme("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("applyTheme with no argument uses the stored/system preference", () => {
    mockMatchMedia(true);
    setThemePreference("system");
    applyTheme();
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
