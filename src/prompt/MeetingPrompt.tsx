import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BrandMark, IconClose, IconRecord } from "../components/Icons";
import { useT } from "../i18n";

type PromptKind = "manual" | "call";

interface MeetingPromptData {
  kind: PromptKind;
  app_name: string | null;
  process_name: string | null;
  suggested_title: string | null;
}

function tokenFromUrl(): number | null {
  const raw = new URLSearchParams(window.location.search).get("t");
  if (!raw) return null;
  const n = Number(raw);
  return Number.isFinite(n) ? n : null;
}

/**
 * The card's top strip. `data-tauri-drag-region` makes the window follow the
 * pointer from here, the way Teams' call toast can be moved out of the way of
 * whatever is underneath it. Only this strip drags: a drag region over the
 * buttons or the title field would swallow their clicks.
 */
function DragStrip({ children }: { children: React.ReactNode }) {
  return (
    <div className="mp-strip" data-tauri-drag-region>
      {children}
    </div>
  );
}

export function MeetingPrompt() {
  const t = useT();
  const [data, setData] = useState<MeetingPromptData | null>(null);
  const [title, setTitle] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  // Synchronous re-entry guard. `busy` is React state and is not applied
  // until the next render, so two events in the same tick (double click,
  // held-down Enter auto-repeat) would both pass a `busy` check and invoke
  // start_recording twice. The ref flips synchronously and closes that gap.
  const busyRef = useRef(false);

  useEffect(() => {
    const token = tokenFromUrl();
    if (token == null) {
      setLoadFailed(true);
      void invoke("close_meeting_prompt");
      return;
    }
    (async () => {
      try {
        const payload = await invoke<MeetingPromptData | null>("get_meeting_prompt", {
          token,
        });
        if (!payload) {
          // Don't auto-close on a missing payload — a remount race used to
          // wipe the staged token and immediately destroy the window.
          setLoadFailed(true);
          setError(t("prompt.loadFailed"));
          return;
        }
        setData(payload);
        setTitle(payload.suggested_title?.trim() ?? "");
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        setLoadFailed(true);
      }
    })();
    // `t` is stable per locale, so this still runs once per window.
  }, [t]);

  async function dismiss() {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    try {
      await invoke("dismiss_meeting_prompt", {
        processName: data?.process_name ?? null,
      });
    } catch {
      await invoke("close_meeting_prompt").catch(() => undefined);
    }
  }

  async function start() {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    setError(null);
    try {
      await invoke("start_recording_from_prompt", {
        title: title.trim() ? title.trim() : null,
        processName: data?.process_name ?? null,
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      busyRef.current = false;
      setBusy(false);
    }
  }

  if (loadFailed && !data) {
    return (
      <div className="mp">
        <DragStrip>
          <BrandMark size={16} className="mp-mark" />
          <span className="mp-wordmark" data-tauri-drag-region>
            Minutes
          </span>
          <span className="mp-grip" data-tauri-drag-region aria-hidden="true" />
          <button
            type="button"
            className="mp-close"
            onClick={() => void invoke("close_meeting_prompt")}
            aria-label={t("common.close")}
          >
            <IconClose size={15} />
          </button>
        </DragStrip>
        <div className="mp-body">
          <h1 className="mp-title">{t("prompt.errorHeading")}</h1>
          <p className="mp-sub">{error ?? t("prompt.errorBody")}</p>
        </div>
        <div className="mp-actions">
          <span className="mp-hint" />
          <button
            type="button"
            className="mp-btn"
            onClick={() => void invoke("close_meeting_prompt")}
          >
            {t("common.close")}
          </button>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="mp">
        <DragStrip>
          <BrandMark size={16} className="mp-mark" />
          <span className="mp-wordmark" data-tauri-drag-region>
            Minutes
          </span>
          <span className="mp-grip" data-tauri-drag-region aria-hidden="true" />
        </DragStrip>
        <div className="mp-body mp-body-loading">
          <span className="spinner" aria-hidden="true" />
          <span className="mp-sub">{t("common.loading")}</span>
        </div>
      </div>
    );
  }

  const isCall = data.kind === "call";
  const source = data.app_name ?? t("prompt.call");
  const status = isCall
    ? t("prompt.callDetected", { app: source })
    : t("prompt.newMeeting");
  const heading = isCall ? t("prompt.callHeading") : t("prompt.manualHeading");
  const sub = isCall ? t("prompt.callSub") : t("prompt.manualSub");
  const primaryLabel = isCall
    ? t("prompt.takeNotes")
    : t("prompt.startRecording");

  return (
    <div className={isCall ? "mp is-call" : "mp"}>
      <DragStrip>
        <BrandMark size={16} className="mp-mark" />
        <span className="mp-wordmark" data-tauri-drag-region>
          Minutes
        </span>
        {/* The grip is the affordance: it says "this card can be moved". */}
        <span className="mp-grip" data-tauri-drag-region aria-hidden="true" />
        <button
          type="button"
          className="mp-close"
          onClick={dismiss}
          aria-label={t("prompt.dismiss")}
          disabled={busy}
        >
          <IconClose size={15} />
        </button>
      </DragStrip>

      <div className="mp-body">
        <span className="mp-status">
          <span className="mp-status-dot" aria-hidden="true" />
          {status}
        </span>
        <h1 className="mp-title">{heading}</h1>
        <p className="mp-sub">{sub}</p>

        <label className="mp-label" htmlFor="meeting-title">
          {t("prompt.meetingTitle")}
        </label>
        <input
          id="meeting-title"
          className="mp-input"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder={
            isCall
              ? t("prompt.callPlaceholder", { app: source })
              : t("prompt.manualPlaceholder")
          }
          disabled={busy}
          autoFocus
          onKeyDown={(e) => {
            // Ignore OS key auto-repeat: holding Enter must not fire a
            // second start while the first is still in flight.
            if (e.repeat) return;
            if (e.key === "Enter") void start();
            if (e.key === "Escape") void dismiss();
          }}
        />
        {error && <p className="mp-error">{error}</p>}
      </div>

      <div className="mp-actions">
        <span className="mp-hint" aria-hidden="true">
          <kbd>↵</kbd> {t("prompt.hintStart")}{" "}
          <span className="mp-hint-dot">·</span> <kbd>esc</kbd>{" "}
          {t("prompt.hintClose")}
        </span>
        <button type="button" className="mp-btn" onClick={dismiss} disabled={busy}>
          {t("prompt.notNow")}
        </button>
        <button
          type="button"
          className="mp-btn primary"
          onClick={start}
          disabled={busy}
        >
          {busy ? (
            <>
              <span className="spinner" aria-hidden="true" />
              {t("common.starting")}
            </>
          ) : (
            <>
              <IconRecord size={9} />
              {primaryLabel}
            </>
          )}
        </button>
      </div>
    </div>
  );
}
