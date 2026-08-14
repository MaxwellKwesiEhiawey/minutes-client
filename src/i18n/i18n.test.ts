import { afterEach, describe, expect, it, vi } from "vitest";
import { en } from "./en";
import {
  LOCALES,
  LOCALE_NAMES,
  dictionaryFor,
  getLocale,
  setLocale,
  translatorFor,
  type Locale,
} from "./index";

const KEYS = Object.keys(en) as (keyof typeof en)[];
/** Every locale except English, i.e. the ones that are translations. */
const TRANSLATED = LOCALES.filter((l) => l !== "en");

function placeholders(text: string): string[] {
  return [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
}

describe("dictionary completeness", () => {
  it("ships a dictionary and a native name for every locale", () => {
    for (const locale of LOCALES) {
      expect(dictionaryFor(locale), locale).toBeTruthy();
      expect(LOCALE_NAMES[locale], locale).toBeTruthy();
    }
  });

  it.each(TRANSLATED)("%s has exactly the keys English has", (locale) => {
    const keys = Object.keys(dictionaryFor(locale)).sort();
    expect(keys).toEqual([...KEYS].sort());
  });

  it.each(LOCALES)("%s has no empty or whitespace-only strings", (locale) => {
    const dictionary = dictionaryFor(locale);
    const blank = KEYS.filter((k) => dictionary[k].trim() === "");
    expect(blank).toEqual([]);
  });
});

describe("placeholder parity", () => {
  /* A dropped `{message}` or `{model}` does not throw — it just silently
     removes the detail from the sentence, which is how a translated error ends
     up less useful than the English one. */
  it.each(TRANSLATED)("%s keeps every placeholder English uses", (locale) => {
    const dictionary = dictionaryFor(locale);
    const mismatched = KEYS.filter(
      (k) => placeholders(en[k]).join() !== placeholders(dictionary[k]).join(),
    );
    expect(mismatched).toEqual([]);
  });
});

describe("translation coverage", () => {
  /* Guards against a dictionary that is really English with a different name on
     it. A handful of strings are legitimately identical across languages —
     "Format", "PDF (.pdf)", "Status", "Markdown" — so this asserts most strings
     differ rather than all of them. */
  it.each(TRANSLATED)("%s is not a copy of English", (locale) => {
    const dictionary = dictionaryFor(locale);
    const identical = KEYS.filter((k) => dictionary[k] === en[k]);
    expect(identical.length / KEYS.length).toBeLessThan(0.15);
  });

  it("translates the strings that describe where data goes", () => {
    // These carry a privacy meaning, so an untranslated one is worse than a
    // missing feature: it leaves a non-English reader unable to check the claim.
    const critical = [
      "share.includeOff",
      "share.includeOn",
      "settings.telemetryDetail",
      "settings.engineWhisperHint",
      "settings.captureSystemAudioHint",
      "settings.autoSummarizeHint",
    ] as const;
    for (const locale of TRANSLATED) {
      const dictionary = dictionaryFor(locale);
      for (const key of critical) {
        expect(dictionary[key], `${locale} ${key}`).not.toBe(en[key]);
        expect(dictionary[key].length, `${locale} ${key}`).toBeGreaterThan(20);
      }
    }
  });
});

describe("interpolation", () => {
  it("substitutes named placeholders", () => {
    const t = translatorFor("en");
    expect(t("notes.results", { query: "budget" })).toBe(
      "Results for “budget”",
    );
  });

  it("leaves a placeholder alone when no value is supplied", () => {
    // Better a visible `{message}` in a bug report than a silently truncated
    // sentence that reads as if nothing went wrong.
    const t = translatorFor("en");
    expect(t("toast.transcription")).toContain("{message}");
  });

  it("returns the German string for a German translator", () => {
    expect(translatorFor("de")("nav.settings")).toBe("Einstellungen");
    expect(translatorFor("fr")("nav.settings")).toBe("Réglages");
  });
});

describe("locale detection", () => {
  const originalLanguages = navigator.languages;

  function pretendLanguages(...tags: string[]) {
    Object.defineProperty(navigator, "languages", {
      configurable: true,
      value: tags,
    });
  }

  afterEach(() => {
    localStorage.clear();
    Object.defineProperty(navigator, "languages", {
      configurable: true,
      value: originalLanguages,
    });
    vi.restoreAllMocks();
  });

  it("prefers an explicit choice over the system language", () => {
    pretendLanguages("fr-FR");
    setLocale("nl");
    expect(getLocale()).toBe("nl");
  });

  it("falls back to the system language", () => {
    pretendLanguages("de-DE", "en-GB");
    expect(getLocale()).toBe("de");
  });

  it("matches on the base subtag, so a region variant still counts", () => {
    // de-AT and pt-BR must not fall through to English over a region.
    pretendLanguages("de-AT");
    expect(getLocale()).toBe("de");
    pretendLanguages("pt-BR");
    expect(getLocale()).toBe("pt");
  });

  it("falls back to English for a language we do not ship", () => {
    pretendLanguages("ja-JP", "ko-KR");
    expect(getLocale()).toBe("en");
  });

  it("ignores a stored value that is not a locale we ship", () => {
    pretendLanguages("it-IT");
    localStorage.setItem("desksec-locale", "klingon");
    expect(getLocale()).toBe("it");
  });

  it("still detects a language when storage is unavailable", () => {
    // A locked-down profile can throw on access; the OS language is still a
    // better answer than English.
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    pretendLanguages("es-ES");
    expect(getLocale() as Locale).toBe("es");
  });
});

describe("server connection state", () => {
  /* The pill used to render `ServerStatus.message`, which the Rust side writes
     in English ("Connected", "Server error (503)"), so it stayed English next to
     translated text. The four states are structured data, so every locale must
     have its own words for them. */
  it.each(TRANSLATED)("%s names every connection state itself", (locale) => {
    const dictionary = dictionaryFor(locale);
    for (const key of [
      "settings.connected",
      "settings.notConfigured",
      "settings.unreachable",
      "settings.checking",
    ] as const) {
      expect(dictionary[key], `${locale} ${key}`).toBeTruthy();
      expect(dictionary[key], `${locale} ${key}`).not.toBe(en[key]);
    }
  });
});
