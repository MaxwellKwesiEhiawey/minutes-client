import { Modal } from "./Modal";

interface Props {
  label: string;
  message: string;
  /** Called on backdrop click / Escape; ignored while `dismissible` is false. */
  onDismiss?: () => void;
  dismissible?: boolean;
}

/** Transient modal shown while data loads. */
export function LoadingModal({
  label,
  message,
  onDismiss,
  dismissible = true,
}: Props) {
  return (
    <Modal
      label={label}
      onClose={dismissible ? onDismiss : undefined}
    >
      <div className="settings-loading">
        <span className="spinner" aria-hidden="true" />
        <span>{message}</span>
      </div>
    </Modal>
  );
}
