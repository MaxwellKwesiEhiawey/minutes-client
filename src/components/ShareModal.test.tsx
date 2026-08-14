import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { ShareModal } from "./ShareModal";
import type { ShareFormat } from "../types";

afterEach(cleanup);

function props(overrides: Partial<Parameters<typeof ShareModal>[0]> = {}) {
  return {
    hasSummary: true,
    hasTranscript: true,
    shareSupported: true,
    onClose: vi.fn(),
    onCopySummary: vi.fn(),
    onCopyTranscript: vi.fn(),
    onShare: vi.fn(),
    onSave: vi.fn(),
    ...overrides,
  };
}

const sendButton = () =>
  screen.getByRole("button", { name: /Send to an app/ }) as HTMLButtonElement;
const saveButton = () =>
  screen.getByRole("button", { name: /Save to this device/ }) as HTMLButtonElement;
const formatSelect = () => screen.getByLabelText("Format") as HTMLSelectElement;
const toggle = () => screen.getByRole("switch") as HTMLButtonElement;

function chooseFormat(format: ShareFormat) {
  fireEvent.change(formatSelect(), { target: { value: format } });
}

describe("ShareModal destinations", () => {
  it("offers sending to an app and saving to the device", () => {
    render(<ShareModal {...props()} />);
    expect(sendButton()).toBeTruthy();
    expect(saveButton()).toBeTruthy();
  });

  it("hides sending, but not saving, where the platform has no picker", () => {
    render(<ShareModal {...props({ shareSupported: false })} />);
    expect(screen.queryByRole("button", { name: /Send to an app/ })).toBeNull();
    expect(saveButton()).toBeTruthy();
  });

  it("disables both destinations when there is nothing to put in a file", () => {
    render(<ShareModal {...props({ hasSummary: false, hasTranscript: false })} />);
    chooseFormat("pdf");
    expect(sendButton().disabled).toBe(true);
    expect(saveButton().disabled).toBe(true);
    expect(screen.getByText(/no summary or transcript/)).toBeTruthy();
  });

  it("still offers both with a summary but no transcript", () => {
    // Summary-only is a legitimate document.
    render(<ShareModal {...props({ hasTranscript: false })} />);
    chooseFormat("pdf");
    expect(sendButton().disabled).toBe(false);
    expect(saveButton().disabled).toBe(false);
  });

  it("closes after sending and after saving", () => {
    for (const which of ["send", "save"] as const) {
      cleanup();
      const onClose = vi.fn();
      render(<ShareModal {...props({ onClose })} />);
      chooseFormat("pdf");
      fireEvent.click(which === "send" ? sendButton() : saveButton());
      expect(onClose, which).toHaveBeenCalledTimes(1);
    }
  });
});

describe("ShareModal format gate", () => {
  it("starts with no format chosen and both destinations unavailable", () => {
    const onShare = vi.fn();
    const onSave = vi.fn();
    render(<ShareModal {...props({ onShare, onSave })} />);

    expect(formatSelect().value).toBe("");
    expect(sendButton().disabled).toBe(true);
    expect(saveButton().disabled).toBe(true);
    expect(screen.getByText("Choose a format above to send or save.")).toBeTruthy();

    // Nothing can be provoked into firing while the gate is closed.
    fireEvent.click(sendButton());
    fireEvent.click(saveButton());
    expect(onShare).not.toHaveBeenCalled();
    expect(onSave).not.toHaveBeenCalled();
  });

  it("opens both destinations, and drops the hint, once a format is chosen", () => {
    render(<ShareModal {...props()} />);
    chooseFormat("docx");
    expect(sendButton().disabled).toBe(false);
    expect(saveButton().disabled).toBe(false);
    expect(screen.queryByText(/Choose a format above/)).toBeNull();
  });

  it("passes the chosen format to whichever destination is used", () => {
    for (const format of ["pdf", "docx", "md"] as const) {
      for (const which of ["send", "save"] as const) {
        cleanup();
        const onShare = vi.fn();
        const onSave = vi.fn();
        render(<ShareModal {...props({ onShare, onSave })} />);
        chooseFormat(format);
        fireEvent.click(which === "send" ? sendButton() : saveButton());
        const spy = which === "send" ? onShare : onSave;
        expect(spy, `${which} ${format}`).toHaveBeenCalledWith(format, false);
      }
    }
  });
});

describe("ShareModal include-transcript flag", () => {
  it("leaves the transcript out until it is asked for", () => {
    const onShare = vi.fn();
    render(<ShareModal {...props({ onShare })} />);
    expect(toggle().getAttribute("aria-checked")).toBe("false");
    chooseFormat("pdf");
    fireEvent.click(sendButton());
    expect(onShare).toHaveBeenCalledWith("pdf", false);
  });

  it("passes the choice to both destinations once switched on", () => {
    for (const which of ["send", "save"] as const) {
      cleanup();
      const onShare = vi.fn();
      const onSave = vi.fn();
      render(<ShareModal {...props({ onShare, onSave })} />);
      fireEvent.click(toggle());
      expect(toggle().getAttribute("aria-checked")).toBe("true");
      chooseFormat("md");
      fireEvent.click(which === "send" ? sendButton() : saveButton());
      const spy = which === "send" ? onShare : onSave;
      expect(spy, which).toHaveBeenCalledWith("md", true);
    }
  });

  it("says what including or leaving out the transcript means", () => {
    render(<ShareModal {...props()} />);
    expect(screen.getByText(/summary only/)).toBeTruthy();
    fireEvent.click(toggle());
    expect(screen.getByText(/everything that was said/)).toBeTruthy();
  });

  it("locks the flag on when there is no summary to carry the file", () => {
    // Otherwise the file would be a title and a date presented as a share.
    const onSave = vi.fn();
    render(<ShareModal {...props({ hasSummary: false, onSave })} />);
    expect(toggle().disabled).toBe(true);
    expect(toggle().getAttribute("aria-checked")).toBe("true");
    chooseFormat("pdf");
    fireEvent.click(saveButton());
    expect(onSave).toHaveBeenCalledWith("pdf", true);
  });
});
