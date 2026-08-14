import { useState } from "react";
import type { ShareFormat } from "../types";
import { IconClose, IconDownload, IconShare } from "./Icons";
import { Modal } from "./Modal";
import { useT, type TranslationKey } from "../i18n";

const FORMATS: { value: ShareFormat; label: TranslationKey }[] = [
  { value: "pdf", label: "share.formatPdf" },
  { value: "docx", label: "share.formatDocx" },
  { value: "md", label: "share.formatMd" },
];

interface Props {
  hasSummary: boolean;
  hasTranscript: boolean;
  /** Whether this platform has an OS share picker (macOS and Windows do). */
  shareSupported: boolean;
  onClose: () => void;
  onCopySummary: () => void;
  onCopyTranscript: () => void;
  onShare: (format: ShareFormat, includeTranscript: boolean) => void;
  onSave: (format: ShareFormat, includeTranscript: boolean) => void;
}

export function ShareModal({
  hasSummary,
  hasTranscript,
  shareSupported,
  onClose,
  onCopySummary,
  onCopyTranscript,
  onShare,
  onSave,
}: Props) {
  const t = useT();
  // Off by default: the verbatim record is the sensitive half, so putting it in
  // a file someone is about to send should be a deliberate act rather than
  // something that happens because the default was convenient.
  const [includeTranscript, setIncludeTranscript] = useState(false);

  // Deliberately no default format. Both destinations stay unavailable until a
  // format is chosen, so nothing is ever sent or saved as a type nobody picked.
  const [format, setFormat] = useState<ShareFormat | "">("");

  // Dropping the transcript only makes sense when the summary can carry the
  // file on its own. With no summary the transcript is the entire document, so
  // the switch is locked on rather than allowed to produce an empty export.
  const canOmitTranscript = hasSummary && hasTranscript;
  const withTranscript = canOmitTranscript ? includeTranscript : true;

  // A file needs something in it: either half will do.
  const canExport = hasSummary || hasTranscript;
  const ready = canExport && format !== "";

  function run(action: (format: ShareFormat, includeTranscript: boolean) => void) {
    // Narrowed here, so `""` never reaches a callback.
    if (format === "") return;
    action(format, withTranscript);
    onClose();
  }

  return (
    <Modal labelledBy="share-title" onClose={onClose}>
      <div className="modal-header">
        <h2 id="share-title">{t("share.title")}</h2>
        <button
          type="button"
          className="icon-btn"
          onClick={onClose}
          aria-label={t("common.close")}
          title={t("common.close")}
        >
          <IconClose size={18} />
        </button>
      </div>

      <div className="modal-body">
        <div className="share-option">
          <span className="share-option-text">
            <span className="share-option-label" id="share-include-transcript-label">
              {t("share.includeTranscript")}
            </span>
            <span className="share-option-hint">
              {canOmitTranscript
                ? withTranscript
                  ? t("share.includeOn")
                  : t("share.includeOff")
                : hasTranscript
                  ? t("share.includeForced")
                  : t("share.includeNone")}
            </span>
          </span>
          <button
            type="button"
            role="switch"
            aria-checked={withTranscript}
            aria-labelledby="share-include-transcript-label"
            className={withTranscript ? "st-toggle on" : "st-toggle"}
            disabled={!canOmitTranscript}
            onClick={() => setIncludeTranscript((v) => !v)}
          >
            <span className="st-toggle-knob" />
          </button>
        </div>

        <div className="share-option">
          <span className="share-option-text">
            <label className="share-option-label" htmlFor="share-format">
              {t("share.format")}
            </label>
            <span className="share-option-hint">
              {t("share.formatHint")}
            </span>
          </span>
          <select
            id="share-format"
            value={format}
            onChange={(e) => setFormat(e.target.value as ShareFormat)}
          >
            <option value="" disabled>
              {t("share.formatPlaceholder")}
            </option>
            {FORMATS.map((f) => (
              <option key={f.value} value={f.value}>
                {t(f.label)}
              </option>
            ))}
          </select>
        </div>

        <div className="share-destinations">
          {shareSupported && (
            <button
              type="button"
              className="btn primary"
              onClick={() => run(onShare)}
              disabled={!ready}
              title={t("share.sendToAppTitle")}
            >
              <IconShare size={15} />
              {t("share.sendToApp")}
            </button>
          )}
          <button
            type="button"
            className="btn"
            onClick={() => run(onSave)}
            disabled={!ready}
            title={t("share.saveToDeviceTitle")}
          >
            <IconDownload size={15} />
            {t("share.saveToDevice")}
          </button>
        </div>

        {/* A disabled button with no explanation is its own usability problem. */}
        {canExport && format === "" && (
          <p className="share-gate-hint">{t("share.gateHint")}</p>
        )}
        {!canExport && (
          <p className="share-gate-hint">{t("share.nothingToShare")}</p>
        )}

        <div className="share-group">
          <h4>{t("share.copyGroup")}</h4>
          <div className="share-actions">
            <button
              type="button"
              className="btn"
              onClick={() => {
                onCopySummary();
                onClose();
              }}
              disabled={!hasSummary}
              title={t("share.copySummaryTitle")}
            >
              {t("share.copySummary")}
            </button>
            <button
              type="button"
              className="btn"
              onClick={() => {
                onCopyTranscript();
                onClose();
              }}
              disabled={!hasTranscript}
              title={t("share.copyTranscriptTitle")}
            >
              {t("share.copyTranscript")}
            </button>
          </div>
        </div>
      </div>

      <div className="modal-actions">
        <button type="button" className="btn ghost" onClick={onClose}>
          {t("common.close")}
        </button>
      </div>
    </Modal>
  );
}
