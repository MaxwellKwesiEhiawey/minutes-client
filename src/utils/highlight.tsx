import type { ReactNode } from "react";

/**
 * Wrap every case-insensitive occurrence of `query` within `text` in `<mark>`.
 * Returns the original string when the query is empty. Shared by the sidebar
 * search snippets and the transcript view so the two can't drift apart.
 */
export function highlight(text: string, query: string): ReactNode {
  const q = query.trim();
  if (!q) return text;
  const lower = text.toLowerCase();
  const needle = q.toLowerCase();
  const parts: ReactNode[] = [];
  let from = 0;
  let idx = lower.indexOf(needle, from);
  while (idx !== -1) {
    if (idx > from) parts.push(text.slice(from, idx));
    parts.push(<mark key={idx}>{text.slice(idx, idx + needle.length)}</mark>);
    from = idx + needle.length;
    idx = lower.indexOf(needle, from);
  }
  if (from < text.length) parts.push(text.slice(from));
  return parts;
}
