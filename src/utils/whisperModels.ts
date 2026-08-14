/**
 * Single source of truth for whisper model id -> display label/size, so the
 * Settings dropdown options and the in-progress download hint can't drift
 * apart (they previously hardcoded the same sizes twice and had quietly gone
 * out of sync — 465 MB vs. 466 MB for "small").
 */
import type { TranslationKey } from "../i18n";

export interface WhisperModelInfo {
  id: string;
  /** Translation key, not text: the dropdown resolves it at render time. */
  label: TranslationKey;
  sizeLabel: string;
}

export const WHISPER_MODELS: WhisperModelInfo[] = [
  { id: "tiny", label: "model.tiny", sizeLabel: "75 MB" },
  { id: "base", label: "model.base", sizeLabel: "141 MB" },
  { id: "small", label: "model.small", sizeLabel: "466 MB" },
  { id: "medium", label: "model.medium", sizeLabel: "1.5 GB" },
  { id: "large-v3", label: "model.largeV3", sizeLabel: "3.1 GB" },
];

/** Size for a model id, or null when the id is not one we ship. */
export function whisperModelSizeLabel(id: string): string | null {
  return WHISPER_MODELS.find((m) => m.id === id)?.sizeLabel ?? null;
}

/**
 * Name an installed model from the structured fields rather than the backend's
 * `label`.
 *
 * `list_installed_models` sends `label` as English prose ("Whisper small",
 * "Speaker identification (diarization)"), which cannot be translated from here.
 * It also sends `id` and `kind`, which can — so the label is derived, and the
 * backend string is only a fallback for an id we do not know about.
 */
export function installedModelLabelKey(
  id: string,
  kind: string,
): TranslationKey | null {
  if (kind === "diarization") return "model.diarization";
  if (kind === "vad") return "model.vad";
  return WHISPER_MODELS.find((m) => m.id === id)?.label ?? null;
}
