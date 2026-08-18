import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type {
  AudioDevicesResponse,
  InstalledModelsInfo,
  ModelProgressEvent,
  ServerStatus,
  SettingsView,
  TranscriptionEngine,
  TranscriptionStatus,
} from "../types";
import { api, events } from "../api";
import {
  getThemePreference,
  setThemePreference,
  type ThemePreference,
} from "../theme";
import {
  getReadingPrefs,
  setReadingPrefs,
  type ReadingPrefs,
} from "../readingPrefs";
import { LANGUAGES } from "../languages";
import { normalizeError } from "../utils/errors";
import {
  WHISPER_MODELS,
  installedModelLabelKey,
  whisperModelSizeLabel,
} from "../utils/whisperModels";
import { serverUrlProblem } from "../utils/serverUrl";
import { LOCALES, LOCALE_NAMES, useI18n, type TranslationKey } from "../i18n";

interface Props {
  current: SettingsView;
  onClose: () => void;
  onSaved: (s: SettingsView) => void;
  /** Re-run first-run setup. Lives on the Call detection tab where that exists,
   *  and on Audio otherwise — Call detection hides itself off macOS, so putting
   *  the only entry point there would strand Windows and Linux users. */
  onRerunOnboarding: () => void;
}

type TabId =
  | "language"
  | "appearance"
  | "reading"
  | "audio"
  | "call-detection"
  | "transcription"
  | "summary"
  | "privacy"
  | "advanced";

/** Tab order and copy. Labels and blurbs are keys, resolved at render time so
 *  the rail relabels itself when the language changes. */
const TABS: { id: TabId; label: TranslationKey; blurb: TranslationKey }[] = [
  // First: someone who cannot read the current language comes here to fix that,
  // so the word "Language" has to be visible in the rail itself.
  {
    id: "language",
    label: "settings.tab.languageRegion",
    blurb: "settings.blurb.languageRegion",
  },
  {
    id: "appearance",
    label: "settings.tab.appearance",
    blurb: "settings.blurb.appearance",
  },
  {
    id: "reading",
    label: "settings.tab.reading",
    blurb: "settings.blurb.reading",
  },
  { id: "audio", label: "settings.tab.audio", blurb: "settings.blurb.audio" },
  {
    id: "call-detection",
    label: "settings.tab.callDetection",
    blurb: "settings.blurb.callDetection",
  },
  {
    id: "transcription",
    label: "settings.tab.transcription",
    blurb: "settings.blurb.transcription",
  },
  {
    id: "summary",
    label: "settings.tab.summary",
    blurb: "settings.blurb.summary",
  },
  {
    id: "privacy",
    label: "settings.tab.privacy",
    blurb: "settings.blurb.privacy",
  },
  {
    id: "advanced",
    label: "settings.tab.advanced",
    blurb: "settings.blurb.advanced",
  },
];

/* A settings row: label + hint on the left, control on the right. `stack`
   drops the control onto its own full-width line (textareas, long copy). */
function Row({
  label,
  hint,
  htmlFor,
  stack,
  children,
}: {
  label: string;
  hint?: React.ReactNode;
  htmlFor?: string;
  stack?: boolean;
  children?: React.ReactNode;
}) {
  return (
    <div className={stack ? "st-row st-row-stack" : "st-row"}>
      <div className="st-row-text">
        {htmlFor ? (
          <label className="st-row-label" htmlFor={htmlFor}>
            {label}
          </label>
        ) : (
          <span className="st-row-label">{label}</span>
        )}
        {hint && <div className="st-row-hint">{hint}</div>}
      </div>
      {children && <div className="st-row-control">{children}</div>}
    </div>
  );
}

function Toggle({
  id,
  checked,
  onChange,
  disabled,
  label,
}: {
  id: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
  label: string;
}) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      className={checked ? "st-toggle on" : "st-toggle"}
      onClick={() => onChange(!checked)}
    >
      <span className="st-toggle-knob" />
    </button>
  );
}

/**
 * Settings as a full screen: a tab rail beside one card, matching the design.
 * Changes apply as they are made — each edit to a server-backed field debounces
 * into the same `api.saveSettings` call the old modal's Save button used.
 */
export function SettingsScreen({
  current,
  onClose,
  onSaved,
  onRerunOnboarding,
}: Props) {
  const { t, locale, setLocale } = useI18n();
  const [tab, setTab] = useState<TabId>("language");

  const [serverUrl, setServerUrl] = useState(current.server_url);
  const [transcriptionEngine, setTranscriptionEngine] =
    useState<TranscriptionEngine>(current.transcription_engine ?? "deepgram");
  const [whisperModel, setWhisperModel] = useState(current.whisper_model);
  const [diarizationEnabled, setDiarizationEnabled] = useState(
    current.diarization_enabled,
  );
  const [exportMarkdown, setExportMarkdown] = useState(current.export_markdown);
  const [startAtLogin, setStartAtLogin] = useState(current.start_at_login);
  const [anthropicModel, setAnthropicModel] = useState(current.anthropic_model);
  const [chunkSecs, setChunkSecs] = useState(current.chunk_secs);
  const [partialSecs, setPartialSecs] = useState(current.partial_secs);
  const [modelStatus, setModelStatus] = useState<TranscriptionStatus | null>(
    null,
  );
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] =
    useState<ModelProgressEvent | null>(null);
  const [installedModels, setInstalledModels] =
    useState<InstalledModelsInfo | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [recordingActive, setRecordingActive] = useState(false);
  const [summaryInstructions, setSummaryInstructions] = useState(
    current.summary_instructions ?? "",
  );
  const [transcriptionLanguage, setTranscriptionLanguage] = useState(
    current.transcription_language ?? "",
  );
  const [summaryLanguage, setSummaryLanguage] = useState(
    current.summary_language ?? "",
  );
  const [autoSummarize, setAutoSummarize] = useState(
    current.auto_summarize ?? true,
  );
  const [captureMicrophone, setCaptureMicrophone] = useState(
    current.capture_microphone ?? true,
  );
  const [inputDevice, setInputDevice] = useState(current.input_device ?? "");
  const [captureSystemAudio, setCaptureSystemAudio] = useState(
    current.capture_system_audio ?? true,
  );
  const [systemAudioDevice, setSystemAudioDevice] = useState(
    current.system_audio_device ?? "",
  );
  const [callDetectionEnabled, setCallDetectionEnabled] = useState(
    current.call_detection_enabled ?? false,
  );
  const [telemetryEnabled, setTelemetryEnabled] = useState(
    current.telemetry_enabled ?? true,
  );
  const [callDetectionCooldown, setCallDetectionCooldown] = useState(
    current.call_detection_cooldown_minutes ?? 5,
  );
  const [audioDevices, setAudioDevices] = useState<AudioDevicesResponse>({
    platform: "unknown",
    devices: [],
    has_loopback: false,
  });
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);
  const [checking, setChecking] = useState(true);
  const [saveState, setSaveState] = useState<"idle" | "saving" | "saved">(
    "idle",
  );
  const [err, setErr] = useState<string | null>(null);
  const [theme, setTheme] = useState<ThemePreference>(getThemePreference);
  const [reading, setReading] = useState<ReadingPrefs>(getReadingPrefs);

  function updateReading(patch: Partial<ReadingPrefs>) {
    const next = { ...reading, ...patch };
    setReading(next);
    setReadingPrefs(next);
  }

  useEffect(() => {
    api
      .listAudioDevices()
      .then(setAudioDevices)
      .catch(() =>
        setAudioDevices({
          platform: "unknown",
          devices: [],
          has_loopback: false,
        }),
      );
  }, []);

  const loopbackDevices = useMemo(
    () => audioDevices.devices.filter((d) => d.kind === "loopback"),
    [audioDevices.devices],
  );
  // A microphone picker must never offer monitor/loopback sources: choosing one
  // would record the machine's output where the user's own voice is expected.
  const microphoneOptions = useMemo(
    () => audioDevices.devices.filter((d) => d.kind !== "loopback"),
    [audioDevices.devices],
  );

  const loopbackSetupHint = useMemo(() => {
    if (audioDevices.has_loopback) return null;
    switch (audioDevices.platform) {
      case "linux":
        return t("settings.loopbackLinux");
      case "windows":
        return t("settings.loopbackWindows");
      case "macos":
        return t("settings.loopbackMacos");
      default:
        return t("settings.loopbackUnknown");
    }
  }, [audioDevices.has_loopback, audioDevices.platform, t]);

  const noCaptureSource = !captureMicrophone && !captureSystemAudio;
  const serverUrlError = useMemo(
    () => serverUrlProblem(serverUrl, t),
    [serverUrl, t],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  useEffect(() => {
    let cancelled = false;
    setChecking(true);
    api
      .checkServer()
      .then((s) => {
        if (!cancelled) setServerStatus(s);
      })
      .catch(() => {
        if (!cancelled) {
          setServerStatus({
            configured: current.server_token_present,
            reachable: false,
            message: t("settings.connectionCheckFailed"),
          });
        }
      })
      .finally(() => {
        if (!cancelled) setChecking(false);
      });
    return () => {
      cancelled = true;
    };
  }, [current.server_token_present, t]);

  // Probe the dropdown selection (not only the last-saved model) so status and
  // download stay in sync when the user picks a model that is not on disk yet.
  useEffect(() => {
    let cancelled = false;
    const probe =
      transcriptionEngine === "whisper"
        ? api.transcriptionStatus(whisperModel)
        : api.transcriptionStatus();
    probe
      .then((status) => {
        if (!cancelled) setModelStatus(status);
      })
      .catch(() => {
        if (!cancelled) setModelStatus(null);
      });
    return () => {
      cancelled = true;
    };
  }, [whisperModel, transcriptionEngine]);

  const isWhisper = transcriptionEngine === "whisper";

  // An online install may still hold the empty value from when
  // "Auto-detect" was offered for every engine. Show it as English —
  // what the server actually does with an absent language — rather than
  // leaving the select blank on a value no option matches. Display-only
  // on purpose: the stored value is left alone, so switching back to
  // Whisper restores real detection instead of having been rewritten.
  const languageSelectValue =
    !isWhisper && transcriptionLanguage === "" ? "en" : transcriptionLanguage;

  const selectedModelReady = isWhisper
    ? modelStatus?.model === whisperModel && Boolean(modelStatus?.model_ready)
    : Boolean(modelStatus?.model_ready);

  const refreshInstalledModels = useCallback(() => {
    api
      .listInstalledModels()
      .then(setInstalledModels)
      .catch(() => setInstalledModels(null));
  }, []);

  useEffect(() => {
    refreshInstalledModels();
    api
      .recordingState()
      .then((id) => setRecordingActive(Boolean(id)))
      .catch(() => setRecordingActive(false));
  }, [refreshInstalledModels]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;
    events
      .onModelProgress((e) => {
        setDownloadProgress(e.done ? null : e);
      })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  /* ---- instant apply -------------------------------------------------- */
  const payload = useMemo(
    () => ({
      server_url: serverUrl,
      transcription_engine: transcriptionEngine,
      start_at_login: startAtLogin,
      whisper_model: whisperModel,
      diarization_enabled: diarizationEnabled,
      export_markdown: exportMarkdown,
      anthropic_model: anthropicModel,
      chunk_secs: chunkSecs,
      partial_secs: partialSecs,
      capture_microphone: captureMicrophone,
      input_device: inputDevice,
      capture_system_audio: captureSystemAudio,
      system_audio_device: systemAudioDevice,
      summary_instructions: summaryInstructions,
      transcription_language: transcriptionLanguage,
      summary_language: summaryLanguage,
      auto_summarize: autoSummarize,
      call_detection_enabled: callDetectionEnabled,
      call_detection_cooldown_minutes: callDetectionCooldown,
      telemetry_enabled: telemetryEnabled,
    }),
    [
      serverUrl,
      transcriptionEngine,
      startAtLogin,
      whisperModel,
      diarizationEnabled,
      exportMarkdown,
      anthropicModel,
      chunkSecs,
      partialSecs,
      captureMicrophone,
      inputDevice,
      captureSystemAudio,
      systemAudioDevice,
      summaryInstructions,
      transcriptionLanguage,
      summaryLanguage,
      autoSummarize,
      callDetectionEnabled,
      callDetectionCooldown,
      telemetryEnabled,
    ],
  );

  const savedSnapshot = useRef(JSON.stringify(payload));
  const savedTimer = useRef<number | undefined>(undefined);

  useEffect(() => {
    const serialized = JSON.stringify(payload);
    if (serialized === savedSnapshot.current) return;

    if (serverUrlError) {
      setErr(serverUrlError);
      return;
    }
    if (noCaptureSource) {
      setErr(t("settings.noCaptureSource"));
      return;
    }

    let cancelled = false;
    const handle = window.setTimeout(async () => {
      setSaveState("saving");
      setErr(null);
      try {
        const next = await api.saveSettings(payload);
        if (cancelled) return;
        savedSnapshot.current = serialized;
        onSaved(next);
        setSaveState("saved");
        window.clearTimeout(savedTimer.current);
        savedTimer.current = window.setTimeout(
          () => setSaveState("idle"),
          2000,
        );
      } catch (e) {
        if (cancelled) return;
        setErr(normalizeError(e).message);
        setSaveState("idle");
      }
    }, 500);

    return () => {
      cancelled = true;
      window.clearTimeout(handle);
    };
  }, [payload, serverUrlError, noCaptureSource, onSaved, t]);

  useEffect(() => () => window.clearTimeout(savedTimer.current), []);

  /* ---- model management ----------------------------------------------- */
  async function downloadModels() {
    setDownloading(true);
    setErr(null);
    setDownloadProgress(null);
    try {
      const status = await api.downloadModels(whisperModel);
      setModelStatus(status);
      refreshInstalledModels();
    } catch (e) {
      setErr(normalizeError(e).message);
    } finally {
      setDownloading(false);
      setDownloadProgress(null);
    }
  }

  async function deleteModel(modelId: string) {
    setDeletingId(modelId);
    setErr(null);
    try {
      const info = await api.deleteInstalledModel(modelId);
      setInstalledModels(info);
      setConfirmDeleteId(null);
      const status = await api.transcriptionStatus(whisperModel);
      setModelStatus(status);
    } catch (e) {
      setErr(normalizeError(e).message);
    } finally {
      setDeletingId(null);
    }
  }

  function formatSize(bytes: number): string {
    if (bytes >= 1_073_741_824) {
      return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
    }
    return `${Math.max(1, Math.round(bytes / 1_048_576))} MB`;
  }

  function formatMB(bytes: number): string {
    return `${(bytes / 1_048_576).toFixed(0)} MB`;
  }

  function downloadLabel(): string {
    if (!downloading) {
      return selectedModelReady
        ? t("settings.redownload")
        : t("settings.downloadModel");
    }
    if (!downloadProgress) return t("common.starting");
    const { downloaded, total } = downloadProgress;
    if (total && total > 0) {
      const pct = Math.min(100, Math.round((downloaded / total) * 100));
      return `${pct}%`;
    }
    return formatMB(downloaded);
  }

  /**
   * The pill's label, derived from the structured `configured`/`reachable`
   * fields rather than from `serverStatus.message`.
   *
   * The backend sends prose ("Connected", "Server error (503)") which cannot be
   * translated from here, and it sat in English next to translated text. The
   * state itself is structured data, so the label is ours to write; the
   * backend's message is kept as the tooltip, where the detail still earns its
   * place.
   */
  /** Translated model name, falling back to the backend's own label. */
  function installedModelLabel(id: string, kind: string, fallback: string) {
    const key = installedModelLabelKey(id, kind);
    return key ? t(key) : fallback;
  }

  function connectionLabel() {
    if (checking) return t("settings.checking");
    if (!serverStatus) return t("settings.unknown");
    if (serverStatus.reachable) return t("settings.connected");
    return serverStatus.configured
      ? t("settings.unreachable")
      : t("settings.notConfigured");
  }

  function connectionClass() {
    if (checking) return "connection-pill checking";
    if (!serverStatus?.configured) return "connection-pill warn";
    if (serverStatus.reachable) return "connection-pill ok";
    return "connection-pill warn";
  }

  const activeTab = TABS.find((t) => t.id === tab) ?? TABS[0];

  const saveHint =
    saveState === "saving"
      ? t("settings.saving")
      : saveState === "saved"
        ? t("settings.saved")
        : t("settings.applyImmediately");

  return (
    <section className="settings-screen" aria-labelledby="settings-title">
      <header className="st-head">
        <div>
          <h2 id="settings-title">{t("settings.title")}</h2>
          <p className={saveState === "saved" ? "st-save saved" : "st-save"}>
            {saveHint}
          </p>
        </div>
        <button type="button" className="btn ghost" onClick={onClose}>
          {t("common.done")}
        </button>
      </header>

      <div className="st-connection">
        <div className="connection-row">
          <span className="connection-label">{t("settings.server")}</span>
          <span
            className={connectionClass()}
            // The backend's own wording, kept where the detail is useful — it
            // carries things the four states cannot, like an HTTP status.
            title={serverStatus?.message}
          >
            {checking && <span className="spinner" aria-hidden="true" />}
            {connectionLabel()}
          </span>
        </div>
        {!checking && serverStatus && !serverStatus.reachable && (
          <p className="muted tiny connection-hint">
            {serverStatus.configured
              ? t("settings.serverUnreachableConfigured")
              : t("settings.serverUnlinked")}
          </p>
        )}
      </div>

      {err && <p className="error-text st-error">{err}</p>}

      <div className="st-body">
        <nav
          className="st-tabs"
          role="tablist"
          aria-label={t("settings.sectionsLabel")}
        >
          {/* Named `section`, not `t` — that would shadow the translator. */}
          {TABS.map((section) => (
            <button
              key={section.id}
              type="button"
              role="tab"
              id={`st-tab-${section.id}`}
              aria-selected={section.id === tab}
              aria-controls="st-panel"
              className={section.id === tab ? "st-tab active" : "st-tab"}
              onClick={() => setTab(section.id)}
            >
              {t(section.label)}
            </button>
          ))}
        </nav>

        <div
          className="st-panel"
          id="st-panel"
          role="tabpanel"
          aria-labelledby={`st-tab-${tab}`}
        >
          <h3>{t(activeTab.label)}</h3>
          <p className="st-panel-blurb">{t(activeTab.blurb)}</p>

          {tab === "language" && (
            <div className="st-rows">
              <Row
                label={t("settings.language")}
                hint={t("settings.languageHint")}
                htmlFor="ui-language"
              >
                <select
                  id="ui-language"
                  value={locale}
                  onChange={(e) => setLocale(e.target.value as typeof locale)}
                >
                  {/* Each language named in itself, so it is recognisable while
                      the rest of the UI is still unreadable. */}
                  {LOCALES.map((code) => (
                    <option key={code} value={code}>
                      {LOCALE_NAMES[code]}
                    </option>
                  ))}
                </select>
              </Row>
              {/* Read-only: dates follow the OS region, so there is nothing for
                  us to set — but the tab is called "& Region", and saying where
                  the format comes from is what makes that honest. */}
              <Row
                label={t("settings.dateFormat")}
                hint={t("settings.dateFormatHint")}
              >
                <span className="muted tiny">
                  {new Date().toLocaleString()}
                </span>
              </Row>
            </div>
          )}

          {tab === "appearance" && (
            <div className="st-theme-cards">
              {(
                [
                  { key: "light", label: t("theme.light") },
                  { key: "dark", label: t("theme.dark") },
                  { key: "system", label: t("theme.system") },
                ] as { key: ThemePreference; label: string }[]
              ).map((c) => (
                <button
                  key={c.key}
                  type="button"
                  aria-pressed={theme === c.key}
                  className={
                    theme === c.key ? "st-theme-card active" : "st-theme-card"
                  }
                  onClick={() => {
                    setTheme(c.key);
                    setThemePreference(c.key);
                  }}
                >
                  <span
                    className={`st-theme-swatch ${c.key}`}
                    aria-hidden="true"
                  />
                  <span className="st-theme-label">{c.label}</span>
                </button>
              ))}
            </div>
          )}

          {tab === "reading" && (
            <div className="st-rows">
              <Row
                label={t("settings.textSize")}
                hint={t("settings.textSizeHint")}
                htmlFor="text-scale"
              >
                <select
                  id="text-scale"
                  value={reading.textScale}
                  onChange={(e) =>
                    updateReading({
                      textScale: e.target.value as ReadingPrefs["textScale"],
                    })
                  }
                >
                  <option value="normal">{t("settings.sizeDefault")}</option>
                  <option value="large">{t("settings.sizeLarge")}</option>
                  <option value="xlarge">{t("settings.sizeXLarge")}</option>
                </select>
              </Row>
              <Row label={t("settings.lineSpacing")} htmlFor="line-spacing">
                <select
                  id="line-spacing"
                  value={reading.lineSpacing}
                  onChange={(e) =>
                    updateReading({
                      lineSpacing: e.target
                        .value as ReadingPrefs["lineSpacing"],
                    })
                  }
                >
                  <option value="normal">{t("settings.spacingDefault")}</option>
                  <option value="relaxed">
                    {t("settings.spacingRelaxed")}
                  </option>
                  <option value="loose">{t("settings.spacingLoose")}</option>
                </select>
              </Row>
              <Row label={t("settings.highContrast")}>
                <Toggle
                  id="high-contrast"
                  label={t("settings.highContrast")}
                  checked={reading.highContrast}
                  onChange={(v) => updateReading({ highContrast: v })}
                />
              </Row>
              <Row
                label={t("settings.reduceMotion")}
                hint={t("settings.reduceMotionHint")}
              >
                <Toggle
                  id="reduce-motion"
                  label={t("settings.reduceMotion")}
                  checked={reading.reduceMotion}
                  onChange={(v) => updateReading({ reduceMotion: v })}
                />
              </Row>
              <p className="muted tiny st-note">
                {t("settings.readingOnThisDevice")}
              </p>
            </div>
          )}

          {tab === "audio" && (
            <div className="st-rows">
              {/* Only where the Call detection tab is hidden, so setup is always
                  reachable but never listed twice. */}
              {!current.call_detection_supported && (
                <Row
                  label={t("settings.rerunOnboarding")}
                  hint={t("settings.rerunOnboardingHint")}
                >
                  <button
                    type="button"
                    className="btn"
                    onClick={onRerunOnboarding}
                  >
                    {t("settings.rerunOnboardingAction")}
                  </button>
                </Row>
              )}
              <Row label={t("settings.captureMic")}>
                <Toggle
                  id="capture-microphone"
                  label={t("settings.captureMic")}
                  checked={captureMicrophone}
                  onChange={setCaptureMicrophone}
                />
              </Row>
              {captureMicrophone && (
                <Row
                  label={t("settings.microphone")}
                  htmlFor="input-device"
                  hint={t("settings.microphoneHint")}
                >
                  <select
                    id="input-device"
                    value={inputDevice}
                    onChange={(e) => setInputDevice(e.target.value)}
                  >
                    <option value="">{t("settings.systemDefault")}</option>
                    {microphoneOptions.map((d) => (
                      <option key={d.name} value={d.name}>
                        {d.label}
                      </option>
                    ))}
                  </select>
                </Row>
              )}
              <Row
                label={t("settings.captureSystemAudio")}
                hint={t("settings.captureSystemAudioHint")}
              >
                <Toggle
                  id="capture-system-audio"
                  label={t("settings.captureSystemAudio")}
                  checked={captureSystemAudio}
                  onChange={setCaptureSystemAudio}
                />
              </Row>
              {captureSystemAudio &&
                (loopbackSetupHint ? (
                  <Row
                    label={t("settings.systemAudioSource")}
                    hint={loopbackSetupHint}
                  />
                ) : (
                  <Row
                    label={t("settings.systemAudioSource")}
                    htmlFor="system-audio-device"
                  >
                    <select
                      id="system-audio-device"
                      value={systemAudioDevice}
                      onChange={(e) => setSystemAudioDevice(e.target.value)}
                    >
                      <option value="">{t("settings.defaultOutput")}</option>
                      {loopbackDevices.map((d) => (
                        <option key={d.name} value={d.name}>
                          {d.label}
                        </option>
                      ))}
                    </select>
                  </Row>
                ))}
              {noCaptureSource && (
                <p className="error-text tiny st-note">
                  {t("settings.noCaptureSource")}
                </p>
              )}
            </div>
          )}

          {tab === "call-detection" && (
            <div className="st-rows">
              {current.call_detection_supported ? (
                <>
                  <Row
                    label={t("settings.rerunOnboarding")}
                    hint={t("settings.rerunOnboardingHint")}
                  >
                    <button
                      type="button"
                      className="btn"
                      onClick={onRerunOnboarding}
                    >
                      {t("settings.rerunOnboardingAction")}
                    </button>
                  </Row>
                  <Row
                    label={t("settings.callPrompt")}
                    hint={t("settings.callPromptHint")}
                  >
                    <Toggle
                      id="call-detection-enabled"
                      label={t("settings.callPrompt")}
                      checked={callDetectionEnabled}
                      onChange={setCallDetectionEnabled}
                    />
                  </Row>
                  <Row
                    label={t("settings.callCooldown")}
                    hint={t("settings.callCooldownHint")}
                    htmlFor="call-detection-cooldown"
                  >
                    <input
                      id="call-detection-cooldown"
                      className="st-number"
                      type="number"
                      min={0}
                      max={120}
                      value={callDetectionCooldown}
                      onChange={(e) =>
                        setCallDetectionCooldown(
                          Math.max(
                            0,
                            Math.min(120, Number(e.target.value) || 0),
                          ),
                        )
                      }
                      disabled={!callDetectionEnabled}
                    />
                  </Row>
                </>
              ) : (
                <p className="muted tiny st-note">
                  {t("settings.callUnsupported")}
                </p>
              )}
            </div>
          )}

          {tab === "transcription" && (
            <div className="st-rows">
              <Row
                label={t("settings.engine")}
                htmlFor="transcription-engine"
                hint={
                  isWhisper
                    ? t("settings.engineWhisperHint")
                    : t("settings.engineCloudHint")
                }
              >
                <select
                  id="transcription-engine"
                  value={transcriptionEngine}
                  onChange={(e) =>
                    setTranscriptionEngine(
                      e.target.value as TranscriptionEngine,
                    )
                  }
                >
                  <option value="deepgram">{t("settings.engineCloud")}</option>
                  <option value="whisper">{t("settings.engineWhisper")}</option>
                </select>
              </Row>

              {!isWhisper ? (
                <Row
                  label={t("settings.statusLabel")}
                  hint={
                    selectedModelReady
                      ? t("settings.onlineReady", {
                          model: modelStatus?.model ?? "Deepgram",
                        })
                      : t("settings.onlineNotConfigured")
                  }
                />
              ) : (
                <>
                  <Row
                    label={t("settings.accuracyModel")}
                    htmlFor="whisper-model"
                  >
                    <select
                      id="whisper-model"
                      value={whisperModel}
                      onChange={(e) => setWhisperModel(e.target.value)}
                    >
                      {WHISPER_MODELS.map((m) => (
                        <option key={m.id} value={m.id}>
                          {t(m.label)} ({m.sizeLabel})
                        </option>
                      ))}
                    </select>
                  </Row>

                  <Row
                    label={t("settings.modelFiles")}
                    hint={
                      downloading && downloadProgress
                        ? t("settings.modelDownloading", {
                            label: downloadProgress.label,
                          })
                        : selectedModelReady
                          ? t("settings.modelReady", { model: whisperModel })
                          : t("settings.modelMissing", { model: whisperModel })
                    }
                  >
                    <button
                      type="button"
                      className="btn ghost"
                      onClick={downloadModels}
                      disabled={downloading}
                    >
                      {downloadLabel()}
                    </button>
                  </Row>

                  {downloading && (
                    <div className="st-note">
                      <div
                        className="level-meter"
                        title={t("settings.downloadProgress")}
                        aria-hidden="true"
                      >
                        <div
                          className="level-fill"
                          style={{
                            width:
                              downloadProgress?.total &&
                              downloadProgress.total > 0
                                ? `${Math.min(100, Math.round((downloadProgress.downloaded / downloadProgress.total) * 100))}%`
                                : "100%",
                          }}
                        />
                      </div>
                      <p className="muted tiny">
                        {t("settings.downloadOnce", {
                          model: whisperModel,
                          size:
                            whisperModelSizeLabel(whisperModel) ??
                            t("model.unknownSize"),
                        })}
                      </p>
                    </div>
                  )}

                  {installedModels && installedModels.models.length > 0 && (
                    <details
                      className="advanced-section installed-models-section"
                      onToggle={(e) => {
                        if (!(e.currentTarget as HTMLDetailsElement).open) {
                          setConfirmDeleteId(null);
                        }
                      }}
                    >
                      <summary>
                        {t("settings.downloadedModels", {
                          size: formatSize(installedModels.total_bytes),
                        })}
                      </summary>
                      <p className="muted tiny">
                        {t("settings.downloadedModelsHint")}
                      </p>
                      <ul className="installed-models-list">
                        {installedModels.models.map((m) => (
                          <li key={m.id} className="installed-model-row">
                            <div className="installed-model-meta">
                              <span className="installed-model-label">
                                {installedModelLabel(m.id, m.kind, m.label)}
                              </span>
                              <span className="muted tiny">
                                {formatSize(m.size_bytes)}
                                {(m.kind === "whisper" &&
                                  m.id === whisperModel) ||
                                (m.kind === "diarization" && diarizationEnabled)
                                  ? t("settings.inUse")
                                  : ""}
                              </span>
                            </div>
                            {confirmDeleteId === m.id ? (
                              <div className="installed-model-confirm">
                                <span className="tiny">
                                  {t("settings.deleteQuestion")}
                                </span>
                                <button
                                  type="button"
                                  className="btn ghost danger"
                                  disabled={
                                    Boolean(deletingId) ||
                                    downloading ||
                                    recordingActive
                                  }
                                  onClick={() => deleteModel(m.id)}
                                >
                                  {deletingId === m.id
                                    ? t("settings.deleting")
                                    : t("common.delete")}
                                </button>
                                <button
                                  type="button"
                                  className="btn ghost"
                                  disabled={Boolean(deletingId)}
                                  onClick={() => setConfirmDeleteId(null)}
                                >
                                  {t("common.cancel")}
                                </button>
                              </div>
                            ) : (
                              <button
                                type="button"
                                className="btn ghost danger"
                                disabled={
                                  Boolean(deletingId) ||
                                  downloading ||
                                  recordingActive
                                }
                                onClick={() => setConfirmDeleteId(m.id)}
                              >
                                {t("common.delete")}
                              </button>
                            )}
                          </li>
                        ))}
                      </ul>
                      {recordingActive && (
                        <p className="muted tiny">
                          {t("settings.stopBeforeDeletingModels")}
                        </p>
                      )}
                    </details>
                  )}
                </>
              )}

              <Row
                label={t("settings.identifySpeakers")}
                hint={
                  isWhisper
                    ? t("settings.identifySpeakersWhisper")
                    : t("settings.identifySpeakersCloud")
                }
              >
                <Toggle
                  id="diarization"
                  label={t("settings.identifySpeakers")}
                  checked={diarizationEnabled}
                  onChange={setDiarizationEnabled}
                />
              </Row>

              <Row
                label={t("settings.spokenLanguage")}
                htmlFor="transcription-language"
                hint={t("settings.spokenLanguageHint")}
              >
                <select
                  id="transcription-language"
                  value={languageSelectValue}
                  onChange={(e) => setTranscriptionLanguage(e.target.value)}
                >
                  {/* Only Whisper actually detects the spoken language. The online
                      engine ignores an absent language and falls back to English, so
                      offering "Auto-detect" there promises something it does not do —
                      and the failure is silent: non-English speech transcribes to
                      nothing at all. */}
                  {isWhisper && (
                    <option value="">{t("settings.autoDetect")}</option>
                  )}
                  {LANGUAGES.map((l) => (
                    <option key={l.code} value={l.code}>
                      {l.name}
                    </option>
                  ))}
                </select>
              </Row>
            </div>
          )}

          {tab === "summary" && (
            <div className="st-rows">
              <Row
                label={t("settings.autoSummarize")}
                hint={t("settings.autoSummarizeHint")}
              >
                <Toggle
                  id="auto-summarize"
                  label={t("settings.autoSummarize")}
                  checked={autoSummarize}
                  onChange={setAutoSummarize}
                />
              </Row>
              <Row
                label={t("settings.summaryLanguage")}
                htmlFor="summary-language"
                hint={t("settings.summaryLanguageHint")}
              >
                <select
                  id="summary-language"
                  value={summaryLanguage}
                  onChange={(e) => setSummaryLanguage(e.target.value)}
                >
                  <option value="">{t("settings.matchTranscript")}</option>
                  {LANGUAGES.map((l) => (
                    <option key={l.code} value={l.name}>
                      {l.name}
                    </option>
                  ))}
                </select>
              </Row>
              <Row
                label={t("settings.summaryInstructions")}
                htmlFor="summary-instructions"
                hint={t("settings.summaryInstructionsHint")}
                stack
              >
                <textarea
                  id="summary-instructions"
                  className="settings-textarea"
                  rows={4}
                  maxLength={2000}
                  value={summaryInstructions}
                  onChange={(e) => setSummaryInstructions(e.target.value)}
                  placeholder={t("detail.instructionsPlaceholder")}
                />
              </Row>
            </div>
          )}

          {tab === "privacy" && (
            <div className="st-rows">
              <Row
                label={t("settings.telemetry")}
                hint={t("settings.telemetryHint")}
              >
                <Toggle
                  id="telemetry-enabled"
                  label={t("settings.telemetry")}
                  checked={telemetryEnabled}
                  onChange={setTelemetryEnabled}
                />
              </Row>
              <p className="muted tiny st-note">
                What is sent: feature usage counts, duration ranges, error
                categories, app version, operating system and version, CPU type
                and core count, and a random install ID you can reset. What is
                never sent: your recordings, transcripts, summaries, meeting
                titles, participant names, file paths, or anything you type or
                say. If the app is offline, reports wait in a small file on this
                device and are sent later. Reports are kept for 12 months.
                Turning this off stops all reporting immediately, deletes
                anything still waiting on this device, and deletes the install
                ID.
              </p>
            </div>
          )}

          {tab === "advanced" && (
            <div className="st-rows">
              <Row
                label={t("settings.startAtLogin")}
                hint={
                  // Only macOS detects meetings, so only there does running
                  // in the background buy anything beyond opening faster.
                  // Saying otherwise promises Windows and Linux users a
                  // feature they will never get.
                  current.call_detection_supported
                    ? t("settings.startAtLoginHint")
                    : t("settings.startAtLoginHintNoDetection")
                }
              >
                <Toggle
                  id="start-at-login"
                  label={t("settings.startAtLogin")}
                  checked={startAtLogin}
                  onChange={setStartAtLogin}
                />
              </Row>
              <Row
                label={t("settings.serverUrl")}
                htmlFor="server-url"
                hint={
                  current.server_url_from_build
                    ? t("settings.serverUrlLocked", {
                        url: serverUrl || t("settings.serverUrlEmbedded"),
                      })
                    : serverUrlError
                      ? serverUrlError
                      : t("settings.serverUrlHint")
                }
              >
                <input
                  id="server-url"
                  className="st-input"
                  value={serverUrl}
                  onChange={(e) => setServerUrl(e.target.value)}
                  placeholder="https://minutes.example.com"
                  disabled={current.server_url_from_build}
                  aria-invalid={serverUrlError ? "true" : undefined}
                />
              </Row>
              <Row
                label={t("settings.accessToken")}
                hint={
                  current.server_token_from_build
                    ? t("settings.tokenFromBuild")
                    : current.server_token_from_env
                      ? t("settings.tokenFromEnv")
                      : current.server_token_present
                        ? t("settings.tokenInKeychain")
                        : t("settings.tokenMissing")
                }
              />
              {current.device_id && (
                <Row
                  label={t("settings.deviceId")}
                  hint={t("settings.deviceIdHint")}
                >
                  <code className="st-readonly">{current.device_id}</code>
                </Row>
              )}
              <Row label={t("settings.summaryModel")} htmlFor="anthropic-model">
                <input
                  id="anthropic-model"
                  className="st-input"
                  value={anthropicModel}
                  onChange={(e) => setAnthropicModel(e.target.value)}
                />
              </Row>
              <Row
                label={t("settings.chunkLength")}
                htmlFor="chunk-secs"
                hint={t("settings.chunkLengthHint")}
              >
                <input
                  id="chunk-secs"
                  className="st-number"
                  type="number"
                  min={2}
                  max={60}
                  value={chunkSecs}
                  onChange={(e) => setChunkSecs(Number(e.target.value))}
                />
              </Row>
              <Row
                label={t("settings.partialInterval")}
                htmlFor="partial-secs"
                hint={t("settings.partialIntervalHint")}
              >
                <input
                  id="partial-secs"
                  className="st-number"
                  type="number"
                  min={0}
                  max={30}
                  value={partialSecs}
                  onChange={(e) => setPartialSecs(Number(e.target.value))}
                />
              </Row>
              <Row
                label={t("settings.exportMarkdown")}
                hint={t("settings.exportMarkdownHint")}
              >
                <Toggle
                  id="export-markdown"
                  label={t("settings.exportMarkdown")}
                  checked={exportMarkdown}
                  onChange={setExportMarkdown}
                />
              </Row>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
