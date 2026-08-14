import type { Summary } from "../types";
import { shortModelName } from "../utils/modelName";
import { IconCheck, IconSparkle } from "./Icons";
import { useT } from "../i18n";

/**
 * Renders a generated summary in the design's Overview / Key discussion points
 * / Decisions / Action items / Open questions shape. Read-only — the summary is
 * produced by the backend and regenerated, never edited in place.
 */
export function SummaryView({ summary }: { summary: Summary }) {
  const t = useT();
  const c = summary.content;

  return (
    <div className="summary">
      <div className="ai-note">
        <IconSparkle size={15} />
        <span>{t("summary.aiNote")}</span>
      </div>

      {c.executive_summary && (
        <>
          <h3 className="sum-h3">{t("summary.overview")}</h3>
          <p className="sum-p">{c.executive_summary}</p>
        </>
      )}

      {c.key_topics.length > 0 && (
        <div className="sum-block">
          <h3 className="sum-h3">{t("summary.keyPoints")}</h3>
          {/* Named `topic`, not `t` — that would shadow the translator. */}
          {c.key_topics.map((topic, i) => (
            <div key={i} className="sum-block">
              {topic.topic && (
                <div className="point-group-name">{topic.topic}</div>
              )}
              <div className="points">
                {topic.bullets.map((b, j) => (
                  <div key={j} className="point">
                    <span className="point-dot" aria-hidden="true" />
                    <span>{b}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {c.decisions.length > 0 && (
        <div className="sum-block">
          <h3 className="sum-h3">{t("summary.decisions")}</h3>
          {c.decisions.map((d, i) => (
            <div key={i} className="decision">
              <IconCheck size={17} />
              <span>
                {d.text}
                {d.owner && (
                  <span className="tag">
                    {t("summary.owner", { name: d.owner })}
                  </span>
                )}
              </span>
            </div>
          ))}
        </div>
      )}

      {c.action_items.length > 0 && (
        <div className="sum-block">
          <h3 className="sum-h3">{t("summary.actionItems")}</h3>
          {c.action_items.map((a, i) => (
            <div key={i} className="action-row">
              <span className="action-box" aria-hidden="true" />
              <div className="action-main">
                <div className="action-task">{a.task}</div>
                {(a.assignee || a.due) && (
                  <div className="action-meta">
                    {a.assignee && <span>{t("summary.assignedTo", { name: a.assignee })}</span>}
                    {a.due && <span>{t("summary.due", { date: a.due })}</span>}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {c.open_questions.length > 0 && (
        <div className="sum-block">
          <h3 className="sum-h3">{t("summary.openQuestions")}</h3>
          {c.open_questions.map((q, i) => (
            <div key={i} className="quote">
              <div className="quote-label">{t("summary.openQuestion")}</div>
              <div className="quote-text">{q}</div>
            </div>
          ))}
        </div>
      )}

      <p className="sum-foot" title={summary.model}>
        {t("summary.generatedBy", {
          model: shortModelName(summary.model),
          date: new Date(summary.created_at).toLocaleString(),
        })}
      </p>
    </div>
  );
}
