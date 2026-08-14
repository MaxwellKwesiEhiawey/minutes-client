import { useEffect, useRef, useState } from "react";
import type { MeetingListItem, MeetingSearchHit } from "../types";
import {
  formatDate,
  formatDuration,
  meetingDurationMs,
  statusBadgeClass,
} from "../utils/format";
import { IconDots, IconOpen, IconSearch, IconTrash } from "./Icons";
import { highlight } from "../utils/highlight";
import { useT } from "../i18n";
import { useStatusLabel } from "../utils/statusLabel";

interface Props {
  meetings: MeetingListItem[];
  /** Backend search hits when a query is active, else null (not searching). */
  searchResults: MeetingSearchHit[] | null;
  searchQuery: string;
  onSearchChange: (value: string) => void;
  selectedId: string | null;
  recordingId: string | null;
  onOpen: (id: string) => void;
  onDelete: (id: string) => void;
  onNewMeeting: () => void;
}

export function NotesScreen({
  meetings,
  searchResults,
  searchQuery,
  onSearchChange,
  selectedId,
  recordingId,
  onOpen,
  onDelete,
  onNewMeeting,
}: Props) {
  const t = useT();
  const statusLabel = useStatusLabel();
  const [menuFor, setMenuFor] = useState<string | null>(null);
  const tableRef = useRef<HTMLDivElement>(null);

  const searching = searchResults !== null;
  const query = searchQuery.trim();
  const rows: MeetingListItem[] = searching ? searchResults : meetings;

  const snippetFor = (id: string): string | null =>
    searching ? (searchResults.find((h) => h.id === id)?.snippet ?? null) : null;

  // Close the row menu on any click outside the table and on Escape.
  useEffect(() => {
    if (!menuFor) return;
    const onDown = (e: MouseEvent) => {
      if (!tableRef.current?.contains(e.target as Node)) setMenuFor(null);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setMenuFor(null);
    };
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [menuFor]);

  return (
    <div className="screen screen-notes">
      <h2 className="screen-title">{t("notes.title")}</h2>
      <p className="screen-sub">
        {searching ? t("notes.results", { query }) : t("notes.sub")}
      </p>

      <div className="notes-toolbar">
        <div className="field">
          <IconSearch size={16} />
          <input
            placeholder={t("topbar.search")}
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            aria-label={t("topbar.searchLabel")}
          />
          {query && (
            <button
              type="button"
              className="field-clear"
              onClick={() => onSearchChange("")}
              aria-label={t("notes.clearSearchText")}
            >
              ×
            </button>
          )}
        </div>
      </div>

      {rows.length > 0 ? (
        <div className="table-card" ref={tableRef}>
          <div className="tbl-head" aria-hidden="true">
            <span>{t("notes.colMeeting")}</span>
            <span>{t("notes.colDate")}</span>
            <span>{t("notes.colDuration")}</span>
            <span>{t("notes.colSummary")}</span>
            <span>{t("notes.colStatus")}</span>
            <span />
          </div>
          {rows.map((m) => {
            const isRec = m.id === recordingId;
            const snippet = snippetFor(m.id);
            return (
              <div
                key={m.id}
                className={`tbl-row ${m.id === selectedId ? "active" : ""}`}
                role="button"
                tabIndex={0}
                aria-current={m.id === selectedId ? "true" : undefined}
                aria-label={`${m.title}, ${statusLabel(m.status, isRec)}`}
                onClick={() => onOpen(m.id)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onOpen(m.id);
                  }
                }}
              >
                <div className="tbl-main">
                  <div className="tbl-title">{m.title}</div>
                  {snippet && (
                    <div className="tbl-sub">{highlight(snippet, query)}</div>
                  )}
                </div>
                <span className="tbl-cell">{formatDate(m.created_at)}</span>
                <span className="tbl-cell">
                  {formatDuration(
                    meetingDurationMs(m.created_at, m.ended_at, isRec),
                  )}
                </span>
                <span className="tbl-cell">
                  {m.has_summary ? t("common.yes") : t("common.none")}
                </span>
                <span className={`tbl-status ${statusBadgeClass(m.status, isRec)}`}>
                  {statusLabel(m.status, isRec)}
                </span>
                <button
                  type="button"
                  className="row-menu-btn"
                  aria-label={t("notes.moreActions", { title: m.title })}
                  aria-expanded={menuFor === m.id}
                  onClick={(e) => {
                    e.stopPropagation();
                    setMenuFor((cur) => (cur === m.id ? null : m.id));
                  }}
                >
                  <IconDots size={16} />
                </button>

                {menuFor === m.id && (
                  <div className="row-menu" role="menu">
                    <button
                      type="button"
                      role="menuitem"
                      className="row-menu-item"
                      onClick={(e) => {
                        e.stopPropagation();
                        setMenuFor(null);
                        onOpen(m.id);
                      }}
                    >
                      <IconOpen size={15} />
                      {t("common.open")}
                    </button>
                    <button
                      type="button"
                      role="menuitem"
                      className="row-menu-item danger"
                      disabled={isRec}
                      title={isRec ? t("notes.stopBeforeDelete") : undefined}
                      onClick={(e) => {
                        e.stopPropagation();
                        setMenuFor(null);
                        onDelete(m.id);
                      }}
                    >
                      <IconTrash size={15} />
                      {t("common.delete")}
                    </button>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      ) : (
        <div className="empty-panel">
          <h3>
            {searching ? t("notes.emptySearchTitle") : t("notes.emptyTitle")}
          </h3>
          <p>
            {searching ? t("notes.emptySearchBody") : t("notes.emptyBody")}
          </p>
          <div className="empty-actions">
            {searching ? (
              <button
                type="button"
                className="btn"
                onClick={() => onSearchChange("")}
              >
                {t("notes.clearSearch")}
              </button>
            ) : (
              <button type="button" className="btn primary" onClick={onNewMeeting}>
                {t("nav.newMeeting")}
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
