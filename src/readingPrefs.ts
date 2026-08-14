// Reading-comfort preferences. These tune how the transcript (the app's main
// long-reading surface) renders, plus app-wide contrast and motion. Like the
// theme preference, they are stored locally and applied to the document root as
// data-* attributes that styles.css keys off — independent of server settings.

export type TextScale = "normal" | "large" | "xlarge";
export type LineSpacing = "normal" | "relaxed" | "loose";

export interface ReadingPrefs {
  textScale: TextScale;
  lineSpacing: LineSpacing;
  highContrast: boolean;
  reduceMotion: boolean;
}

const STORAGE_KEY = "desksec-reading";

const DEFAULTS: ReadingPrefs = {
  textScale: "normal",
  lineSpacing: "normal",
  highContrast: false,
  reduceMotion: false,
};

const TEXT_SCALES: TextScale[] = ["normal", "large", "xlarge"];
const LINE_SPACINGS: LineSpacing[] = ["normal", "relaxed", "loose"];

export function getReadingPrefs(): ReadingPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<ReadingPrefs>;
    return {
      textScale: TEXT_SCALES.includes(parsed.textScale as TextScale)
        ? (parsed.textScale as TextScale)
        : DEFAULTS.textScale,
      lineSpacing: LINE_SPACINGS.includes(parsed.lineSpacing as LineSpacing)
        ? (parsed.lineSpacing as LineSpacing)
        : DEFAULTS.lineSpacing,
      highContrast: Boolean(parsed.highContrast),
      reduceMotion: Boolean(parsed.reduceMotion),
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export function applyReadingPrefs(prefs?: ReadingPrefs) {
  const p = prefs ?? getReadingPrefs();
  const root = document.documentElement;
  root.dataset.textScale = p.textScale;
  root.dataset.lineSpacing = p.lineSpacing;
  if (p.highContrast) root.dataset.highContrast = "true";
  else delete root.dataset.highContrast;
  if (p.reduceMotion) root.dataset.reduceMotion = "true";
  else delete root.dataset.reduceMotion;
}

export function setReadingPrefs(prefs: ReadingPrefs) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  applyReadingPrefs(prefs);
}

/** Whether motion should be minimized (explicit toggle or OS preference). */
export function prefersReducedMotion(): boolean {
  if (getReadingPrefs().reduceMotion) return true;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function initReadingPrefs() {
  applyReadingPrefs();
}
