import type { Summary } from "../types";

/** Render a summary as Markdown for clipboard copy (summary only, no transcript). */
export function summaryToMarkdown(summary: Summary): string {
  const c = summary.content;
  const lines: string[] = [];
  if (c.title) lines.push(`# ${c.title}`, "");
  if (c.executive_summary) lines.push(c.executive_summary, "");

  if (c.key_topics.length > 0) {
    lines.push("## Key Topics");
    for (const t of c.key_topics) {
      lines.push(`### ${t.topic}`);
      for (const b of t.bullets) lines.push(`- ${b}`);
      lines.push("");
    }
  }

  if (c.decisions.length > 0) {
    lines.push("## Decisions");
    for (const d of c.decisions) {
      lines.push(`- ${d.text}${d.owner ? ` (owner: ${d.owner})` : ""}`);
    }
    lines.push("");
  }

  if (c.action_items.length > 0) {
    lines.push("## Action Items");
    for (const a of c.action_items) {
      const meta = [
        a.assignee ? `assignee: ${a.assignee}` : null,
        a.due ? `due: ${a.due}` : null,
      ].filter(Boolean);
      lines.push(`- [ ] ${a.task}${meta.length ? ` (${meta.join(", ")})` : ""}`);
    }
    lines.push("");
  }

  if (c.open_questions.length > 0) {
    lines.push("## Open Questions");
    for (const q of c.open_questions) lines.push(`- ${q}`);
    lines.push("");
  }

  return lines.join("\n").trim();
}
