import { IconMoon, IconPanel, IconSearch, IconSun } from "./Icons";
import type { ThemePreference } from "../theme";
import { useT, type TranslationKey } from "../i18n";

/** Live recording, surfaced on every screen so it is never out of reach. */
export interface RecordingIndicator {
  elapsed: string;
  busy: boolean;
  onOpen: () => void;
  onStop: () => void;
}

interface Props {
  title: string;
  themePreference: ThemePreference;
  recording: RecordingIndicator | null;
  onToggleRail: () => void;
  onOpenPalette: () => void;
  onCycleTheme: () => void;
}

const THEME_KEY: Record<ThemePreference, TranslationKey> = {
  light: "theme.light",
  dark: "theme.dark",
  system: "theme.system",
};

export function TopBar({
  title,
  themePreference,
  recording,
  onToggleRail,
  onOpenPalette,
  onCycleTheme,
}: Props) {
  const t = useT();
  const themeLabel = t(THEME_KEY[themePreference]);
  return (
    <header className="topbar">
      <button
        type="button"
        className="topbar-toggle"
        onClick={onToggleRail}
        title={t("topbar.toggleSidebar")}
        aria-label={t("topbar.toggleSidebar")}
      >
        <IconPanel size={17} />
      </button>

      <h1 className="topbar-title">{title}</h1>

      <button
        type="button"
        className="topbar-search"
        onClick={onOpenPalette}
        aria-label={t("topbar.searchLabel")}
      >
        <IconSearch size={16} />
        <span className="topbar-search-label">{t("topbar.search")}</span>
        <span className="kbd" aria-hidden="true">
          ⌘K
        </span>
      </button>

      <div className="topbar-right">
        {recording && (
          <div className="topbar-rec">
            <button
              type="button"
              className="topbar-rec-open"
              onClick={recording.onOpen}
              title={t("topbar.recordingOpen")}
              aria-label={t("topbar.recordingOpen")}
            >
              <span className="rec-status-dot" aria-hidden="true" />
              <span className="topbar-rec-time" aria-live="polite">
                {recording.elapsed}
              </span>
            </button>
            <button
              type="button"
              className="topbar-rec-stop"
              onClick={recording.onStop}
              disabled={recording.busy}
              title={t("topbar.recordingStop")}
              aria-label={t("topbar.recordingStop")}
            >
              <span className="rec-end-square" aria-hidden="true" />
              {t("topbar.stop")}
            </button>
          </div>
        )}
        <button
          type="button"
          className="theme-btn"
          onClick={onCycleTheme}
          title={t("topbar.themeTitle", { theme: themeLabel })}
        >
          {themePreference === "dark" ? (
            <IconMoon size={16} />
          ) : (
            <IconSun size={16} />
          )}
          <span>{themeLabel}</span>
        </button>
      </div>
    </header>
  );
}
