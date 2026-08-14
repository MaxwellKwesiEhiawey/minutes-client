import { useEffect, useRef, type ReactNode } from "react";
import { useFocusTrap } from "../useFocusTrap";

interface Props {
  /** Accessible name. Use `labelledBy` instead when a visible heading exists. */
  label?: string;
  labelledBy?: string;
  /** Extra classes on the card, e.g. "palette" or "modal-wide". */
  className?: string;
  /** Backdrop click and Escape both call this; omit for a blocking modal. */
  onClose?: () => void;
  children: ReactNode;
}

/**
 * The app's single modal shell: the design's blurred veil plus a 16px-radius
 * card, with focus trapped inside. Every dialog surface renders through this so
 * the chrome and the escape/backdrop behaviour can't drift between them.
 */
export function Modal({
  label,
  labelledBy,
  className,
  onClose,
  children,
}: Props) {
  const ref = useRef<HTMLDivElement>(null);
  useFocusTrap(ref);

  useEffect(() => {
    if (!onClose) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="modal-backdrop" onClick={() => onClose?.()}>
      <div
        className={className ? `modal ${className}` : "modal"}
        role="dialog"
        aria-modal="true"
        aria-label={labelledBy ? undefined : label}
        aria-labelledby={labelledBy}
        ref={ref}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </div>
  );
}
