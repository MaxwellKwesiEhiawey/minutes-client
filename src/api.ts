import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AudioDevicesResponse,
  Meeting,
  MeetingDetail,
  MeetingListItem,
  MeetingSearchHit,
  SettingsInput,
  ShareFormat,
  SettingsView,
  Summary,
  ServerStatus,
  TranscriptionStatus,
  InstalledModelsInfo,
  StatusEvent,
  PartialEvent,
  FinalEvent,
  ErrorEvent,
  LevelEvent,
  CaptureNoticeEvent,
  ModelProgressEvent,
  PermissionsReport,
  PermissionState,
  PrivacyPane,
} from "./types";

export const api = {
  startRecording: (title?: string) =>
    invoke<Meeting>("start_recording", { title: title ?? null }),
  showNewMeetingPrompt: () => invoke<void>("show_new_meeting_prompt"),
  stopRecording: () => invoke<string>("stop_recording"),
  recordingState: () => invoke<string | null>("recording_state"),
  listAudioDevices: () => invoke<AudioDevicesResponse>("list_audio_devices"),

  listMeetings: () => invoke<MeetingListItem[]>("list_meetings"),
  searchMeetings: (query: string) =>
    invoke<MeetingSearchHit[]>("search_meetings", { query }),
  getMeeting: (meetingId: string) =>
    invoke<MeetingDetail>("get_meeting", { meetingId }),
  deleteMeeting: (meetingId: string) =>
    invoke<void>("delete_meeting", { meetingId }),
  renameMeeting: (meetingId: string, title: string) =>
    invoke<void>("rename_meeting", { meetingId, title }),

  generateSummary: (meetingId: string, instructions?: string) =>
    invoke<Summary>("generate_summary", {
      meetingId,
      instructions: instructions?.trim() ? instructions.trim() : null,
    }),

  getSettings: () => invoke<SettingsView>("get_settings"),
  checkServer: () => invoke<ServerStatus>("check_server"),
  saveSettings: (input: SettingsInput) =>
    invoke<SettingsView>("save_settings", { input }),

  transcriptionStatus: (model?: string) =>
    invoke<TranscriptionStatus>("transcription_status", { model: model ?? null }),
  downloadModels: (model?: string) =>
    invoke<TranscriptionStatus>("download_models", { model: model ?? null }),
  listInstalledModels: () => invoke<InstalledModelsInfo>("list_installed_models"),
  deleteInstalledModel: (modelId: string) =>
    invoke<InstalledModelsInfo>("delete_installed_model", { modelId }),

  exportMarkdown: (meetingId: string, includeTranscript: boolean) =>
    invoke<string>("export_markdown", { meetingId, includeTranscript }),
  writeTextFile: (path: string, contents: string) =>
    invoke<void>("write_text_file", { path, contents }),
  exportDocx: (meetingId: string, path: string, includeTranscript: boolean) =>
    invoke<void>("export_docx", { meetingId, path, includeTranscript }),
  exportPdf: (meetingId: string, path: string, includeTranscript: boolean) =>
    invoke<void>("export_pdf", { meetingId, path, includeTranscript }),
  shareMeeting: (
    meetingId: string,
    format: ShareFormat,
    includeTranscript: boolean,
  ) => invoke<void>("share_meeting", { meetingId, format, includeTranscript }),

  /* First-run permission onboarding. `permissionStatus` never prompts, so it is
     safe to call on mount and again after every grant. */
  permissionStatus: () => invoke<PermissionsReport>("permission_status"),
  requestMicrophone: () => invoke<PermissionState>("request_microphone"),
  requestBrowserAutomation: (appName: string) =>
    invoke<PermissionState>("request_browser_automation", { appName }),
  openPrivacySettings: (pane: PrivacyPane) =>
    invoke<void>("open_privacy_settings", { pane }),
  completeOnboarding: () => invoke<SettingsView>("complete_onboarding"),
  resetOnboarding: () => invoke<PermissionsReport>("reset_onboarding"),
};

export const events = {
  onStatus: (cb: (e: StatusEvent) => void): Promise<UnlistenFn> =>
    listen<StatusEvent>("recording-status", (e) => cb(e.payload)),
  onPartial: (cb: (e: PartialEvent) => void): Promise<UnlistenFn> =>
    listen<PartialEvent>("transcript-partial", (e) => cb(e.payload)),
  onFinal: (cb: (e: FinalEvent) => void): Promise<UnlistenFn> =>
    listen<FinalEvent>("transcript-final", (e) => cb(e.payload)),
  onError: (cb: (e: ErrorEvent) => void): Promise<UnlistenFn> =>
    listen<ErrorEvent>("transcript-error", (e) => cb(e.payload)),
  onLevel: (cb: (e: LevelEvent) => void): Promise<UnlistenFn> =>
    listen<LevelEvent>("audio-level", (e) => cb(e.payload)),
  onCaptureNotice: (cb: (e: CaptureNoticeEvent) => void): Promise<UnlistenFn> =>
    listen<CaptureNoticeEvent>("capture-notice", (e) => cb(e.payload)),
  onModelProgress: (cb: (e: ModelProgressEvent) => void): Promise<UnlistenFn> =>
    listen<ModelProgressEvent>("model-download-progress", (e) => cb(e.payload)),
  onMeetingStarted: (cb: (meeting: Meeting) => void): Promise<UnlistenFn> =>
    listen<Meeting>("meeting-started", (e) => cb(e.payload)),
};
