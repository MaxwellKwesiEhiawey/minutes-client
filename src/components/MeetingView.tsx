import { useEffect, useMemo, useRef, useState } from "react";
import type { MeetingDetail, ShareFormat } from "../types";
import {
  formatDateTime,
  formatDuration,
  formatTime,
  meetingDurationMs,
  statusBadgeClass,
} from "../utils/format";
import { SummaryView } from "./SummaryView";
import { ShareModal } from "./ShareModal";
import { IconChevronLeft, IconShare, IconTrash } from "./Icons";
import { highlight } from "../utils/highlight";
import { groupSegments } from "../utils/transcript";
import { speakerColor, speakerInitials } from "../utils/speaker";
import type { AppError } from "../utils/errors";
import { useT, type Translate, type TranslationKey } from "../i18n";
import { errorText } from "../utils/errorText";
import { useStatusLabel } from "../utils/statusLabel";

/** Skeleton rows shown while a summary is being generated (design's shape). */
const SKELETONS: [string, string][] = [
  ["22px", "38%"],
  ["14px", "96%"],
  ["14px", "90%"],
  ["14px", "72%"],
  ["22px", "30%"],
  ["14px", "84%"],
  ["14px", "66%"],
];

interface Props {
  detail: MeetingDetail;
  highlightQuery: string;
  summarizing: boolean;
  summaryError: AppError | null;
  instructions: string;
  hasGlobalInstructions: boolean;
  onInstructionsChange: (value: string) => void;
  onGenerateSummary: () => void;
  onCopySummary: () => void;
  onCopyTranscript: () => void;
  onShare: (format: ShareFormat, includeTranscript: boolean) => void;
  onSave: (format: ShareFormat, includeTranscript: boolean) => void;
  shareSupported: boolean;
  onDelete: () => void;
  onBack: () => void;
}

/**
 * User-facing copy per error category, shown in the persistent summary banner.
 *
 * The `server` and default cases put `err.message` in the hint: that text comes
 * from the backend and is not translated, so the sentence around it carries the
 * meaning and the raw detail follows.
 */
function summaryErrorCopy(
  err: AppError,
  t: Translate,
): { title: string; hint: string } {
  const pair = (title: TranslationKey, hint: TranslationKey) => ({
    title: t(title),
    hint: t(hint),
  });
  switch (err.kind) {
    case "network":
      return pair("summaryError.networkTitle", "summaryError.networkHint");
    case "timeout":
      return pair("summaryError.timeoutTitle", "summaryError.timeoutHint");
    case "auth":
      return pair("summaryError.authTitle", "summaryError.authHint");
    // The hint is the backend's own detail. A coded error has a translated
    // sentence to show instead; anything else is diagnostic English, which is
    // still better than hiding why it failed.
    case "server":
      return { title: t("summaryError.serverTitle"), hint: errorText(err, t) };
    default:
      return { title: t("summaryError.genericTitle"), hint: errorText(err, t) };
  }
}

type PanelTab = "summary" | "transcript";

export function MeetingView({
  detail,
  highlightQuery,
  summarizing,
  summaryError,
  instructions,
  hasGlobalInstructions,
  onInstructionsChange,
  onGenerateSummary,
  onCopySummary,
  onCopyTranscript,
  onShare,
  onSave,
  shareSupported,
  onDelete,
  onBack,
}: Props) {
  const t = useT();
  const statusLabel = useStatusLabel();
  const [showShare, setShowShare] = useState(false);
  const { meeting, segments, summary } = detail;

  // Grouped transcript blocks. Memoised: meetings reach 400-500 segments and
  // only need regrouping when the segment list itself changes.
  const groups = useMemo(() => groupSegments(segments), [segments]);

  const [activeTab, setActiveTab] = useState<PanelTab>(
    summary ? "summary" : "transcript",
  );
  // The meeting whose tab the user picked themselves. A summary arriving (the
  // automatic post-meeting one, or a regenerate) reveals itself by switching to
  // the Summary tab — that is the payoff of waiting — but it must not yank
  // someone out of a transcript they deliberately opened. Storing the meeting id
  // rather than a flag means it expires by itself when another meeting opens.
  const tabChosenFor = useRef<string | null>(null);
  function chooseTab(tab: PanelTab) {
    tabChosenFor.current = meeting.id;
    setActiveTab(tab);
  }

  // The per-meeting instructions box earns its space before the first summary
  // exists; once a summary is there, collapse it out of prime position.
  const hasSummary = !!summary;
  const [instructionsOpen, setInstructionsOpen] = useState(!hasSummary);
  useEffect(() => {
    setInstructionsOpen(!hasSummary);
    if (tabChosenFor.current !== meeting.id) {
      setActiveTab(hasSummary ? "summary" : "transcript");
    }
  }, [meeting.id, hasSummary]);

  const canSummarize = segments.length > 0;
  const badgeClass = statusBadgeClass(meeting.status, false);
  const duration = formatDuration(
    meetingDurationMs(meeting.created_at, meeting.ended_at),
  );

  return (
    <div className="screen screen-detail">
      <button type="button" className="back-link" onClick={onBack}>
        <IconChevronLeft size={15} />
        <span>{t("detail.back")}</span>
      </button>

      <header className="detail-head">
        <div className="detail-head-text">
          <h2 className="detail-title" title={meeting.title}>
            {meeting.title}
          </h2>
          <div className="detail-meta">
            <span className={badgeClass}>{statusLabel(meeting.status, false)}</span>
            <span>{formatDateTime(meeting.created_at)}</span>
            <span>{duration}</span>
          </div>
        </div>
        <div className="detail-actions">
          <button
            type="button"
            className="icon-act"
            onClick={() => setShowShare(true)}
            disabled={segments.length === 0}
            title={t("detail.share")}
            aria-label={t("detail.share")}
          >
            <IconShare size={16} />
          </button>
          <button
            type="button"
            className="icon-act danger"
            onClick={onDelete}
            title={t("detail.delete")}
            aria-label={t("detail.delete")}
          >
            <IconTrash size={16} />
          </button>
        </div>
      </header>

      <div className="tabbar" role="tablist" aria-label={t("detail.tabsLabel")}>
        <button
          role="tab"
          type="button"
          className={activeTab === "summary" ? "tab active" : "tab"}
          aria-selected={activeTab === "summary"}
          onClick={() => chooseTab("summary")}
        >
          {t("detail.tabSummary")}
        </button>
        <button
          role="tab"
          type="button"
          className={activeTab === "transcript" ? "tab active" : "tab"}
          aria-selected={activeTab === "transcript"}
          onClick={() => chooseTab("transcript")}
        >
          {t("detail.tabTranscript")}
        </button>
        <div className="tabbar-right">
          {summarizing && (
            <span className="tabbar-state">{t("detail.summarizing")}</span>
          )}
          <button
            type="button"
            className="btn primary"
            onClick={onGenerateSummary}
            disabled={!canSummarize || summarizing}
            title={
              canSummarize
                ? t("detail.generateTitle")
                : t("detail.generateDisabled")
            }
          >
            {summarizing
              ? t("detail.summarizing")
              : summary
                ? t("detail.regenerate")
                : t("detail.generate")}
          </button>
        </div>
      </div>

      {activeTab === "summary" && (
        <section
          className="tab-panel"
          role="tabpanel"
          aria-label={t("detail.tabSummary")}
        >
          {summaryError && (
            <div className="err-card" role="alert">
              <h3>{summaryErrorCopy(summaryError, t).title}</h3>
              <p>{summaryErrorCopy(summaryError, t).hint}</p>
              <button
                type="button"
                className="btn primary"
                onClick={onGenerateSummary}
                disabled={summarizing}
              >
                {summarizing ? t("common.retrying") : t("common.tryAgain")}
              </button>
            </div>
          )}

          {canSummarize && (
            <details
              className="summary-instructions"
              open={instructionsOpen}
              onToggle={(e) => setInstructionsOpen(e.currentTarget.open)}
            >
              <summary>{t("detail.instructionsToggle")}</summary>
              <label htmlFor="meeting-instructions">
                {t("detail.instructionsLabel")}
              </label>
              <textarea
                id="meeting-instructions"
                rows={2}
                maxLength={2000}
                value={instructions}
                disabled={summarizing}
                onChange={(e) => onInstructionsChange(e.target.value)}
                placeholder={t("detail.instructionsPlaceholder")}
              />
              <p className="muted tiny">
                {hasGlobalInstructions
                  ? t("detail.instructionsCombined")
                  : t("detail.instructionsApplied")}
              </p>
            </details>
          )}

          {summarizing && !summary && (
            <>
              <div className="proc-strip">
                <span className="spinner" aria-hidden="true" />
                <span>{t("detail.writingSummary")}</span>
              </div>
              <div className="skel-stack" aria-hidden="true">
                {SKELETONS.map(([h, w], i) => (
                  <div
                    key={i}
                    className="skel"
                    style={{
                      height: h,
                      width: w,
                      animationDelay: `${i * 120}ms`,
                    }}
                  />
                ))}
              </div>
            </>
          )}

          {summary && <SummaryView summary={summary} />}

          {!summary && !summarizing && !summaryError && (
            <div className="panel-empty">
              <h3>{t("detail.noSummaryTitle")}</h3>
              <p>
                {canSummarize
                  ? t("detail.noSummaryReady")
                  : t("detail.noSummaryNoTranscript")}
              </p>
            </div>
          )}
        </section>
      )}

      {activeTab === "transcript" && (
        <section
          className="tab-panel"
          role="tabpanel"
          aria-label={t("detail.tabTranscript")}
        >
          {groups.length > 0 ? (
            <div className="transcript">
              {groups.map((g) => (
                <div key={g.key} className="tr-line">
                  <span
                    className="tr-avatar"
                    style={{ background: speakerColor(g.speaker) }}
                    aria-hidden="true"
                  >
                    {speakerInitials(g.speaker)}
                  </span>
                  <div className="tr-body">
                    <div className="tr-head">
                      <span className="tr-name">{g.speaker ?? t("detail.speaker")}</span>
                      <time className="tr-time" dateTime={g.startedAt}>
                        {formatTime(g.startedAt)}
                      </time>
                    </div>
                    <p className="tr-text">
                      {highlightQuery
                        ? highlight(g.text, highlightQuery)
                        : g.text}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="panel-empty">
              <h3>{t("detail.noTranscriptTitle")}</h3>
              <p>{t("detail.noTranscriptBody")}</p>
            </div>
          )}
        </section>
      )}

      {showShare && (
        <ShareModal
          hasSummary={!!summary}
          hasTranscript={segments.length > 0}
          onClose={() => setShowShare(false)}
          onCopySummary={onCopySummary}
          onCopyTranscript={onCopyTranscript}
          shareSupported={shareSupported}
          onShare={onShare}
          onSave={onSave}
        />
      )}
    </div>
  );
}
