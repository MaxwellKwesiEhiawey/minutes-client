import type { SettingsView } from "../types";
import type { Translate } from "../i18n";

/**
 * Plain-language description of where transcription runs, plus the technical
 * detail as a tooltip. Users should never have to know the word "Whisper" to
 * understand whether their audio leaves the device, so the visible label says
 * what it means and the engine name stays in the title attribute.
 */
export function engineModeLabel(
  settings: SettingsView,
  t: Translate,
): { label: string; title: string } {
  if (settings.transcription_engine === "whisper") {
    return {
      label: t("engine.onDevice"),
      title: t("engine.onDeviceTitle", { model: settings.whisper_model }),
    };
  }
  return { label: t("engine.cloud"), title: t("engine.cloudTitle") };
}
