import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { MeetingPrompt } from "./MeetingPrompt";

const promptPayload = {
  kind: "manual",
  app_name: null,
  process_name: null,
  suggested_title: null,
};

function startCalls() {
  return invokeMock.mock.calls.filter(
    ([cmd]) => cmd === "start_recording_from_prompt",
  );
}

describe("MeetingPrompt double-invoke guard", () => {
  // RTL auto-cleanup needs vitest `globals: true`, which this repo does not
  // enable — clean up renders explicitly so tests don't see stale DOM.
  afterEach(cleanup);

  beforeEach(() => {
    invokeMock.mockReset();
    // The component reads the staging token from the query string.
    window.history.replaceState({}, "", "/?t=1");
    invokeMock.mockImplementation(async (cmd: unknown) => {
      if (cmd === "get_meeting_prompt") return promptPayload;
      // Keep the start pending, like a real recording start in flight.
      if (cmd === "start_recording_from_prompt") return new Promise(() => {});
      return null;
    });
  });

  it("invokes start_recording_from_prompt once for two clicks in the same tick", async () => {
    render(<MeetingPrompt />);
    const button = await screen.findByRole("button", { name: "Start recording" });

    // Two clicks before React re-renders: the async `busy` state cannot block
    // the second one, so only the synchronous ref guard prevents a double
    // start_recording invoke.
    act(() => {
      fireEvent.click(button);
      fireEvent.click(button);
    });

    expect(startCalls()).toHaveLength(1);
  });

  it("invokes start once for rapid repeated Enter presses", async () => {
    render(<MeetingPrompt />);
    const input = await screen.findByLabelText("Meeting title");

    act(() => {
      fireEvent.keyDown(input, { key: "Enter" });
      fireEvent.keyDown(input, { key: "Enter", repeat: true });
      fireEvent.keyDown(input, { key: "Enter" });
    });

    expect(startCalls()).toHaveLength(1);
  });

  it("ignores OS key auto-repeat Enter entirely", async () => {
    render(<MeetingPrompt />);
    const input = await screen.findByLabelText("Meeting title");

    act(() => {
      fireEvent.keyDown(input, { key: "Enter", repeat: true });
    });

    expect(startCalls()).toHaveLength(0);
  });

  it("re-enables start after a failed attempt", async () => {
    let attempts = 0;
    invokeMock.mockImplementation(async (cmd: unknown) => {
      if (cmd === "get_meeting_prompt") return promptPayload;
      if (cmd === "start_recording_from_prompt") {
        attempts += 1;
        if (attempts === 1) throw new Error("device busy");
        return new Promise(() => {});
      }
      return null;
    });

    render(<MeetingPrompt />);
    const button = await screen.findByRole("button", { name: "Start recording" });

    fireEvent.click(button);
    // The error path must release the guard so the user can retry.
    await screen.findByText("device busy");

    fireEvent.click(button);
    await waitFor(() => expect(startCalls()).toHaveLength(2));
  });

  it("does not double-invoke dismiss for two clicks in the same tick", async () => {
    render(<MeetingPrompt />);
    const button = await screen.findByRole("button", { name: "Not now" });

    act(() => {
      fireEvent.click(button);
      fireEvent.click(button);
    });

    const dismissCalls = invokeMock.mock.calls.filter(
      ([cmd]) => cmd === "dismiss_meeting_prompt",
    );
    expect(dismissCalls).toHaveLength(1);
  });
});

describe("MeetingPrompt window chrome", () => {
  it("exposes a drag region so the card can be moved out of the way", async () => {
    const { container } = render(<MeetingPrompt />);
    await screen.findByLabelText("Meeting title");
    const strip = container.querySelector("[data-tauri-drag-region]");
    expect(strip).not.toBeNull();
    // The controls must stay clickable, so the drag region cannot cover them.
    expect(strip!.querySelector("input")).toBeNull();
    expect(container.querySelector(".mp-actions [data-tauri-drag-region]")).toBeNull();
  });

  it("keeps dismiss and start reachable outside the drag region", async () => {
    const { container } = render(<MeetingPrompt />);
    await screen.findByLabelText("Meeting title");
    // Scoped to this render: the suite above does not unmount between tests.
    const labels = [...container.querySelectorAll(".mp-actions .mp-btn")].map(
      (b) => b.textContent,
    );
    expect(labels).toHaveLength(2);
    expect(labels[0]).toMatch(/Not now/);
    expect(labels[1]).toMatch(/Take notes|Start recording/);
  });
});
