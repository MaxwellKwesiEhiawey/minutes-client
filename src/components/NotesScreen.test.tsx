import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { NotesScreen } from "./NotesScreen";
import type { MeetingListItem } from "../types";

afterEach(cleanup);

function makeMeeting(overrides: Partial<MeetingListItem> = {}): MeetingListItem {
  return {
    id: "m1",
    title: "Weekly sync",
    status: "completed",
    created_at: "2026-08-11T09:30:26.000Z",
    ended_at: "2026-08-11T10:01:40.000Z",
    segment_count: 408,
    has_summary: true,
    ...overrides,
  };
}

function makeProps(overrides: Partial<Parameters<typeof NotesScreen>[0]> = {}) {
  return {
    meetings: [makeMeeting()],
    searchResults: null,
    searchQuery: "",
    onSearchChange: vi.fn(),
    selectedId: null,
    recordingId: null,
    onOpen: vi.fn(),
    onDelete: vi.fn(),
    onNewMeeting: vi.fn(),
    ...overrides,
  };
}

describe("NotesScreen rows", () => {
  it("labels an interrupted meeting", () => {
    render(
      <NotesScreen
        {...makeProps({ meetings: [makeMeeting({ status: "interrupted" })] })}
      />,
    );
    expect(screen.getByText(/interrupted/i)).toBeTruthy();
  });

  it("labels the meeting being recorded", () => {
    render(<NotesScreen {...makeProps({ recordingId: "m1" })} />);
    expect(screen.getByText(/recording/i)).toBeTruthy();
  });

  it("does not surface the raw segment count", () => {
    render(<NotesScreen {...makeProps()} />);
    expect(screen.queryByText(/segment/i)).toBeNull();
    expect(screen.queryByText("408")).toBeNull();
  });

  it("shows date and duration cells", () => {
    const { container } = render(<NotesScreen {...makeProps()} />);
    const cells = container.querySelectorAll(".tbl-row .tbl-cell");
    // Date, duration, summary.
    expect(cells.length).toBe(3);
    expect(cells[1].textContent).toBe("00:31:14");
  });

  it("opens a meeting when its row is activated", () => {
    const onOpen = vi.fn();
    const { container } = render(<NotesScreen {...makeProps({ onOpen })} />);
    fireEvent.click(container.querySelector(".tbl-row")!);
    expect(onOpen).toHaveBeenCalledWith("m1");
  });

  it("offers delete from the row menu, disabled while recording", () => {
    const { container } = render(
      <NotesScreen {...makeProps({ recordingId: "m1" })} />,
    );
    fireEvent.click(container.querySelector(".row-menu-btn")!);
    const del = screen.getByRole("menuitem", { name: /delete/i });
    expect(del).toBeTruthy();
    expect((del as HTMLButtonElement).disabled).toBe(true);
  });

  it("renders the search snippet for a transcript hit", () => {
    render(
      <NotesScreen
        {...makeProps({
          searchQuery: "budget",
          searchResults: [{ ...makeMeeting(), snippet: "the budget review" }],
        })}
      />,
    );
    // The query is highlighted inside the snippet.
    expect(screen.getByText("budget").tagName).toBe("MARK");
    expect(screen.getByText(/review/)).toBeTruthy();
  });

  it("offers a way out of an empty search", () => {
    const onSearchChange = vi.fn();
    render(
      <NotesScreen
        {...makeProps({
          searchQuery: "nothing",
          searchResults: [],
          onSearchChange,
        })}
      />,
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Clear search" }),
    );
    expect(onSearchChange).toHaveBeenCalledWith("");
  });
});
