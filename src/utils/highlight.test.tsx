import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import { highlight } from "./highlight";

function renderNodes(nodes: ReturnType<typeof highlight>) {
  const { container } = render(<>{nodes}</>);
  return container;
}

describe("highlight", () => {
  it("returns the original text unchanged when the query is empty", () => {
    expect(highlight("Weekly sync", "")).toBe("Weekly sync");
    expect(highlight("Weekly sync", "   ")).toBe("Weekly sync");
  });

  it("wraps a single case-insensitive match in <mark>", () => {
    const container = renderNodes(highlight("Weekly Sync Notes", "sync"));
    const marks = container.querySelectorAll("mark");
    expect(marks).toHaveLength(1);
    expect(marks[0].textContent).toBe("Sync");
    expect(container.textContent).toBe("Weekly Sync Notes");
  });

  it("wraps every occurrence when the query repeats", () => {
    const container = renderNodes(highlight("ababab", "ab"));
    const marks = container.querySelectorAll("mark");
    expect(marks).toHaveLength(3);
    expect(container.textContent).toBe("ababab");
  });

  it("handles a match at the very start and end of the string", () => {
    const container = renderNodes(highlight("catscat", "cat"));
    const marks = container.querySelectorAll("mark");
    expect(marks).toHaveLength(2);
    expect(marks[0].textContent).toBe("cat");
    expect(marks[1].textContent).toBe("cat");
  });

  it("returns the text untouched when there is no match", () => {
    const container = renderNodes(highlight("Weekly sync", "standup"));
    expect(container.querySelectorAll("mark")).toHaveLength(0);
    expect(container.textContent).toBe("Weekly sync");
  });

  it("trims whitespace from the query before matching", () => {
    const container = renderNodes(highlight("Weekly sync", "  sync  "));
    expect(container.querySelectorAll("mark")).toHaveLength(1);
  });

  it("does not match across unicode case-folding surprises but handles accented text safely", () => {
    // Non-ASCII input shouldn't throw, even if it doesn't "smart match".
    const container = renderNodes(highlight("Café standup", "café"));
    expect(container.textContent).toBe("Café standup");
  });
});
