import type { ReactNode } from "react";

interface IconProps {
  size?: number;
  className?: string;
}

function IconBase({
  size = 18,
  className,
  strokeWidth = 2,
  children,
}: IconProps & { strokeWidth?: number; children: ReactNode }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

/** The design's stroke weight for navigation and chrome icons. */
function IconThin({ children, ...props }: IconProps & { children: ReactNode }) {
  return (
    <IconBase {...props} strokeWidth={1.75}>
      {children}
    </IconBase>
  );
}

/** Brand mark: the four-bar Minutes glyph. Inherits colour from `currentColor`. */
export function BrandMark({ size = 26, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 268 228"
      fill="none"
      stroke="currentColor"
      strokeWidth="28"
      strokeLinecap="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M20 34v160" />
      <path d="M90 14v200" />
      <path d="M160 54v120" />
      <path d="M230 84v60" />
    </svg>
  );
}

export function IconHome(props: IconProps) {
  return (
    <IconThin {...props}>
      <path d="M3 12l9-9 9 9" />
      <path d="M5 10v10h14V10" />
    </IconThin>
  );
}

export function IconNotes(props: IconProps) {
  return (
    <IconThin {...props}>
      <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
      <path d="M14 3v5h5" />
      <path d="M9 13h6" />
      <path d="M9 17h4" />
    </IconThin>
  );
}

export function IconPanel(props: IconProps) {
  return (
    <IconThin {...props}>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M9.5 4v16" />
    </IconThin>
  );
}

export function IconSun(props: IconProps) {
  return (
    <IconThin {...props}>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5L19 19M19 5l-1.5 1.5M6.5 17.5L5 19" />
    </IconThin>
  );
}

export function IconMoon(props: IconProps) {
  return (
    <IconThin {...props}>
      <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5z" />
    </IconThin>
  );
}

export function IconPlus(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 5v14" />
      <path d="M5 12h14" />
    </IconBase>
  );
}

export function IconChevronLeft(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M15 6l-6 6 6 6" />
    </IconBase>
  );
}

export function IconDownload(props: IconProps) {
  return (
    <IconThin {...props}>
      <path d="M12 4v11" />
      <path d="M8 11l4 4 4-4" />
      <path d="M5 19h14" />
    </IconThin>
  );
}

export function IconDots(props: IconProps) {
  return (
    <IconBase {...props}>
      <circle cx="12" cy="5" r="1.3" />
      <circle cx="12" cy="12" r="1.3" />
      <circle cx="12" cy="19" r="1.3" />
    </IconBase>
  );
}

export function IconCheck(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M5 12.5l4.5 4.5L19 7.5" />
    </IconBase>
  );
}

export function IconOpen(props: IconProps) {
  return (
    <IconThin {...props}>
      <path d="M9 6l6 6-6 6" />
    </IconThin>
  );
}

export function IconSettings(props: IconProps) {
  return (
    <IconBase {...props}>
      <circle cx="12" cy="12" r="3" />
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
      />
    </IconBase>
  );
}

export function IconTrash(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
      <path d="M10 11v6M14 11v6" />
    </IconBase>
  );
}

export function IconRecord(props: IconProps) {
  return (
    <IconBase {...props} size={props.size ?? 16}>
      <circle cx="12" cy="12" r="6" fill="currentColor" stroke="none" />
    </IconBase>
  );
}

export function IconSearch(props: IconProps) {
  return (
    <IconBase {...props}>
      <circle cx="11" cy="11" r="7" />
      <path d="M20 20l-3-3" />
    </IconBase>
  );
}

export function IconMic(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2M12 19v3M8 22h8" />
    </IconBase>
  );
}

export function IconTranscript(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8" />
    </IconBase>
  );
}

export function IconSummary(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 3l1.5 4.5L18 9l-4.5 1.5L12 15l-1.5-4.5L6 9l4.5-1.5L12 3z" />
      <path d="M5 19h14M5 22h14" />
    </IconBase>
  );
}

/**
 * Share: a tray with the arrow pointing *out* of it.
 *
 * The download glyph (an arrow pointing down into a line) reads as "bring this
 * onto my machine", which is the opposite of what this button does — it opens
 * copy-to-clipboard and export-a-file. The arrow direction is the whole signal,
 * so the shaft is drawn from the tray upward and the tray is open at the top.
 */
export function IconShare(props: IconProps) {
  return (
    <IconThin {...props}>
      <path d="M8 11H6a1.5 1.5 0 0 0-1.5 1.5v6A1.5 1.5 0 0 0 6 20h12a1.5 1.5 0 0 0 1.5-1.5v-6A1.5 1.5 0 0 0 18 11h-2" />
      <path d="M12 15V4" />
      <path d="M8.5 7.5L12 4l3.5 3.5" />
    </IconThin>
  );
}

export function IconSparkle(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 3l1.2 3.6L17 8l-3.8 1.2L12 13l-1.2-3.8L7 8l3.8-1.2L12 3z" />
      <path d="M5 17l.6 1.8L7.4 19l-1.8.6L5 21.4l-.6-1.8L2.6 19l1.8-.6L5 17z" />
      <path d="M19 15l.6 1.8L21.4 17l-1.8.6L19 19.4l-.6-1.8L16.6 17l1.8-.6L19 15z" />
    </IconBase>
  );
}

export function IconClose(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M18 6 6 18M6 6l12 12" />
    </IconBase>
  );
}

export function IconEmpty(props: IconProps) {
  return (
    <IconBase {...props} size={props.size ?? 48}>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M7 8h10M7 12h6" />
    </IconBase>
  );
}
