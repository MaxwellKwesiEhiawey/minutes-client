import { useEffect, useMemo, useRef } from "react";
import type { MeetingDetail } from "../types";
import { formatTime } from "../utils/format";
import { groupSegments } from "../utils/transcript";
import { speakerColor, speakerInitials } from "../utils/speaker";
import { IconChevronLeft } from "./Icons";
import { useT } from "../i18n";

/** Bar count in the design's waveform card. */
const BAR_COUNT = 44;

interface Props {
  detail: MeetingDetail;
  elapsed: string;
  /** Live input level, 0..1, from the backend's level events. */
  level: number;
  partialText: string;
  engineMode: { label: string; title: string } | null;
  busy: boolean;
  onStop: () => void;
  onBack: () => void;
}

export function RecordingScreen({
  detail,
  elapsed,
  level,
  partialText,
  engineMode,
  busy,
  onStop,
  onBack,
}: Props) {
  const t = useT();
  const scrollRef = useRef<HTMLDivElement>(null);
  const { meeting, segments } = detail;
  const groups = useMemo(() => groupSegments(segments), [segments]);

  // Keep the newest line in view by scrolling the transcript's own container,
  // never the page. Scrolling the page is what made the screen twitch: interim
  // text arrives several times a second, and each scroll dragged the timer and
  // waveform along with it.
  //
  // Following is a mode, not a per-update guess: it stays on until the reader
  // scrolls up to re-read something, and resumes when they come back to the
  // bottom. Deciding per update from the current offset instead would give up
  // following for good the moment the transcript outgrew the box.
  const following = useRef(true);

  function onScroll() {
    const el = scrollRef.current;
    if (!el) return;
    following.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
  }

  useEffect(() => {
    const el = scrollRef.current;
    if (!el || !following.current) return;
    el.scrollTop = el.scrollHeight;
  }, [segments.length, partialText]);

  // Bar heights come from the real input level. A centred envelope makes the
  // middle of the meter react most, and a fixed per-bar factor keeps the shape
  // ragged like audio rather than a smooth lens — deterministic, so bars don't
  // jump around between renders at a steady level.
  const bars = useMemo(() => {
    const amplitude = Math.min(1, Math.max(0, level * 3.2));
    return Array.from({ length: BAR_COUNT }, (_, i) => {
      const centre =
        1 - Math.abs(i - (BAR_COUNT - 1) / 2) / ((BAR_COUNT - 1) / 2);
      const envelope = 0.35 + 0.65 * centre;
      const texture = 0.55 + 0.45 * Math.abs(Math.sin(i * 2.399));
      const height = Math.max(
        4,
        Math.round(amplitude * envelope * texture * 84),
      );
      return { key: i, height };
    });
  }, [level]);

  return (
    <div className="screen screen-rec">
      <button type="button" className="back-link" onClick={onBack}>
        <IconChevronLeft size={15} />
        <span>{t("recording.back")}</span>
      </button>

      <div className="rec-head">
        <div>
          <span className="rec-status">
            <span className="rec-status-dot" aria-hidden="true" />
            {t("status.recording")}
          </span>
          <h2 className="rec-title">{meeting.title}</h2>
          <div className="rec-sub">
            <span>{t("recording.transcriptSaved")}</span>
            {engineMode && (
              <span className="pill" title={engineMode.title}>
                {engineMode.label}
              </span>
            )}
          </div>
        </div>
        <div className="rec-timer" aria-live="polite">
          {elapsed}
        </div>
      </div>

      <div className="rec-card">
        <div className="rec-wave" title={t("recording.inputLevel")} aria-hidden="true">
          {bars.map((b) => (
            <span
              key={b.key}
              className="rec-bar"
              style={{ height: `${b.height}px` }}
            />
          ))}
        </div>
        <div className="rec-controls">
          <button
            type="button"
            className="rec-end"
            onClick={onStop}
            disabled={busy}
          >
            <span className="rec-end-square" aria-hidden="true" />
            {t("recording.endMeeting")}
          </button>
        </div>
      </div>

      <div className="live-section">
        <div className="live-head">
          <h3>{t("recording.liveTranscript")}</h3>
          <span className="muted tiny">
            {groups.length > 0
              ? t("recording.savedAsCaptured")
              : t("recording.nothingYet")}
          </span>
        </div>
        <div className="live-scroll" ref={scrollRef} onScroll={onScroll}>
          <div className="live-list" aria-live="polite">
          {groups.map((g) => (
            <div key={g.key} className="live-line">
              <span
                className="live-avatar"
                style={{ background: speakerColor(g.speaker) }}
                aria-hidden="true"
              >
                {speakerInitials(g.speaker)}
              </span>
              <div className="tr-body">
                <div className="live-meta">
                  <strong>{g.speaker ?? t("detail.speaker")}</strong> ·{" "}
                  {formatTime(g.startedAt)}
                </div>
                <div className="tr-text">{g.text}</div>
              </div>
            </div>
          ))}

          {/* Always mounted, even with nothing to say. Interim text lands and
              clears constantly as words are finalized, and a row that appears
              and disappears took the rest of the list up and down with it. */}
          <div
            className={partialText ? "live-line partial" : "live-line partial empty"}
            title={t("recording.interim")}
            aria-hidden={partialText ? undefined : true}
          >
            <span className="live-avatar" aria-hidden="true">
              …
            </span>
            <div className="tr-body">
              <div className="live-meta">
                <strong>
                  {groups.length === 0
                    ? t("prompt.listening")
                    : t("recording.live")}
                </strong>
              </div>
              <div className="tr-text">
                {partialText || (
                  <span className="muted">
                    {groups.length === 0
                      ? t("recording.appearsWhenSpoken")
                      : "…"}
                  </span>
                )}
                {partialText && (
                  <span className="caret" aria-hidden="true">
                    ▍
                  </span>
                )}
              </div>
            </div>
          </div>
          </div>
        </div>
      </div>
    </div>
  );
}
