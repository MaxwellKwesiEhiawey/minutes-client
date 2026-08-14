import type { MeetingListItem } from "../types";
import {
  formatDate,
  formatDuration,
  meetingDurationMs,
  statusBadgeClass,
} from "../utils/format";
import { IconMic, IconPlus, IconSummary } from "./Icons";
import artwork from "../assets/img/online-meeting.png";
import { useT } from "../i18n";
import { useStatusLabel } from "../utils/statusLabel";

const RECENT_COUNT = 4;

interface Props {
  meetings: MeetingListItem[];
  recordingId: string | null;
  busy: boolean;
  onNewMeeting: () => void;
  onOpen: (id: string) => void;
  onViewAll: () => void;
}

export function HomeScreen({
  meetings,
  recordingId,
  busy,
  onNewMeeting,
  onOpen,
  onViewAll,
}: Props) {
  const t = useT();
  const statusLabel = useStatusLabel();
  const recents = meetings.slice(0, RECENT_COUNT);

  return (
    <div className="screen screen-home">
      <div className="home-hero">
        <div className="home-hero-text">
          <h2>{t("home.greeting")}</h2>
          <p className="home-sub">{t("home.sub")}</p>
          <div className="home-cta">
            <button
              type="button"
              className="btn primary btn-lg"
              onClick={onNewMeeting}
              disabled={busy}
            >
              <IconPlus size={18} />
              <span>{t("nav.newMeeting")}</span>
            </button>
          </div>
        </div>
        <img src={artwork} alt="" className="home-art" />
      </div>

      <div className="section-head">
        <h3>{t("home.recent")}</h3>
        {meetings.length > 0 && (
          <button type="button" className="link-btn" onClick={onViewAll}>
            {t("home.viewAll")}
          </button>
        )}
      </div>

      {recents.length > 0 ? (
        <div className="card-grid">
          {recents.map((m) => {
            const isRec = m.id === recordingId;
            return (
              <button
                type="button"
                key={m.id}
                className="m-card"
                onClick={() => onOpen(m.id)}
              >
                <div className="m-card-head">
                  <div className="m-card-title">{m.title}</div>
                  <span className={statusBadgeClass(m.status, isRec)}>
                    {statusLabel(m.status, isRec)}
                  </span>
                </div>
                <div className="m-card-meta">
                  <span>{formatDate(m.created_at)}</span>
                  <span>
                    {formatDuration(
                      meetingDurationMs(m.created_at, m.ended_at, isRec),
                    )}
                  </span>
                </div>
                <div className="m-card-foot">
                  {m.has_summary ? (
                    <>
                      <IconSummary size={14} />
                      <span>{t("home.summaryReady")}</span>
                    </>
                  ) : (
                    <>
                      <IconMic size={14} />
                      <span>{t("home.transcriptOnly")}</span>
                    </>
                  )}
                </div>
              </button>
            );
          })}
        </div>
      ) : (
        <div className="empty-panel">
          <div className="empty-icon">
            <IconMic size={26} />
          </div>
          <h3>{t("home.emptyTitle")}</h3>
          <p>{t("home.emptyBody")}</p>
          <div className="empty-actions">
            <button
              type="button"
              className="btn primary"
              onClick={onNewMeeting}
              disabled={busy}
            >
              {t("home.emptyCta")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
