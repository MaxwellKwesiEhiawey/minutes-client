import type { MeetingStatus } from "../types";

export function formatDate(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString();
}

export function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function formatDuration(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const hh = String(h).padStart(2, "0");
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return `${hh}:${mm}:${ss}`;
}

export function meetingDurationMs(
  createdAt: string,
  endedAt: string | null,
  live = false,
): number {
  const start = Date.parse(createdAt);
  const end = live || !endedAt ? Date.now() : Date.parse(endedAt);
  return Math.max(0, end - start);
}

export function formatStatusLabel(
  status: MeetingStatus | string,
  isRecording: boolean,
): string {
  if (isRecording) return "Recording";
  if (status === "interrupted") return "Interrupted";
  if (status === "completed") return "Completed";
  if (status === "recording") return "Recording";
  return status;
}

export function statusBadgeClass(
  status: MeetingStatus | string,
  isRecording: boolean,
): string {
  if (isRecording) return "badge badge-rec";
  if (status === "interrupted") return "badge badge-warn";
  return "badge badge-done";
}

/**
 * Turn a meeting title into a safe base filename for export (Markdown/Word)
 * save dialogs: strips characters that are invalid or awkward across
 * macOS/Windows/Linux filesystems and caps the length. Falls back to
 * `fallback` when the result would be empty (e.g. an emoji-only title).
 * Shared by every export path so the sanitization rule can't drift.
 */
export function sanitizeFilename(title: string, fallback = "meeting"): string {
  const safe = title.replace(/[^\w-]+/g, "_").slice(0, 60);
  // A title made entirely of characters the regex strips (emoji, punctuation
  // only, etc.) collapses to just underscores, not an empty string — treat
  // that as "empty" too rather than shipping a useless "_.docx".
  return /^_*$/.test(safe) ? fallback : safe;
}
