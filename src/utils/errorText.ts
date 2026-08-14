import { useCallback } from "react";
import { normalizeError, type AppError } from "./errors";
import { useT, type Translate, type TranslationKey } from "../i18n";
import { en } from "../i18n/en";

/**
 * Display text for anything a command rejected with.
 *
 * Backend errors arrive as `{ kind, message, code? }`. When a `code` is present
 * the backend meant the user to read it, so it is translated; `message` — the
 * English wording the backend also sends — is the fallback for a code this build
 * of the UI does not know, which is what happens when the Rust side ships a new
 * code before the dictionary catches up.
 */
export function errorText(e: unknown, t: Translate): string {
  const err: AppError = normalizeError(e);
  if (err.code && err.code in en) {
    return t(err.code as TranslationKey);
  }
  return err.message;
}

/**
 * Hook form, for components. Memoised on the translator, so it is stable across
 * renders and safe to list in a dependency array.
 */
export function useErrorText() {
  const t = useT();
  return useCallback((e: unknown) => errorText(e, t), [t]);
}
