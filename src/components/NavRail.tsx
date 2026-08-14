import {
  BrandMark,
  IconHome,
  IconNotes,
  IconPlus,
  IconSettings,
} from "./Icons";
import type { Screen } from "../screens";
import { useT } from "../i18n";

interface Props {
  screen: Screen;
  collapsed: boolean;
  busy: boolean;
  onHome: () => void;
  onNotes: () => void;
  onSettings: () => void;
  onNewMeeting: () => void;
}

export function NavRail({
  screen,
  collapsed,
  busy,
  onHome,
  onNotes,
  onSettings,
  onNewMeeting,
}: Props) {
  const t = useT();
  // "detail" and "recording" are reached from the notes list, so the list stays
  // highlighted while one of its meetings is open.
  const notesActive =
    screen === "notes" || screen === "detail" || screen === "recording";

  return (
    <aside className={collapsed ? "nav-rail collapsed" : "nav-rail"}>
      <button
        type="button"
        className="nav-brand"
        onClick={onHome}
        title={t("nav.brandHome")}
        aria-label={t("nav.brandHome")}
      >
        <BrandMark size={collapsed ? 28 : 26} className="nav-brand-mark" />
        <span className="nav-brand-word">Minutes</span>
      </button>

      <div className="nav-new-wrap">
        <button
          type="button"
          className="nav-new"
          onClick={onNewMeeting}
          disabled={busy}
          title={t("nav.newMeeting")}
        >
          <IconPlus size={18} />
          <span>{t("nav.newMeeting")}</span>
        </button>
      </div>

      <nav className="nav-group" aria-label={t("nav.main")}>
        <button
          type="button"
          className={screen === "home" ? "nav-item active" : "nav-item"}
          aria-current={screen === "home" ? "page" : undefined}
          onClick={onHome}
          title={t("nav.home")}
        >
          <IconHome size={19} />
          <span>{t("nav.home")}</span>
        </button>
        <button
          type="button"
          className={notesActive ? "nav-item active" : "nav-item"}
          aria-current={notesActive ? "page" : undefined}
          onClick={onNotes}
          title={t("nav.myNotes")}
        >
          <IconNotes size={19} />
          <span>{t("nav.myNotes")}</span>
        </button>
      </nav>

      <div className="nav-list" />

      <div className="nav-foot">
        <button
          type="button"
          className={screen === "settings" ? "nav-item active" : "nav-item"}
          aria-current={screen === "settings" ? "page" : undefined}
          onClick={onSettings}
          title={t("nav.settings")}
        >
          <IconSettings size={18} />
          <span>{t("nav.settings")}</span>
        </button>
      </div>
    </aside>
  );
}
