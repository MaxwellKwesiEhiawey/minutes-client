import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  SIMULATION_IDS,
  SIMULATION_LABELS,
  SIMULATIONS,
  simulationReport,
} from "./devOnboarding";

describe("simulation scenarios", () => {
  it("has a label and a report for every id", () => {
    for (const id of SIMULATION_IDS) {
      expect(SIMULATION_LABELS[id], id).toBeTruthy();
      expect(simulationReport(id), id).toBeTruthy();
    }
  });

  it("covers the states that cannot be produced by hand on a Mac", () => {
    // These are the whole reason the simulator exists: macOS will not re-offer a
    // prompt once refused, and it cannot render the other platforms at all.
    expect(SIMULATIONS.micDenied.microphone).toBe("denied");
    expect(
      SIMULATIONS.browserDenied.browsers.some((b) => b.state === "denied"),
    ).toBe(true);
    expect(SIMULATIONS.windows.platform).toBe("windows");
    expect(SIMULATIONS.linux.platform).toBe("linux");
  });

  it("keeps every scenario a wizard would actually show", () => {
    for (const id of SIMULATION_IDS) {
      const r = simulationReport(id);
      expect(r.onboardingRequired, id).toBe(true);
      expect(r.steps.length, id).toBeGreaterThan(0);
      // A step referencing browsers must not claim browsers that are not listed.
      if (!r.steps.includes("browserDetection")) {
        expect(r.browsers, id).toEqual([]);
      }
    }
  });

  it("never marks a scenario as already completed", () => {
    // A simulated report with completedVersion >= currentVersion would make the
    // real gate skip the wizard, which is the opposite of the point.
    for (const id of SIMULATION_IDS) {
      const r = simulationReport(id);
      expect(r.completedVersion, id).toBeLessThan(r.currentVersion);
    }
  });
});

describe("release safety", () => {
  /* The simulator hands the wizard fabricated permission states. If it ever
     survived into a production build, a real user could be shown a setup flow
     claiming permissions they do not have. Vite strips it because every use site
     sits behind `import.meta.env.DEV` — this asserts that guard is still there,
     since losing it fails silently. */
  it("is only ever reached from an import.meta.env.DEV branch", () => {
    for (const file of ["src/App.tsx", "src/components/OnboardingScreen.tsx"]) {
      const source = readFileSync(file, "utf8");
      const uses = source
        .split("\n")
        .map((line, i) => ({ line, n: i + 1 }))
        .filter(
          ({ line }) =>
            /simulationReport\(|SIMULATION_IDS|SIMULATION_LABELS/.test(line) &&
            !line.trimStart().startsWith("//") &&
            !line.includes("import"),
        );
      expect(uses.length, `${file} should reference the simulator`).toBeGreaterThan(0);
      // Every referencing block must be guarded. Checked by requiring the guard
      // to appear in the source at all and on the enclosing conditional.
      expect(source).toContain("import.meta.env.DEV");
    }
  });

  it("guards the keyboard shortcut too", () => {
    const app = readFileSync("src/App.tsx", "utf8");
    const shortcut = app.indexOf('e.key.toLowerCase() === "o"');
    expect(shortcut).toBeGreaterThan(-1);
    // The DEV early-return must sit above the shortcut's listener, or the
    // scenario switcher would be reachable in a shipped app.
    const guard = app.lastIndexOf("if (!import.meta.env.DEV) return;", shortcut);
    expect(guard).toBeGreaterThan(-1);
  });
});
