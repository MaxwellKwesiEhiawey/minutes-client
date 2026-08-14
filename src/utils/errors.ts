/**
 * Normalizes whatever a Tauri command rejects with into one consistent shape.
 *
 * Most commands still reject with a plain string (the Rust side's
 * `CmdResult<T> = Result<T, String>`). A few network/auth-sensitive paths
 * (currently AI summary generation) reject with a structured
 * `{ kind, message }` object instead, so the UI can distinguish "can't reach
 * the server" from "bad token" from a generic failure. This is the one place
 * that distinction is resolved, so call sites never need their own
 * `String(e)`/`instanceof` checks (which could render "[object Object]" for
 * a non-Error, non-string rejection).
 */

export type ErrorKind = "network" | "timeout" | "auth" | "server" | "internal";

export interface AppError {
  kind: ErrorKind;
  message: string;
  /**
   * Stable translation key when the backend meant this for the user to read.
   * The backend is never told the UI language — that is a device-local
   * preference — so a message a user acts on travels as an identifier and is
   * translated here. `message` is the English fallback and the log line.
   */
  code?: string;
}

const VALID_KINDS: readonly ErrorKind[] = [
  "network",
  "timeout",
  "auth",
  "server",
  "internal",
];

function isErrorKind(value: unknown): value is ErrorKind {
  return typeof value === "string" && (VALID_KINDS as readonly string[]).includes(value);
}

function isStructuredError(value: unknown): value is AppError {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  if (!isErrorKind(candidate.kind) || typeof candidate.message !== "string") {
    return false;
  }
  return candidate.code === undefined || typeof candidate.code === "string";
}

/**
 * Fallback message for a rejection carrying nothing usable. Exported so the UI
 * can recognise it and substitute a translated string — the value itself has to
 * stay a plain constant because `normalizeError` is called from non-React code.
 */
export const UNKNOWN_ERROR = "An unknown error occurred.";

export function normalizeError(e: unknown): AppError {
  if (isStructuredError(e)) return e;
  if (e instanceof Error) return { kind: "internal", message: e.message };
  if (typeof e === "string") return { kind: "internal", message: e };
  if (e === null || e === undefined) {
    return { kind: "internal", message: UNKNOWN_ERROR };
  }
  // Last resort for anything else Tauri could hand back (e.g. a plain
  // object without our expected shape) — avoid the raw `String(e)` ===
  // "[object Object]" footgun by trying JSON first.
  try {
    return { kind: "internal", message: JSON.stringify(e) };
  } catch {
    return { kind: "internal", message: String(e) };
  }
}

/** Convenience for the common case of just wanting a display string. */
export function errorMessage(e: unknown): string {
  return normalizeError(e).message;
}
