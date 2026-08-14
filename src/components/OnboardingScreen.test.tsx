import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { OnboardingScreen } from "./OnboardingScreen";
import { api } from "../api";
import type { PermissionsReport } from "../types";

vi.mock("../api", () => ({
  api: {
    requestMicrophone: vi.fn().mockResolvedValue("granted"),
    requestBrowserAutomation: vi.fn().mockResolvedValue("granted"),
    openPrivacySettings: vi.fn().mockResolvedValue(undefined),
    completeOnboarding: vi.fn().mockResolvedValue({}),
  },
}));

function report(over: Partial<PermissionsReport> = {}): PermissionsReport {
  return {
    onboardingRequired: true,
    steps: ["microphone", "browserDetection"],
    completedVersion: 0,
    currentVersion: 1,
    microphone: "notDetermined",
    browsers: [
      { appName: "Google Chrome", state: "notDetermined" },
      { appName: "Safari", state: "notDetermined" },
    ],
    platform: "macos",
    ...over,
  };
}

beforeEach(() => vi.clearAllMocks());
afterEach(cleanup);

const next = () =>
  fireEvent.click(
    screen.getByRole("button", { name: /get started|continue/i }),
  );

describe("step sequence", () => {
  it("opens on the welcome screen and counts only the permission steps", () => {
    render(<OnboardingScreen report={report()} onFinished={() => {}} />);
    expect(screen.getByText(/welcome to minutes/i)).toBeTruthy();
    // Welcome shows no counter — it is not a step the user is being asked about.
    expect(screen.queryByText(/step \d+ of/i)).toBeNull();

    next();
    // Two permission steps, not four: the welcome and summary bookends are not
    // counted, because nobody thinks of them as steps.
    expect(screen.getByText("Step 1 of 2")).toBeTruthy();
  });

  it("only renders the steps the backend asked for", () => {
    render(
      <OnboardingScreen
        report={report({ steps: ["microphone"] })}
        onFinished={() => {}}
      />,
    );
    next();
    // Heading specifically: "Microphone" is also the status row's label.
    expect(
      screen.getByRole("heading", { name: /^Microphone$/ }),
    ).toBeTruthy();
    next();
    // Straight to the summary: browser and system-audio steps were not listed.
    expect(screen.getByText(/you're ready/i)).toBeTruthy();
  });

  it("can go back to a previous step", () => {
    render(<OnboardingScreen report={report()} onFinished={() => {}} />);
    next();
    next();
    expect(screen.getByText(/meetings opened in a browser/i)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^back$/i }));
    expect(screen.getByText("Step 1 of 2")).toBeTruthy();
  });
});

describe("microphone step", () => {
  it("prompts only when the permission has never been asked", async () => {
    render(<OnboardingScreen report={report()} onFinished={() => {}} />);
    next();
    fireEvent.click(screen.getByRole("button", { name: /allow microphone/i }));
    await waitFor(() => expect(api.requestMicrophone).toHaveBeenCalledTimes(1));
    // The row reflects the new state rather than the stale report.
    expect(screen.getByText(/^Allowed$/)).toBeTruthy();
  });

  it("offers System Settings instead of a dead retry once denied", () => {
    render(
      <OnboardingScreen
        report={report({ microphone: "denied" })}
        onFinished={() => {}}
      />,
    );
    next();
    // macOS will not ask again, so an Allow button here would do nothing.
    expect(screen.queryByRole("button", { name: /allow microphone/i })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: /open system settings/i }));
    expect(api.openPrivacySettings).toHaveBeenCalledWith("microphone");
    expect(screen.getByText(/only asks once/i)).toBeTruthy();
  });
});

describe("browser step", () => {
  it("lists one row per installed browser and grants them individually", async () => {
    render(<OnboardingScreen report={report()} onFinished={() => {}} />);
    next();
    next();

    expect(screen.getByText("Google Chrome")).toBeTruthy();
    expect(screen.getByText("Safari")).toBeTruthy();

    const allows = screen.getAllByRole("button", { name: /^allow$/i });
    expect(allows).toHaveLength(2);
    fireEvent.click(allows[0]);

    await waitFor(() =>
      expect(api.requestBrowserAutomation).toHaveBeenCalledWith("Google Chrome"),
    );
    // Granting Chrome must not silently mark Safari as done.
    await waitFor(() =>
      expect(screen.getAllByRole("button", { name: /^allow$/i })).toHaveLength(1),
    );
  });

  it("says so plainly when no supported browser is installed", () => {
    render(
      <OnboardingScreen report={report({ browsers: [] })} onFinished={() => {}} />,
    );
    next();
    next();
    expect(screen.getByText(/no supported browser was found/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^allow$/i })).toBeNull();
  });

  it("points a denied browser at the Automation pane", () => {
    render(
      <OnboardingScreen
        report={report({
          browsers: [{ appName: "Safari", state: "denied" }],
        })}
        onFinished={() => {}}
      />,
    );
    next();
    next();
    fireEvent.click(screen.getByRole("button", { name: /open system settings/i }));
    expect(api.openPrivacySettings).toHaveBeenCalledWith("automation");
  });
});

describe("opting out", () => {
  it("offers Skip setup on every step and completes when used", async () => {
    const onFinished = vi.fn();
    render(<OnboardingScreen report={report()} onFinished={onFinished} />);

    // Welcome + the two permission steps; the summary is the fourth screen.
    for (let i = 0; i < 3; i++) {
      expect(screen.getByRole("button", { name: /skip setup/i })).toBeTruthy();
      next();
    }
    // Last screen: the escape hatch becomes the finish action itself.
    expect(screen.queryByRole("button", { name: /skip setup/i })).toBeNull();

    cleanup();
    render(<OnboardingScreen report={report()} onFinished={onFinished} />);
    fireEvent.click(screen.getByRole("button", { name: /skip setup/i }));

    // Skipping still stamps the version: someone who declined has made a
    // choice, and re-asking every launch is the ambush this replaces.
    await waitFor(() => expect(api.completeOnboarding).toHaveBeenCalledTimes(1));
    expect(onFinished).toHaveBeenCalled();
  });

  it("still leaves setup when persisting the marker fails", async () => {
    vi.mocked(api.completeOnboarding).mockRejectedValueOnce(new Error("disk"));
    const onFinished = vi.fn();
    render(<OnboardingScreen report={report()} onFinished={onFinished} />);
    fireEvent.click(screen.getByRole("button", { name: /skip setup/i }));
    // A failed write must not trap the user in the wizard.
    await waitFor(() => expect(onFinished).toHaveBeenCalled());
  });
});

describe("summary", () => {
  it("reports what was granted, including nothing at all", () => {
    render(
      <OnboardingScreen
        report={report({
          steps: ["microphone"],
          microphone: "denied",
        })}
        onFinished={() => {}}
      />,
    );
    next();
    next();
    expect(screen.getByText(/you're ready/i)).toBeTruthy();
    expect(screen.getByText(/^Not allowed$/)).toBeTruthy();
  });
});

describe("platform variants", () => {
  it("explains the missing feature instead of hiding it on Windows", () => {
    render(
      <OnboardingScreen
        report={report({
          steps: ["microphone", "detectionUnavailable"],
          microphone: "unknown",
          browsers: [],
          platform: "windows",
        })}
        onFinished={() => {}}
      />,
    );
    next();
    // Windows has no consent dialog, so guidance replaces a request.
    expect(screen.getByText(/windows doesn't ask apps for this/i)).toBeTruthy();
    next();
    expect(screen.getByText(/only available on macos/i)).toBeTruthy();
  });

  it("does not ask Linux for a permission it has no concept of", () => {
    render(
      <OnboardingScreen
        report={report({
          steps: ["detectionUnavailable"],
          browsers: [],
          platform: "linux",
        })}
        onFinished={() => {}}
      />,
    );
    next();
    expect(screen.getByText(/only available on macos/i)).toBeTruthy();
    // No microphone step: a deb/AppImage build has no per-app audio gate.
    expect(
      screen.queryByRole("button", { name: /allow microphone/i }),
    ).toBeNull();
  });
});
