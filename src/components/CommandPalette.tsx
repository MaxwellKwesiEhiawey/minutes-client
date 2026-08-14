import type { MeetingListItem, MeetingSearchHit } from "../types";
import { formatDate } from "../utils/format";
import { IconSearch } from "./Icons";
import { Modal } from "./Modal";
import { useT } from "../i18n";

interface Props {
  query: string;
  onQueryChange: (value: string) => void;
  /** Backend search hits when a query is active, else null (not searching). */
  searchResults: MeetingSearchHit[] | null;
  /** Full list, shown as "Recent" before anything is typed. */
  meetings: MeetingListItem[];
  onOpen: (id: string) => void;
  onClose: () => void;
}

const MAX_PER_GROUP = 6;

/**
 * ⌘K search surface. It renders the same backend search that drives the My
 * Notes list — title matches and transcript-snippet matches — as two groups.
 */
export function CommandPalette({
  query,
  onQueryChange,
  searchResults,
  meetings,
  onOpen,
  onClose,
}: Props) {
  const t = useT();
  const trimmed = query.trim();
  const searching = searchResults !== null;

  const titleHits = searching
    ? searchResults.filter((h) => !h.snippet).slice(0, MAX_PER_GROUP)
    : [];
  const transcriptHits = searching
    ? searchResults.filter((h) => h.snippet).slice(0, MAX_PER_GROUP)
    : [];
  const recents = searching ? [] : meetings.slice(0, MAX_PER_GROUP);

  const empty =
    searching && titleHits.length === 0 && transcriptHits.length === 0;

  function open(id: string) {
    onOpen(id);
    onClose();
  }

  return (
    <Modal label={t("palette.label")} className="palette" onClose={onClose}>
      <div className="palette-head">
        <IconSearch size={18} />
        {/* Autofocus is the point of a command palette: it opens on ⌘K to be
            typed into immediately, and Escape closes it. */}
        <input
          autoFocus
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          placeholder={t("palette.placeholder")}
          aria-label={t("topbar.searchLabel")}
        />
        <span className="kbd" aria-hidden="true">
          ESC
        </span>
      </div>

      <div className="palette-body">
        {recents.length > 0 && (
          <div className="palette-group">
            <div className="palette-group-label">{t("palette.recent")}</div>
            {recents.map((m) => (
              <button
                type="button"
                key={m.id}
                className="palette-item"
                onClick={() => open(m.id)}
              >
                <span
                  className="palette-dot"
                  style={{ background: "var(--accent)" }}
                />
                <span className="palette-item-main">
                  <span className="palette-item-title">{m.title}</span>
                  <span className="palette-item-sub">
                    {formatDate(m.created_at)}
                  </span>
                </span>
              </button>
            ))}
          </div>
        )}

        {titleHits.length > 0 && (
          <div className="palette-group">
            <div className="palette-group-label">{t("palette.meetings")}</div>
            {titleHits.map((h) => (
              <button
                type="button"
                key={h.id}
                className="palette-item"
                onClick={() => open(h.id)}
              >
                <span
                  className="palette-dot"
                  style={{ background: "var(--accent)" }}
                />
                <span className="palette-item-main">
                  <span className="palette-item-title">{h.title}</span>
                  <span className="palette-item-sub">
                    {formatDate(h.created_at)}
                  </span>
                </span>
              </button>
            ))}
          </div>
        )}

        {transcriptHits.length > 0 && (
          <div className="palette-group">
            <div className="palette-group-label">{t("palette.transcripts")}</div>
            {transcriptHits.map((h) => (
              <button
                type="button"
                key={`t-${h.id}`}
                className="palette-item"
                onClick={() => open(h.id)}
              >
                <span
                  className="palette-dot"
                  style={{ background: "var(--info)" }}
                />
                <span className="palette-item-main">
                  <span className="palette-item-title">{h.snippet}</span>
                  <span className="palette-item-sub">{h.title}</span>
                </span>
                <span className="palette-item-meta">
                  {formatDate(h.created_at)}
                </span>
              </button>
            ))}
          </div>
        )}

        {empty && (
          <div className="palette-empty">
            <strong>{t("palette.noResults", { query: trimmed })}</strong>
            <p>{t("palette.noResultsHint")}</p>
          </div>
        )}

        {!searching && recents.length === 0 && (
          <div className="palette-empty">
            <strong>{t("palette.nothingYet")}</strong>
            <p>{t("palette.nothingYetHint")}</p>
          </div>
        )}
      </div>
    </Modal>
  );
}
