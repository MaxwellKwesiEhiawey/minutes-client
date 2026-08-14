/**
 * Dev-only onboarding simulator.
 *
 * First-run setup is, by design, almost impossible to look at once you have run
 * the app: it shows itself once and then stamps a marker. Worse, the states most
 * worth reviewing cannot be produced on a developer's machine at all — *denied*
 * requires actually refusing a macOS prompt (which macOS then never re-offers),
 * and the Windows and Linux variants cannot be rendered on a Mac.
 *
 * So this hands the wizard a synthetic report. Nothing is probed, no OS dialog is
 * raised, and the real `onboarding_completed_version` is never written, so
 * simulating cannot change what a real launch does.
 *
 * Every export here is behind `import.meta.env.DEV`. `SIMULATIONS` is referenced
 * only from a `import.meta.env.DEV` branch in `App.tsx`, so the bundler drops it
 * from a production build — see `devOnboarding.test.ts`.
 */

import type { PermissionsReport } from "./types";

export type SimulationId =
  | "fresh"
  | "micDenied"
  | "browserDenied"
  | "allGranted"
  | "noBrowsers"
  | "windows"
  | "linux";

/** Shown on the dev switcher. Developer-facing, so deliberately not translated —
 *  same rule the diagnostic strings follow. */
export const SIMULATION_LABELS: Record<SimulationId, string> = {
  fresh: "Fresh install",
  micDenied: "Mic denied",
  browserDenied: "Browser denied",
  allGranted: "All granted",
  noBrowsers: "No browsers",
  windows: "Windows",
  linux: "Linux",
};

export const SIMULATION_IDS = Object.keys(SIMULATION_LABELS) as SimulationId[];

const base: PermissionsReport = {
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
};

export const SIMULATIONS: Record<SimulationId, PermissionsReport> = {
  // What someone actually sees on a brand-new install.
  fresh: base,

  // The state macOS will not let you re-enter by hand: refused, and it will
  // never ask again. The step must offer System Settings, not a dead retry.
  micDenied: { ...base, microphone: "denied" },

  // One browser refused, one still unasked — proves the rows are independent and
  // that granting one does not mark the other done.
  browserDenied: {
    ...base,
    microphone: "granted",
    browsers: [
      { appName: "Google Chrome", state: "denied" },
      { appName: "Safari", state: "notDetermined" },
    ],
  },

  // A reinstall inheriting every grant: the walkthrough still runs, and every
  // row should read "Allowed".
  allGranted: {
    ...base,
    microphone: "granted",
    browsers: [
      { appName: "Google Chrome", state: "granted" },
      { appName: "Safari", state: "granted" },
    ],
  },

  // Nothing supported installed — the step has to say so rather than show an
  // empty list.
  noBrowsers: { ...base, browsers: [] },

  // Shortened flows that cannot be rendered on this platform for real.
  windows: {
    ...base,
    steps: ["microphone", "detectionUnavailable"],
    microphone: "unknown",
    browsers: [],
    platform: "windows",
  },
  linux: {
    ...base,
    steps: ["detectionUnavailable"],
    microphone: "notApplicable",
    browsers: [],
    platform: "linux",
  },
};

export function simulationReport(id: SimulationId): PermissionsReport {
  return SIMULATIONS[id];
}
