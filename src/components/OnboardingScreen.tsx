import { useCallback, useState } from "react";
import { api } from "../api";
import type {
  BrowserPermission,
  OnboardingStepId,
  PermissionsReport,
  PermissionState,
  PrivacyPane,
} from "../types";
import { useT } from "../i18n";
import type { TranslationKey } from "../i18n";
import {
  SIMULATION_IDS,
  SIMULATION_LABELS,
  type SimulationId,
} from "../devOnboarding";

/**
 * First-run setup.
 *
 * Shown once after installation, in the main window, instead of the app shell.
 * It exists because the permissions this app needs used to be requested at the
 * worst possible moment — the microphone prompt on the first Record, Automation
 * consent from whichever background poll ran first, mid-meeting — and a denial
 * of the latter was only ever written to a log file.
 *
 * Two rules shape everything here:
 *
 * - **Nothing is mandatory.** Every step can be skipped and skipping still
 *   completes setup. A wizard that holds the app hostage over a permission is
 *   worse than the ambush it replaces.
 * - **No dead buttons.** macOS asks once per permission; after a denial the
 *   prompt silently never appears again. So a denied step swaps its action for a
 *   deep link into System Settings and says why.
 *
 * The step list is decided by the backend (`permissions.rs`), not here — which
 * steps apply is a platform and install-history question, and duplicating those
 * rules in the UI is how the two drift apart.
 */

export function OnboardingScreen({
  report,
  onFinished,
  simulation,
  onSimulationChange,
}: {
  report: PermissionsReport;
  /** Setup is over (finished or skipped). The marker is already persisted. */
  onFinished: () => void;
  /** Dev-only. When set, this is a synthetic run: the completion marker is
   *  deliberately **not** written, so reviewing the flow cannot change what a
   *  real first launch does. */
  simulation?: SimulationId | null;
  onSimulationChange?: (id: SimulationId) => void;
}) {
  const t = useT();
  // Local mirror of the report: grants update in place as the user makes them,
  // so the Done summary reflects what actually happened.
  const [microphone, setMicrophone] = useState<PermissionState>(
    report.microphone,
  );
  const [browsers, setBrowsers] = useState<BrowserPermission[]>(
    report.browsers,
  );
  const [busy, setBusy] = useState<string | null>(null);
  const [index, setIndex] = useState(0);

  // Welcome first, then the backend's steps, then the summary. The two bookends
  // are always present; only the middle varies by platform.
  const steps: ("welcome" | OnboardingStepId | "done")[] = [
    "welcome",
    ...report.steps,
    "done",
  ];
  const current = steps[index];
  const isLast = index === steps.length - 1;
  const platform = report.platform;

  const finish = useCallback(async () => {
    // A simulated run must leave no trace: stamping the marker here would mean
    // reviewing the flow silently decided a real install had completed it.
    if (!simulation) {
      try {
        await api.completeOnboarding();
      } catch {
        // Persisting the marker failed, which at worst means setup is offered
        // again next launch. Never trap the user in the wizard over it.
      }
    }
    onFinished();
  }, [onFinished, simulation]);

  const openPane = useCallback((pane: PrivacyPane) => {
    // Failure here is not worth a toast on top of the wizard: the copy already
    // names the pane, so a user can navigate there by hand.
    api.openPrivacySettings(pane).catch(() => {});
  }, []);

  async function allowMicrophone() {
    setBusy("microphone");
    try {
      setMicrophone(await api.requestMicrophone());
    } catch {
      setMicrophone("unknown");
    } finally {
      setBusy(null);
    }
  }

  async function allowBrowser(appName: string) {
    setBusy(appName);
    try {
      const state = await api.requestBrowserAutomation(appName);
      setBrowsers((prev) =>
        prev.map((b) => (b.appName === appName ? { ...b, state } : b)),
      );
    } catch {
      setBrowsers((prev) =>
        prev.map((b) =>
          b.appName === appName ? { ...b, state: "unknown" } : b,
        ),
      );
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="ob" role="region" aria-label={t("onboarding.welcomeTitle")}>
      {import.meta.env.DEV && simulation && onSimulationChange && (
        <div className="ob-sim" aria-label="Onboarding simulation (dev only)">
          <span className="ob-sim-tag">Simulating</span>
          {SIMULATION_IDS.map((id) => (
            <button
              key={id}
              type="button"
              className={
                id === simulation ? "ob-sim-btn is-active" : "ob-sim-btn"
              }
              onClick={() => onSimulationChange(id)}
            >
              {SIMULATION_LABELS[id]}
            </button>
          ))}
          <button type="button" className="ob-sim-btn" onClick={onFinished}>
            Exit
          </button>
        </div>
      )}
      <div className="ob-card">
        <header className="ob-head">
          <span className="ob-mark">Minutes</span>
          {current !== "welcome" && !isLast && (
            <span className="ob-progress">
              {t("onboarding.stepOf", {
                // 1-based and counted over the permission steps only: "step 1 of
                // 3" should not include the welcome and summary screens the user
                // does not think of as steps.
                current: String(index),
                total: String(steps.length - 2),
              })}
            </span>
          )}
        </header>

        <div className="ob-body">
          {current === "welcome" && (
            <>
              <h1 className="ob-title">{t("onboarding.welcomeTitle")}</h1>
              <p className="ob-lede">{t("onboarding.welcomeBody")}</p>
              <p className="ob-note">{t("onboarding.welcomeOptional")}</p>
            </>
          )}

          {current === "microphone" && (
            <>
              <h1 className="ob-title">{t("onboarding.microphoneTitle")}</h1>
              <p className="ob-lede">{t("onboarding.microphoneBody")}</p>
              <StateRow label={t("onboarding.microphoneTitle")} state={microphone}>
                {microphone === "notDetermined" && (
                  <button
                    type="button"
                    className="btn primary"
                    disabled={busy === "microphone"}
                    onClick={allowMicrophone}
                  >
                    {t("onboarding.microphoneAllow")}
                  </button>
                )}
                {microphone === "denied" && (
                  <button
                    type="button"
                    className="btn"
                    onClick={() => openPane("microphone")}
                  >
                    {t("onboarding.openSettings")}
                  </button>
                )}
              </StateRow>
              {microphone === "denied" && (
                <p className="ob-note">
                  {t("onboarding.microphoneDeniedHint")}
                </p>
              )}
              {platform === "windows" && (
                <p className="ob-note">
                  {t("onboarding.microphoneWindowsHint")}
                </p>
              )}
            </>
          )}

          {current === "browserDetection" && (
            <>
              <h1 className="ob-title">{t("onboarding.browserTitle")}</h1>
              <p className="ob-lede">{t("onboarding.browserBody")}</p>
              <p className="ob-note">{t("onboarding.browserPrivacy")}</p>
              {browsers.length === 0 ? (
                <p className="ob-note">{t("onboarding.browserNone")}</p>
              ) : (
                <>
                  <p className="ob-note">{t("onboarding.browserPerApp")}</p>
                  <ul className="ob-list">
                    {browsers.map((b) => (
                      <li key={b.appName}>
                        <StateRow label={b.appName} state={b.state}>
                          {b.state !== "granted" && b.state !== "denied" && (
                            <button
                              type="button"
                              className="btn primary"
                              disabled={busy === b.appName}
                              onClick={() => allowBrowser(b.appName)}
                            >
                              {t("onboarding.browserAllow")}
                            </button>
                          )}
                          {b.state === "denied" && (
                            <button
                              type="button"
                              className="btn"
                              onClick={() => openPane("automation")}
                            >
                              {t("onboarding.openSettings")}
                            </button>
                          )}
                        </StateRow>
                      </li>
                    ))}
                  </ul>
                  {browsers.some((b) => b.state === "denied") && (
                    <p className="ob-note">
                      {t("onboarding.browserDeniedHint")}
                    </p>
                  )}
                </>
              )}
            </>
          )}

          {current === "detectionUnavailable" && (
            <>
              <h1 className="ob-title">
                {t("onboarding.detectionUnavailableTitle")}
              </h1>
              <p className="ob-lede">
                {t("onboarding.detectionUnavailableBody")}
              </p>
            </>
          )}

          {current === "done" && (
            <>
              <h1 className="ob-title">{t("onboarding.doneTitle")}</h1>
              <p className="ob-lede">{t("onboarding.doneBody")}</p>
              <ul className="ob-list">
                {report.steps.includes("microphone") && (
                  <li>
                    <StateRow
                      label={t("onboarding.microphoneTitle")}
                      state={microphone}
                    />
                  </li>
                )}
                {report.steps.includes("browserDetection") &&
                  browsers.map((b) => (
                    <li key={b.appName}>
                      <StateRow label={b.appName} state={b.state} />
                    </li>
                  ))}
              </ul>
            </>
          )}
        </div>

        <footer className="ob-foot">
          {index > 0 && !isLast ? (
            <button
              type="button"
              className="btn ghost"
              onClick={() => setIndex((i) => i - 1)}
            >
              {t("onboarding.back")}
            </button>
          ) : (
            <span />
          )}

          <div className="ob-foot-right">
            {/* Present on every screen, including the last, so opting out never
                requires first working out which button is the escape hatch. */}
            {!isLast && (
              <button type="button" className="btn ghost" onClick={finish}>
                {t("onboarding.skipAll")}
              </button>
            )}
            {isLast ? (
              <button type="button" className="btn primary" onClick={finish}>
                {t("onboarding.finish")}
              </button>
            ) : (
              <button
                type="button"
                className="btn primary"
                onClick={() => setIndex((i) => i + 1)}
              >
                {current === "welcome"
                  ? t("onboarding.getStarted")
                  : t("onboarding.continue")}
              </button>
            )}
          </div>
        </footer>
      </div>
    </div>
  );
}

/** Status label for one permission, with its action (if any) on the right. */
function StateRow({
  label,
  state,
  children,
}: {
  label: string;
  state: PermissionState;
  children?: React.ReactNode;
}) {
  const t = useT();
  const key: TranslationKey =
    state === "granted" || state === "notApplicable"
      ? "onboarding.allowed"
      : state === "denied"
        ? "onboarding.notAllowed"
        : "onboarding.notSetUp";
  return (
    <div className="ob-row">
      <div className="ob-row-text">
        <span className="ob-row-label">{label}</span>
        <span className={`ob-pill ob-pill-${stateTone(state)}`}>{t(key)}</span>
      </div>
      {children && <div className="ob-row-action">{children}</div>}
    </div>
  );
}

/** Visual tone per state. `unknown` deliberately reads as neutral, not bad — a
 *  probe that could not answer is not the same as a refusal. */
function stateTone(state: PermissionState): "ok" | "warn" | "idle" {
  if (state === "granted" || state === "notApplicable") return "ok";
  if (state === "denied") return "warn";
  return "idle";
}
