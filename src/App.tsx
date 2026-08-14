import { useCallback, useEffect, useRef, useState } from "react";
import { save, ask } from "@tauri-apps/plugin-dialog";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { api, events } from "./api";
import type {
  Meeting,
  MeetingDetail,
  MeetingListItem,
  MeetingSearchHit,
  PermissionsReport,
  Segment,
  SettingsView,
} from "./types";
import { NavRail } from "./components/NavRail";
import { TopBar } from "./components/TopBar";
import { HomeScreen } from "./components/HomeScreen";
import { NotesScreen } from "./components/NotesScreen";
import { RecordingScreen } from "./components/RecordingScreen";
import { MeetingView } from "./components/MeetingView";
import { SettingsScreen } from "./components/SettingsScreen";
import { OnboardingScreen } from "./components/OnboardingScreen";
import { CommandPalette } from "./components/CommandPalette";
import { LoadingModal } from "./components/LoadingModal";
import type { Screen } from "./screens";
import { simulationReport, type SimulationId } from "./devOnboarding";
import type { ShareFormat } from "./types";
import {
  getThemePreference,
  setThemePreference,
  type ThemePreference,
} from "./theme";
import { formatDuration, meetingDurationMs, sanitizeFilename } from "./utils/format";
import { engineModeLabel } from "./utils/engineLabel";
import { useT } from "./i18n";
import { mergeSegments } from "./utils/transcript";
import { summaryToMarkdown } from "./utils/summaryMarkdown";
import { normalizeError, type AppError } from "./utils/errors";
import { useErrorText } from "./utils/errorText";

const TOAST_MS = 4000;
// Errors and guidance need more reading time than success confirmations.
const TOAST_ERROR_MS = 8000;

/**
 * Meetings shorter than this are not summarized automatically. A stray
 * recording that gets stopped seconds later has nothing worth summarizing, and
 * firing a server request for it is pure waste. The manual button has no such
 * floor, so a genuinely short meeting can still be summarized on request.
 */
const AUTO_SUMMARY_MIN_MS = 60_000;

export default function App() {
  const t = useT();
  const errorText = useErrorText();
  const [meetings, setMeetings] = useState<MeetingListItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<MeetingDetail | null>(null);
  const [recordingId, setRecordingId] = useState<string | null>(null);
  const [partialText, setPartialText] = useState("");
  const [summarizing, setSummarizing] = useState(false);
  const [summaryError, setSummaryError] = useState<AppError | null>(null);
  const [summaryInstructions, setSummaryInstructions] = useState<
    Record<string, string>
  >({});
  const [settings, setSettings] = useState<SettingsView | null>(null);
  const [screen, setScreen] = useState<Screen>("home");
  /* First-run setup. `null` means "not asked yet": the shell stays unrendered
     until the backend answers, because flashing Home and then replacing it with
     a wizard looks like a bug. A *failed* probe resolves to `false` and the app
     opens normally — setup must never be able to lock someone out of their
     notes. */
  const [onboarding, setOnboarding] = useState<PermissionsReport | null>(null);
  const [onboardingChecked, setOnboardingChecked] = useState(false);
  /* Dev-only: which synthetic report the wizard is being shown with, if any.
     `null` is the normal state and the only possible state in a release build. */
  const [onboardingSim, setOnboardingSim] = useState<SimulationId | null>(null);
  const [settingsLoading, setSettingsLoading] = useState(false);
  const [railCollapsed, setRailCollapsed] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [themePreference, setThemePreferenceState] =
    useState<ThemePreference>(getThemePreference);
  const [toast, setToast] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [level, setLevel] = useState(0);
  const [, setTick] = useState(0);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<MeetingSearchHit[] | null>(
    null,
  );

  // Refs so event listeners (registered once) always see current values.
  const selectedRef = useRef<string | null>(null);
  selectedRef.current = selectedId;
  const recordingRef = useRef<string | null>(null);
  recordingRef.current = recordingId;
  // Synchronous re-entry guard for start/stop. The `busy` state only disables
  // the button on the next render, so a double click in the same tick fires
  // the handler twice; the ref flips synchronously and blocks the second call.
  const busyRef = useRef(false);
  // Read inside the once-registered event listeners, which never see updated
  // state directly.
  const settingsRef = useRef<SettingsView | null>(null);
  settingsRef.current = settings;
  const instructionsRef = useRef<Record<string, string>>({});
  instructionsRef.current = summaryInstructions;
  // A summary is one request at a time, and the guard has to be synchronous:
  // `summarizing` state only lands on the next render, so an auto-trigger and a
  // button press in the same tick would both get through.
  const summarizingRef = useRef(false);
  // Meetings already auto-attempted this session, so a repeated `completed`
  // event cannot summarize the same meeting twice.
  const autoSummarized = useRef(new Set<string>());

  // Live `transcript-final` events and `get_meeting` are two racing views of the
  // same transcript. A segment emitted while a fetch is in flight is not in that
  // response, and used to be dropped outright when it arrived before the first
  // fetch resolved (detail still null) — the meeting then showed an incomplete
  // transcript, or none at all, until the next reload. Buffer live segments per
  // meeting and merge them into whatever a fetch returns.
  const liveSegments = useRef(new Map<string, Segment[]>());

  // A leftover timer from an earlier toast would dismiss the current one
  // early (e.g. a 4s success timer cutting off an 8s error), so each new
  // toast cancels the previous timer.
  const toastTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flashToast = useCallback((msg: string, durationMs: number = TOAST_MS) => {
    if (toastTimerRef.current !== null) clearTimeout(toastTimerRef.current);
    setToast(msg);
    toastTimerRef.current = setTimeout(() => {
      toastTimerRef.current = null;
      setToast(null);
    }, durationMs);
  }, []);

  useEffect(() => {
    return () => {
      if (toastTimerRef.current !== null) clearTimeout(toastTimerRef.current);
    };
  }, []);

  const refreshMeetings = useCallback(async () => {
    try {
      setMeetings(await api.listMeetings());
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }, [flashToast, errorText]);

  const loadDetail = useCallback(async (id: string) => {
    try {
      const fetched = await api.getMeeting(id);
      const buffered = liveSegments.current.get(id);
      setDetail(
        buffered?.length
          ? { ...fetched, segments: mergeSegments(fetched.segments, buffered) }
          : fetched,
      );
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }, [flashToast, errorText]);

  /**
   * Generate (or regenerate) the summary for one meeting. Shared by the manual
   * button and the automatic post-meeting run, so both produce the same state:
   * the summary attached to the open meeting, the AI-chosen title adopted, and
   * the list refreshed.
   *
   * Takes a meeting id rather than reading `selectedId`: the automatic run can
   * finish after the user has navigated elsewhere, and must not write its result
   * onto whatever meeting happens to be open by then.
   */
  const runSummary = useCallback(
    async (meetingId: string) => {
      if (summarizingRef.current) return;
      summarizingRef.current = true;
      setSummarizing(true);
      setSummaryError(null);
      try {
        const summary = await api.generateSummary(
          meetingId,
          instructionsRef.current[meetingId],
        );
        // The backend adopts the AI title for the meeting; reflect it locally.
        setDetail((d) =>
          d && d.meeting.id === meetingId
            ? {
                ...d,
                summary,
                meeting: {
                  ...d.meeting,
                  title: summary.content.title || d.meeting.title,
                },
              }
            : d,
        );
        await refreshMeetings();
      } catch (e) {
        // Summary generation is the one path that rejects with a categorized
        // { kind, message } error (see src-tauri/src/error.rs) rather than a
        // plain string, so the UI can tell "can't reach the server" apart from
        // "bad token" apart from a generic failure and show a persistent,
        // specific error with a retry affordance instead of a toast that
        // auto-dismisses before the user can act on it.
        const err = normalizeError(e);
        if (meetingId === selectedRef.current) {
          setSummaryError(err);
        } else {
          // The meeting is not on screen, so the error card has nowhere to
          // show. Say it once rather than failing silently.
          flashToast(
            t("toast.summarizeFailed", { message: err.message }),
            TOAST_ERROR_MS,
          );
        }
      } finally {
        summarizingRef.current = false;
        setSummarizing(false);
      }
    },
    [refreshMeetings, flashToast, t],
  );

  /**
   * Summarize a just-finished meeting without being asked. Called from the
   * `completed` status event rather than from the Stop handler: `stop_recording`
   * returns as soon as capture ends while the transcript is still being flushed,
   * so stopping is not the moment the transcript is final — this event is.
   *
   * Every reason to skip is a silent one. This is a background courtesy, so a
   * meeting that does not qualify should simply not be summarized, leaving the
   * manual button to do it on request.
   */
  const autoSummarize = useCallback(
    async (meetingId: string) => {
      const current = settingsRef.current;
      if (!current?.auto_summarize) return;
      // Nothing the user can act on from here; the manual path is where the
      // "server isn't set up" guidance belongs.
      if (!current.server_token_present) return;
      if (autoSummarized.current.has(meetingId)) return;

      // Fetch rather than trust local state: this needs the flushed transcript
      // and the final ended_at, and the meeting may not be the one on screen.
      let finished: MeetingDetail;
      try {
        finished = await api.getMeeting(meetingId);
      } catch {
        return;
      }
      if (finished.summary) return;
      if (finished.segments.length === 0) return;
      if (
        meetingDurationMs(finished.meeting.created_at, finished.meeting.ended_at) <
        AUTO_SUMMARY_MIN_MS
      ) {
        return;
      }

      autoSummarized.current.add(meetingId);
      await runSummary(meetingId);
    },
    [runSummary],
  );

  // Initial load + recovery of in-progress recording state.
  // Runs before the shell paints. Deliberately its own effect: it must not be
  // able to fail the settings/meetings load, and that load must not delay it.
  useEffect(() => {
    (async () => {
      try {
        const report = await api.permissionStatus();
        if (report.onboardingRequired) setOnboarding(report);
      } catch {
        // Could not tell — open the app. Better than a wizard nobody can leave.
      } finally {
        setOnboardingChecked(true);
      }
    })();
  }, []);

  useEffect(() => {
    (async () => {
      try {
        setSettings(await api.getSettings());
        await refreshMeetings();
        const active = await api.recordingState();
        if (active) {
          setRecordingId(active);
          setSelectedId(active);
          setScreen("detail");
        }
      } catch (e) {
        flashToast(errorText(e), TOAST_ERROR_MS);
      }
    })();
  }, [refreshMeetings, flashToast, errorText]);

  useEffect(() => {
    setSummaryError(null);
    if (selectedId) loadDetail(selectedId);
    else setDetail(null);
  }, [selectedId, loadDetail]);

  // Full-text search across titles + transcripts (debounced). An empty query
  // clears results so the sidebar falls back to the full meeting list.
  useEffect(() => {
    const q = searchQuery.trim();
    if (!q) {
      setSearchResults(null);
      return;
    }
    let cancelled = false;
    const t = setTimeout(() => {
      api
        .searchMeetings(q)
        .then((hits) => {
          if (!cancelled) setSearchResults(hits);
        })
        .catch(() => {
          if (!cancelled) setSearchResults([]);
        });
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(t);
    };
  }, [searchQuery]);

  // Live transcript event wiring (registered once).
  useEffect(() => {
    const PARTIAL_MS = 80;
    let partialTimer: ReturnType<typeof setTimeout> | null = null;
    let pendingPartial: { meetingId: string; text: string } | null = null;

    const flushPartial = () => {
      partialTimer = null;
      if (!pendingPartial) return;
      const { meetingId, text } = pendingPartial;
      pendingPartial = null;
      if (meetingId === selectedRef.current) setPartialText(text);
    };

    const unlisteners = Promise.all([
      events.onFinal(({ meetingId, segment }) => {
        if (partialTimer) {
          clearTimeout(partialTimer);
          partialTimer = null;
        }
        pendingPartial = null;
        if (meetingId === selectedRef.current) {
          // Buffered first, so a fetch that is already in flight still picks
          // this segment up when it resolves.
          const buffer = liveSegments.current;
          buffer.set(meetingId, mergeSegments(buffer.get(meetingId) ?? [], [segment]));
          setDetail((d) =>
            d && d.meeting.id === meetingId
              ? { ...d, segments: mergeSegments(d.segments, [segment]) }
              : d,
          );
          setPartialText("");
        }
        setMeetings((ms) =>
          ms.map((m) =>
            m.id === meetingId
              ? { ...m, segment_count: m.segment_count + 1 }
              : m,
          ),
        );
      }),
      events.onPartial(({ meetingId, text }) => {
        pendingPartial = { meetingId, text };
        if (partialTimer) return;
        partialTimer = setTimeout(flushPartial, PARTIAL_MS);
      }),
      events.onStatus(({ meetingId, status }) => {
        if (status === "completed") {
          setRecordingId((cur) => (cur === meetingId ? null : cur));
          setPartialText("");
          refreshMeetings();
          // Re-fetch first (it merges the buffer), then drop the buffer: the
          // stored transcript is now the complete one.
          if (meetingId === selectedRef.current) {
            loadDetail(meetingId).finally(() => {
              liveSegments.current.delete(meetingId);
            });
          } else {
            liveSegments.current.delete(meetingId);
          }
          // The transcript is final as of this event, so this is the earliest
          // point an automatic summary would summarize the whole meeting.
          void autoSummarize(meetingId);
        }
      }),
      events.onError(({ message, code }) =>
        flashToast(
          t("toast.transcription", { message: errorText({ kind: "internal", message, code: code ?? undefined }) }),
          TOAST_ERROR_MS,
        ),
      ),
      // A device swap mid-recording is recovered automatically, but the user
      // still needs to know their audio moved.
      events.onCaptureNotice(({ message }) =>
        flashToast(t("toast.audio", { message }), TOAST_ERROR_MS),
      ),
      events.onLevel(({ meetingId, level }) => {
        if (meetingId === recordingRef.current) setLevel(level);
      }),
      events.onMeetingStarted((meeting: Meeting) => {
        setRecordingId(meeting.id);
        setSelectedId(meeting.id);
        setScreen("detail");
        setPartialText("");
        void refreshMeetings();
      }),
    ]);
    return () => {
      if (partialTimer) clearTimeout(partialTimer);
      unlisteners.then((fns) => fns.forEach((f) => f()));
    };
  }, [refreshMeetings, loadDetail, flashToast, autoSummarize, t, errorText]);

  // Tick once a second while recording to drive the live duration display.
  useEffect(() => {
    if (!recordingId) {
      setLevel(0);
      return;
    }
    const t = setInterval(() => setTick((n) => n + 1), 1000);
    return () => clearInterval(t);
  }, [recordingId]);

  /** Re-run first-run setup from Settings. */
  async function handleRerunOnboarding() {
    try {
      // Resets the marker and reports as a fresh install, so the whole sequence
      // plays rather than only what is still outstanding — the user asked to see
      // it again.
      setOnboarding(await api.resetOnboarding());
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  /** Where "Done"/Escape returns to when leaving Settings. */
  function leaveSettings() {
    setScreen(selectedId ? "detail" : "home");
  }

  async function openSettings() {
    setScreen("settings");
    if (settings || settingsLoading) return;
    setSettingsLoading(true);
    try {
      setSettings(await api.getSettings());
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
      leaveSettings();
    } finally {
      setSettingsLoading(false);
    }
  }

  function openMeeting(id: string) {
    setSelectedId(id);
    setScreen("detail");
  }

  async function handleStart() {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    try {
      try {
        const status = await api.transcriptionStatus();
        if (!status.model_ready) {
          const isWhisper = settings?.transcription_engine === "whisper";
          flashToast(
            isWhisper
              ? t("toast.downloadModelFirst", { model: status.model })
              : t("toast.configureOnline"),
            TOAST_ERROR_MS,
          );
          await openSettings();
          return;
        }
      } catch {
        // If the status probe fails, let the backend surface the error on start.
      }
      // "New Meeting" is an explicit instruction, so it records straight away.
      // The floating confirm prompt is only for the other trigger — a call the
      // system detected on its own (see call_detect.rs), where asking first is
      // the whole point.
      //
      // The backend names the meeting when no title is given, and
      // `start_recording` does not emit `meeting-started` (only the prompt path
      // does), so adopt the returned meeting here.
      const meeting = await api.startRecording();
      setRecordingId(meeting.id);
      setSelectedId(meeting.id);
      setScreen("detail");
      setPartialText("");
      await refreshMeetings();
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  }

  async function handleStop() {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    try {
      const id = await api.stopRecording();
      // Stop returns as soon as capture ends; transcript flush continues in the background.
      setRecordingId(null);
      setPartialText("");
      await refreshMeetings();
      if (id) {
        setSelectedId(id);
        setScreen("detail");
      }
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  }

  async function handleDelete(id: string) {
    const meeting = meetings.find((m) => m.id === id);
    const name = meeting?.title
      ? `“${meeting.title}”`
      : t("confirm.deleteThis");
    const confirmed = await ask(t("confirm.deleteBody", { name }), {
      title: t("confirm.deleteTitle"),
      kind: "warning",
      okLabel: t("common.delete"),
      cancelLabel: t("common.cancel"),
    });
    if (!confirmed) return;
    try {
      await api.deleteMeeting(id);
      if (selectedId === id) {
        setSelectedId(null);
        setScreen("notes");
      }
      await refreshMeetings();
      flashToast(t("toast.meetingDeleted"));
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  async function handleGenerateSummary() {
    if (!selectedId) return;
    let current = settings;
    try {
      current = await api.getSettings();
      setSettings(current);
    } catch {
      // fall back to cached settings if the refresh fails
    }
    if (!current?.server_token_present) {
      flashToast(t("toast.serverNotSetUp"), TOAST_ERROR_MS);
      await openSettings();
      return;
    }
    await runSummary(selectedId);
  }

  async function handleCopySummary() {
    if (!detail?.summary) return;
    try {
      await writeText(summaryToMarkdown(detail.summary));
      flashToast(t("toast.copiedSummary"));
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  async function handleCopyTranscript() {
    if (!detail || detail.segments.length === 0) return;
    try {
      const text = detail.segments.map((s) => s.text).join("\n");
      await writeText(text);
      flashToast(t("toast.copiedTranscript"));
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  async function handleExportFile(includeTranscript: boolean) {
    if (!selectedId || !detail) return;
    try {
      const md = await api.exportMarkdown(selectedId, includeTranscript);
      const safe = sanitizeFilename(detail.meeting.title);
      const path = await save({
        defaultPath: `${safe}.md`,
        filters: [{ name: t("dialog.markdown"), extensions: ["md"] }],
      });
      if (!path) return;
      await api.writeTextFile(path, md);
      flashToast(t("toast.exportedMarkdown"));
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  async function handleExportDocx(includeTranscript: boolean) {
    if (!selectedId || !detail) return;
    try {
      const safe = sanitizeFilename(detail.meeting.title);
      const path = await save({
        defaultPath: `${safe}.docx`,
        filters: [{ name: t("dialog.word"), extensions: ["docx"] }],
      });
      if (!path) return;
      await api.exportDocx(selectedId, path, includeTranscript);
      flashToast(t("toast.exportedWord"));
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  async function handleExportPdf(includeTranscript: boolean) {
    if (!selectedId || !detail) return;
    try {
      const safe = sanitizeFilename(detail.meeting.title);
      const path = await save({
        defaultPath: `${safe}.pdf`,
        filters: [{ name: t("dialog.pdf"), extensions: ["pdf"] }],
      });
      if (!path) return;
      await api.exportPdf(selectedId, path, includeTranscript);
      flashToast(t("toast.exportedPdf"));
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  /** Save-to-device: one entry point, dispatching to the per-format writers. */
  async function handleSaveToDevice(
    format: ShareFormat,
    includeTranscript: boolean,
  ) {
    if (format === "md") return handleExportFile(includeTranscript);
    if (format === "docx") return handleExportDocx(includeTranscript);
    return handleExportPdf(includeTranscript);
  }

  /** Send-to-an-app: no save dialog — the backend stages the file itself. */
  async function handleShare(format: ShareFormat, includeTranscript: boolean) {
    if (!selectedId) return;
    try {
      await api.shareMeeting(selectedId, format, includeTranscript);
    } catch (e) {
      flashToast(errorText(e), TOAST_ERROR_MS);
    }
  }

  const viewingRecording = selectedId !== null && selectedId === recordingId;

  const recMeeting = meetings.find((m) => m.id === recordingId);
  const elapsed = recMeeting
    ? formatDuration(
        meetingDurationMs(recMeeting.created_at, recMeeting.ended_at, true),
      )
    : "00:00:00";

  // A meeting that is still recording gets the recording screen; the same
  // "detail" navigation state otherwise renders the saved meeting view.
  const view: Screen =
    screen === "detail" && viewingRecording ? "recording" : screen;

  const pageTitle =
    view === "home"
      ? t("page.home")
      : view === "notes"
        ? t("page.notes")
        : view === "settings"
          ? t("page.settings")
          : view === "recording"
            ? t("page.recording")
            : (detail?.meeting.title ?? t("page.meeting"));

  const engineMode = settings ? engineModeLabel(settings, t) : null;

  function cycleTheme() {
    const next: ThemePreference =
      themePreference === "light"
        ? "dark"
        : themePreference === "dark"
          ? "system"
          : "light";
    setThemePreferenceState(next);
    setThemePreference(next);
  }

  // ⌘K / Ctrl+K opens the search palette from anywhere in the window.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  /* Dev-only: ⌘⇧O opens first-run setup with a synthetic report, because a real
     one shows itself once and then stamps a marker — and its most important
     states (denied, Windows, Linux) cannot be produced on this machine at all.
     A Tauri window has no address bar, so a shortcut rather than a URL param.
     The whole effect is compiled out of a release build. */
  useEffect(() => {
    if (!import.meta.env.DEV) return;
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "o") {
        e.preventDefault();
        setOnboardingSim("fresh");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Nothing app-shaped until we know whether setup is due.
  if (!onboardingChecked) return <div className="app app-booting" />;

  // Dev simulation wins over the real report, and never persists anything.
  if (import.meta.env.DEV && onboardingSim) {
    return (
      <OnboardingScreen
        // Remount on scenario change: the step count differs between scenarios,
        // so carrying the old cursor over could land past the last step.
        key={onboardingSim}
        report={simulationReport(onboardingSim)}
        simulation={onboardingSim}
        onSimulationChange={setOnboardingSim}
        onFinished={() => setOnboardingSim(null)}
      />
    );
  }

  if (onboarding) {
    return (
      <OnboardingScreen
        report={onboarding}
        onFinished={() => {
          setOnboarding(null);
          // The marker moved, so the cached view is stale.
          api.getSettings().then(setSettings).catch(() => {});
        }}
      />
    );
  }

  return (
    <div className="app">
      <NavRail
        screen={view}
        collapsed={railCollapsed}
        busy={busy}
        onHome={() => setScreen("home")}
        onNotes={() => setScreen("notes")}
        onSettings={openSettings}
        onNewMeeting={handleStart}
      />

      <div className="main-col">
        <TopBar
          title={pageTitle}
          themePreference={themePreference}
          recording={
            recordingId
              ? {
                  elapsed,
                  busy,
                  onOpen: () => {
                    setSelectedId(recordingId);
                    setScreen("detail");
                  },
                  onStop: handleStop,
                }
              : null
          }
          onToggleRail={() => setRailCollapsed((c) => !c)}
          onOpenPalette={() => setPaletteOpen(true)}
          onCycleTheme={cycleTheme}
        />

        <main className="main">
          {view === "home" && (
            <HomeScreen
              meetings={meetings}
              recordingId={recordingId}
              busy={busy}
              onNewMeeting={handleStart}
              onOpen={openMeeting}
              onViewAll={() => setScreen("notes")}
            />
          )}

          {view === "notes" && (
            <NotesScreen
              meetings={meetings}
              searchResults={searchResults}
              searchQuery={searchQuery}
              onSearchChange={setSearchQuery}
              selectedId={selectedId}
              recordingId={recordingId}
              onOpen={openMeeting}
              onDelete={handleDelete}
              onNewMeeting={handleStart}
            />
          )}

          {view === "recording" && detail && (
            <RecordingScreen
              detail={detail}
              elapsed={elapsed}
              level={level}
              partialText={partialText}
              engineMode={engineMode}
              busy={busy}
              onStop={handleStop}
              onBack={() => setScreen("notes")}
            />
          )}

          {view === "detail" && detail && (
            <MeetingView
              detail={detail}
              highlightQuery={searchResults !== null ? searchQuery.trim() : ""}
              summarizing={summarizing}
              summaryError={summaryError}
              instructions={selectedId ? (summaryInstructions[selectedId] ?? "") : ""}
              hasGlobalInstructions={!!settings?.summary_instructions?.trim()}
              onInstructionsChange={(value) => {
                if (!selectedId) return;
                setSummaryInstructions((prev) => ({ ...prev, [selectedId]: value }));
              }}
              onGenerateSummary={handleGenerateSummary}
              onCopySummary={handleCopySummary}
              onCopyTranscript={handleCopyTranscript}
              shareSupported={!!settings?.share_supported}
              onShare={handleShare}
              onSave={handleSaveToDevice}
              onDelete={() => selectedId && handleDelete(selectedId)}
              onBack={() => setScreen("notes")}
            />
          )}

          {view === "settings" && settings && (
            <SettingsScreen
              current={settings}
              onClose={leaveSettings}
              onSaved={(s) => setSettings(s)}
              onRerunOnboarding={handleRerunOnboarding}
            />
          )}
        </main>
      </div>

      {view === "settings" && !settings && (
        <LoadingModal
          label={t("settingsLoading.label")}
          message={t("settingsLoading.message")}
          dismissible={!settingsLoading}
          onDismiss={leaveSettings}
        />
      )}

      {paletteOpen && (
        <CommandPalette
          query={searchQuery}
          onQueryChange={setSearchQuery}
          searchResults={searchResults}
          meetings={meetings}
          onOpen={openMeeting}
          onClose={() => setPaletteOpen(false)}
        />
      )}

      {/* Always-present live region so screen readers announce status/errors. */}
      <div className="toast-region" role="status" aria-live="polite" aria-atomic="true">
        {toast && (
          <div className="toast">
            <span className="toast-glyph" aria-hidden="true">
              i
            </span>
            <span>{toast}</span>
          </div>
        )}
      </div>
    </div>
  );
}
