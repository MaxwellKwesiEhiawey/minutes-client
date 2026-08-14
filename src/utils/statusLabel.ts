import { useT, type TranslationKey } from "../i18n";
import type { MeetingStatus } from "../types";

/**
 * Translated meeting status.
 *
 * `formatStatusLabel` in `format.ts` still exists and is still what the export
 * paths use — a document keeps the English status, because the file may be read
 * by someone who does not share the reader's UI language. This is the on-screen
 * version, and the two are deliberately separate for that reason.
 */
export function useStatusLabel() {
  const t = useT();
  return (status: MeetingStatus | string, isRecording: boolean): string => {
    if (isRecording) return t("status.recording");
    const key: TranslationKey =
      status === "interrupted"
        ? "status.interrupted"
        : status === "recording"
          ? "status.recording"
          : status === "completed"
            ? "status.completed"
            : "status.completed";
    // An unrecognised status from the backend is shown as-is rather than
    // silently relabelled as something it isn't.
    if (status !== "interrupted" && status !== "recording" && status !== "completed") {
      return status;
    }
    return t(key);
  };
}
