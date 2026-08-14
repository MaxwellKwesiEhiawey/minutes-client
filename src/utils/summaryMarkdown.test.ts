import { describe, it, expect } from "vitest";
import { summaryToMarkdown } from "./summaryMarkdown";
import type { Summary } from "../types";

function makeSummary(overrides: Partial<Summary["content"]> = {}): Summary {
  return {
    meeting_id: "m1",
    model: "claude-x",
    created_at: "2026-07-05T10:00:00Z",
    content: {
      title: "Weekly Sync",
      executive_summary: "We aligned on the roadmap.",
      key_topics: [],
      decisions: [],
      action_items: [],
      open_questions: [],
      ...overrides,
    },
  };
}

describe("summaryToMarkdown", () => {
  it("renders title and executive summary", () => {
    const md = summaryToMarkdown(makeSummary());
    expect(md).toContain("# Weekly Sync");
    expect(md).toContain("We aligned on the roadmap.");
  });

  it("omits empty sections", () => {
    const md = summaryToMarkdown(makeSummary());
    expect(md).not.toContain("## Key Topics");
    expect(md).not.toContain("## Decisions");
    expect(md).not.toContain("## Action Items");
    expect(md).not.toContain("## Open Questions");
  });

  it("renders key topics with bullets", () => {
    const md = summaryToMarkdown(
      makeSummary({
        key_topics: [{ topic: "Roadmap", bullets: ["Q3 launch", "hiring"] }],
      }),
    );
    expect(md).toContain("## Key Topics");
    expect(md).toContain("### Roadmap");
    expect(md).toContain("- Q3 launch");
    expect(md).toContain("- hiring");
  });

  it("renders decisions with optional owner", () => {
    const md = summaryToMarkdown(
      makeSummary({
        decisions: [
          { text: "Ship v2", owner: "Ada" },
          { text: "Freeze scope", owner: null },
        ],
      }),
    );
    expect(md).toContain("- Ship v2 (owner: Ada)");
    expect(md).toContain("- Freeze scope");
    expect(md).not.toContain("Freeze scope (owner:");
  });

  it("renders action items with assignee and due metadata", () => {
    const md = summaryToMarkdown(
      makeSummary({
        action_items: [
          { task: "Draft spec", assignee: "Lin", due: "Fri" },
          { task: "Book room", assignee: null, due: null },
        ],
      }),
    );
    expect(md).toContain("- [ ] Draft spec (assignee: Lin, due: Fri)");
    expect(md).toContain("- [ ] Book room");
    expect(md).not.toContain("Book room (");
  });

  it("renders open questions", () => {
    const md = summaryToMarkdown(
      makeSummary({ open_questions: ["Who owns billing?"] }),
    );
    expect(md).toContain("## Open Questions");
    expect(md).toContain("- Who owns billing?");
  });
});
