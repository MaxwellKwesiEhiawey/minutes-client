import { createContext, useContext } from "react";
import { en } from "./en";
import { de } from "./de";
import { fr } from "./fr";
import { es } from "./es";
import { it } from "./it";
import { pt } from "./pt";
import { nl } from "./nl";

/**
 * UI language.
 *
 * A device-local presentation preference, like the theme and the reading-comfort
 * settings — not a server-backed one. It changes nothing about what the backend
 * does, so it lives in `localStorage` and applies instantly, following
 * `src/theme.ts`. Distinct from `transcription_language` and `summary_language`
 * in Settings, which do change backend behaviour.
 */
export type Locale = "en" | "de" | "fr" | "es" | "it" | "pt" | "nl";

/** Every key the UI can ask for. `en` is the source of truth; the other
 *  dictionaries are typed against it, so a missing or misspelled key is a
 *  compile error rather than a string like "settings.audio.blurb" on screen. */
export type TranslationKey = keyof typeof en;
export type Translations = Record<TranslationKey, string>;

const DICTIONARIES: Record<Locale, Translations> = { en, de, fr, es, it, pt, nl };

/** Shown in the language picker: each language named in itself, which is what a
 *  speaker of it will recognise while the UI is still in a language they can't
 *  read. */
export const LOCALE_NAMES: Record<Locale, string> = {
  en: "English",
  de: "Deutsch",
  fr: "Français",
  es: "Español",
  it: "Italiano",
  pt: "Português",
  nl: "Nederlands",
};

export const LOCALES = Object.keys(LOCALE_NAMES) as Locale[];

const STORAGE_KEY = "desksec-locale";

function isLocale(value: string): value is Locale {
  return (LOCALES as string[]).includes(value);
}

/**
 * The locale to start in: an explicit choice if one was made, otherwise the
 * closest match to the operating system's language, otherwise English.
 *
 * Matches on the base subtag, so `de-AT` and `pt-BR` land on `de` and `pt`
 * rather than falling back to English over a region difference.
 */
export function getLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && isLocale(stored)) return stored;
  } catch {
    // Storage can be unavailable (private mode, a locked-down profile); the OS
    // language is still a better guess than English.
  }
  for (const tag of navigator.languages ?? [navigator.language]) {
    const base = tag?.split("-")[0]?.toLowerCase();
    if (base && isLocale(base)) return base;
  }
  return "en";
}

/** Persist the choice and reflect it on the document for assistive tech. */
export function setLocale(locale: Locale) {
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Non-fatal: the app still switches for this session.
  }
  applyLocale(locale);
}

/** Set `<html lang>` so screen readers use the right pronunciation rules. */
export function applyLocale(locale: Locale = getLocale()) {
  document.documentElement.lang = locale;
}

export function dictionaryFor(locale: Locale): Translations {
  return DICTIONARIES[locale];
}

/** A translator: `t("home.greeting")`, or `t("notes.results", { query })`. */
export type Translate = (
  key: TranslationKey,
  params?: Record<string, string | number>,
) => string;

export function translatorFor(locale: Locale): Translate {
  const dictionary = dictionaryFor(locale);
  return (key, params) => {
    // `en` is the fallback for a string a dictionary somehow lacks at runtime —
    // the types prevent it, but a hand-edited dictionary should degrade to
    // English rather than to a raw key.
    const template = dictionary[key] ?? en[key] ?? key;
    if (!params) return template;
    return template.replace(/\{(\w+)\}/g, (whole, name: string) =>
      name in params ? String(params[name]) : whole,
    );
  };
}

interface I18nValue {
  locale: Locale;
  t: Translate;
  setLocale: (locale: Locale) => void;
}

export const I18nContext = createContext<I18nValue>({
  locale: "en",
  t: translatorFor("en"),
  setLocale: () => undefined,
});

/** The hook every component uses: `const { t } = useI18n()`. */
export function useI18n(): I18nValue {
  return useContext(I18nContext);
}

/** Shorthand for components that only need the translator. */
export function useT(): Translate {
  return useContext(I18nContext).t;
}
