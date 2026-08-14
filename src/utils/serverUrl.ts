import type { Translate } from "../i18n";

/**
 * Mirrors the backend's `validate_server_url` (src-tauri/src/settings.rs):
 * https:// is always allowed; http:// is allowed only for localhost/loopback.
 * This is a client-side hint only — the backend re-validates and is the
 * actual enforcement point, so this function must never be more permissive
 * than it (see SettingsModal.test.ts for the cases that matter, especially
 * the "http://localhost.evil.com" substring trick).
 */
export function serverUrlProblem(raw: string, t: Translate): string | null {
  const value = raw.trim();
  if (!value) return null; // empty means "leave unchanged" on save
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return t("serverUrl.enterFull");
  }
  if (url.protocol === "https:") return null;
  if (url.protocol !== "http:") {
    return t("serverUrl.onlyHttp");
  }
  const host = url.hostname.toLowerCase();
  const isLocal =
    host === "localhost" ||
    host === "127.0.0.1" ||
    host === "::1" ||
    host === "[::1]" ||
    host.endsWith(".localhost");
  if (isLocal) return null;
  return t("serverUrl.httpsRequired");
}
