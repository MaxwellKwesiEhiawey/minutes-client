export type MeetingStatus = "recording" | "completed" | "interrupted";

export interface Meeting {
  id: string;
  title: string;
  status: MeetingStatus;
  created_at: string;
  ended_at: string | null;
}

export interface Segment {
  id: number;
  meeting_id: string;
  seq: number;
  text: string;
  created_at: string;
  speaker_label: string | null;
  speaker_name: string | null;
  start_ms: number | null;
  end_ms: number | null;
}

export interface MeetingListItem extends Meeting {
  segment_count: number;
  has_summary: boolean;
}

export interface MeetingSearchHit extends MeetingListItem {
  snippet: string | null;
}

export interface KeyTopic {
  topic: string;
  bullets: string[];
}

export interface Decision {
  text: string;
  owner: string | null;
}

export interface ActionItem {
  task: string;
  assignee: string | null;
  due: string | null;
}

export interface SummaryContent {
  title: string;
  executive_summary: string;
  key_topics: KeyTopic[];
  decisions: Decision[];
  action_items: ActionItem[];
  open_questions: string[];
}

export interface Summary {
  meeting_id: string;
  content: SummaryContent;
  model: string;
  created_at: string;
}

export interface MeetingDetail {
  meeting: Meeting;
  segments: Segment[];
  summary: Summary | null;
}

export type AudioDeviceKind = "loopback" | "microphone" | "unknown";

export interface AudioInputDevice {
  name: string;
  kind: AudioDeviceKind;
  label: string;
}

export interface AudioDevicesResponse {
  platform: string;
  devices: AudioInputDevice[];
  has_loopback: boolean;
}

export type WhisperModel = "tiny" | "base" | "small" | "medium" | "large-v3";

export type TranscriptionEngine = "whisper" | "deepgram";

/** File format for a save or a share. Kept in one place so the modal, App and
 *  the api bridge cannot drift from the Rust `ShareFormat` enum. */
export type ShareFormat = "pdf" | "docx" | "md";

export interface SettingsView {
  server_url: string;
  whisper_model: string;
  transcription_engine: TranscriptionEngine;
  diarization_enabled: boolean;
  export_markdown: boolean;
  anthropic_model: string;
  chunk_secs: number;
  partial_secs: number;
  capture_microphone: boolean;
  input_device: string | null;
  capture_system_audio: boolean;
  system_audio_device: string | null;
  summary_instructions: string;
  transcription_language: string;
  summary_language: string;
  auto_summarize: boolean;
  onboarding_completed_version: number;
  call_detection_enabled: boolean;
  call_detection_cooldown_minutes: number;
  call_detection_poll_interval_secs: number;
  call_detection_apps: string[];
  call_detection_supported: boolean;
  share_supported: boolean;
  telemetry_enabled: boolean;
  server_url_from_env: boolean;
  server_url_from_build: boolean;
  server_token_present: boolean;
  server_token_from_env: boolean;
  server_token_from_build: boolean;
}

export interface SettingsInput {
  server_url?: string;
  server_token?: string;
  whisper_model?: string;
  transcription_engine?: TranscriptionEngine;
  diarization_enabled?: boolean;
  export_markdown?: boolean;
  anthropic_model?: string;
  chunk_secs?: number;
  partial_secs?: number;
  capture_microphone?: boolean;
  input_device?: string;
  capture_system_audio?: boolean;
  system_audio_device?: string;
  summary_instructions?: string;
  transcription_language?: string;
  summary_language?: string;
  auto_summarize?: boolean;
  call_detection_enabled?: boolean;
  call_detection_cooldown_minutes?: number;
  call_detection_poll_interval_secs?: number;
  telemetry_enabled?: boolean;
}

export interface TranscriptionStatus {
  model: string;
  model_ready: boolean;
  diarization_enabled: boolean;
}

export interface InstalledModelEntry {
  id: string;
  kind: string;
  label: string;
  size_bytes: number;
  in_use: boolean;
}

export interface InstalledModelsInfo {
  models: InstalledModelEntry[];
  models_dir: string;
  total_bytes: number;
}

export interface ModelProgressEvent {
  stage: string;
  label: string;
  downloaded: number;
  total: number | null;
  done: boolean;
}

export interface ServerStatus {
  configured: boolean;
  reachable: boolean;
  message: string;
}

/* First-run permission onboarding. Mirrors `src-tauri/src/permissions.rs`; the
 * step list is decided in Rust so the UI never re-derives gating from raw
 * booleans. */
export type PermissionState =
  | "granted"
  | "denied"
  | "notDetermined"
  | "notApplicable"
  | "unknown";

export type OnboardingStepId =
  | "microphone"
  | "browserDetection"
  | "detectionUnavailable";

/** Which OS privacy page to open. An enum, never a URL — the mapping lives in
 *  Rust so a frontend string can never become something the app opens. */
export type PrivacyPane = "microphone" | "automation";

export interface BrowserPermission {
  /** Installed browsers only, and always a name the detector actually probes. */
  appName: string;
  state: PermissionState;
}

export interface PermissionsReport {
  onboardingRequired: boolean;
  steps: OnboardingStepId[];
  completedVersion: number;
  currentVersion: number;
  microphone: PermissionState;
  browsers: BrowserPermission[];
  /** `macos` | `windows` | `linux` | other, for platform-specific copy. */
  platform: string;
}

// Event payloads emitted from Rust.
export interface StatusEvent {
  meetingId: string;
  status: MeetingStatus;
}
export interface PartialEvent {
  meetingId: string;
  text: string;
}
export interface FinalEvent {
  meetingId: string;
  segment: Segment;
}
export interface ErrorEvent {
  meetingId: string;
  message: string;
  /** Translation key when the failure is one the user can act on. */
  code?: string | null;
}
export interface LevelEvent {
  meetingId: string;
  level: number;
}
/** Non-fatal capture news, e.g. a device swapped mid-recording. `meetingId` is
 *  null when the notice arrives during the initial device open, before the
 *  meeting row exists. */
export interface CaptureNoticeEvent {
  meetingId: string | null;
  message: string;
}
