/**
 * English — the source of truth for every UI string.
 *
 * Every other dictionary is typed `Translations`, i.e. `Record<keyof typeof en,
 * string>`, so adding a key here without translating it is a compile error in
 * the six other files rather than a missing string on screen.
 *
 * Keys are flat and dotted, named for where the string appears. Placeholders are
 * `{name}` and are substituted by `translatorFor`.
 *
 * Not covered here: messages that originate in the Rust backend (transcription
 * errors, the server-connection status line, Whisper model labels). Those arrive
 * already-formed over IPC and would need the backend localized too — see the
 * note in the language row's hint.
 */
export const en = {
  /* ---------------- Shared ---------------- */
  "common.close": "Close",
  "common.cancel": "Cancel",
  "common.delete": "Delete",
  "common.done": "Done",
  "common.open": "Open",
  "common.tryAgain": "Try again",
  "common.retrying": "Retrying…",
  "common.loading": "Loading…",
  "common.yes": "Yes",
  "common.none": "—",
  "common.starting": "Starting…",

  /* ---------------- Navigation rail ---------------- */
  "nav.home": "Home",
  "nav.myNotes": "My Notes",
  "nav.settings": "Settings",
  "nav.newMeeting": "New Meeting",
  "nav.brandHome": "Minutes home",
  "nav.main": "Main",

  /* ---------------- Top bar ---------------- */
  "topbar.toggleSidebar": "Toggle sidebar",
  "topbar.search": "Search meetings and transcripts…",
  "topbar.searchLabel": "Search meetings and transcripts",
  "topbar.themeTitle": "Theme: {theme} — click to change",
  "topbar.recordingOpen": "Go to the meeting being recorded",
  "topbar.recordingStop": "End the meeting being recorded",
  "topbar.stop": "Stop",
  "theme.light": "Light",
  "theme.dark": "Dark",
  "theme.system": "System",

  /* ---------------- Page titles ---------------- */
  "page.home": "Home",
  "page.notes": "My Notes",
  "page.settings": "Settings",
  "page.recording": "Recording",
  "page.meeting": "Meeting",

  /* ---------------- Home ---------------- */
  "home.greeting": "Hello, Welcome!",
  "home.sub": "Ready to turn your next conversation into something useful?",
  "home.recent": "Recent meetings",
  "home.viewAll": "View all notes →",
  "home.summaryReady": "Summary ready",
  "home.transcriptOnly": "Transcript only",
  "home.emptyTitle": "Your meetings will appear here",
  "home.emptyBody":
    "Start your first meeting and let Minutes capture the conversation, generate a summary, and organize everything for you.",
  "home.emptyCta": "Start a Meeting",

  /* ---------------- My Notes ---------------- */
  "notes.title": "My Notes",
  "notes.sub": "Every meeting Minutes has captured on this device.",
  "notes.results": "Results for “{query}”",
  "notes.clearSearchText": "Clear search text",
  "notes.colMeeting": "Meeting",
  "notes.colDate": "Date",
  "notes.colDuration": "Duration",
  "notes.colSummary": "Summary",
  "notes.colStatus": "Status",
  "notes.moreActions": "More actions for {title}",
  "notes.stopBeforeDelete": "Stop recording before deleting",
  "notes.emptySearchTitle": "No meetings match your search",
  "notes.emptySearchBody":
    "Try a different word or phrase — search covers titles and transcript text.",
  "notes.clearSearch": "Clear search",
  "notes.emptyTitle": "No meetings yet",
  "notes.emptyBody":
    "Start a meeting and it will show up here with its transcript and summary.",

  /* ---------------- Meeting status ---------------- */
  "status.recording": "Recording",
  "status.completed": "Completed",
  "status.interrupted": "Interrupted",

  /* ---------------- Meeting detail ---------------- */
  "detail.back": "Back to My Notes",
  "detail.share": "Share and export",
  "detail.delete": "Delete meeting",
  "detail.tabSummary": "Summary",
  "detail.tabTranscript": "Transcription",
  "detail.tabsLabel": "Meeting panels",
  "detail.generate": "Generate summary",
  "detail.regenerate": "Regenerate summary",
  "detail.summarizing": "Summarizing…",
  "detail.generateTitle": "Generate AI summary",
  "detail.generateDisabled": "No transcript captured for this meeting yet",
  "detail.instructionsToggle": "Add instructions",
  "detail.instructionsLabel": "Instructions for this summary (optional)",
  "detail.instructionsPlaceholder":
    "e.g. Do not include the names of people mentioned in the meeting.",
  "detail.instructionsCombined":
    "Combined with your default summary instructions from Settings.",
  "detail.instructionsApplied":
    "Applied when you generate or regenerate the summary.",
  "detail.writingSummary":
    "Writing your summary — this usually takes about a minute.",
  "detail.noSummaryTitle": "No summary yet",
  "detail.noSummaryReady": "Generate one from the transcript when you're ready.",
  "detail.noSummaryNoTranscript":
    "A summary needs a transcript — nothing was captured for this meeting.",
  "detail.noTranscriptTitle": "No transcript captured",
  "detail.noTranscriptBody": "This meeting has no transcript segments.",
  "detail.speaker": "Speaker",

  /* ---------------- Summary errors ---------------- */
  "summaryError.networkTitle": "Could not reach the summarization server.",
  "summaryError.networkHint":
    "Check your network connection and the server URL in Settings, then try again.",
  "summaryError.timeoutTitle": "The summarization server took too long to respond.",
  "summaryError.timeoutHint":
    "This can happen on a slow connection or a very long transcript. Try again.",
  "summaryError.authTitle":
    "The summarization server rejected the request (unauthorized).",
  "summaryError.authHint":
    "Your Minutes access token may be missing or invalid — check Settings, or contact IT.",
  "summaryError.serverTitle": "The summarization server returned an error.",
  "summaryError.genericTitle": "Couldn't generate a summary.",

  /* ---------------- Summary content ---------------- */
  "summary.aiNote": "AI-generated from the transcript · review before sharing",
  "summary.overview": "Overview",
  "summary.keyPoints": "Key discussion points",
  "summary.decisions": "Decisions",
  "summary.actionItems": "Action items",
  "summary.openQuestions": "Open questions",
  "summary.openQuestion": "Open question",
  "summary.owner": "owner: {name}",
  "summary.assignedTo": "Assigned to: {name}",
  "summary.due": "Due: {date}",
  "summary.generatedBy": "Generated by {model} · {date}",

  /* ---------------- Recording ---------------- */
  "recording.back": "Back to My Notes",
  "recording.transcriptSaved": "Transcript is saved live",
  "recording.endMeeting": "End meeting",
  "recording.inputLevel": "Input level",
  "recording.liveTranscript": "Live transcript",
  "recording.savedAsCaptured": "Saved as it is captured",
  "recording.nothingYet": "Nothing captured yet",
  "recording.listening": "Listening — transcript appears as people speak.",
  "recording.interim": "Interim transcript",
  "recording.live": "Live",

  /* ---------------- Engine mode ---------------- */
  "engine.onDevice": "Private · on this device",
  "engine.onDeviceTitle":
    "Transcription runs on this device (Whisper model: {model})",
  "engine.cloud": "Cloud transcription",
  "engine.cloudTitle": "Transcription runs online (Deepgram)",

  /* ---------------- Command palette ---------------- */
  "palette.label": "Search",
  "palette.placeholder": "Search meetings and transcripts…",
  "palette.recent": "Recent",
  "palette.meetings": "Meetings",
  "palette.transcripts": "Transcripts",
  "palette.noResults": "No results for “{query}”",
  "palette.noResultsHint": "Try a speaker name or a phrase from the conversation.",
  "palette.nothingYet": "Nothing to search yet",
  "palette.nothingYetHint": "Record a meeting and it becomes searchable here.",

  /* ---------------- Share & Export ---------------- */
  "share.title": "Share & Export",
  "share.includeTranscript": "Include the full transcript",
  "share.includeOn":
    "The file will contain the summary and everything that was said.",
  "share.includeOff":
    "The file will contain the summary only — nothing anyone said verbatim.",
  "share.includeForced":
    "There is no summary yet, so the transcript is the whole document.",
  "share.includeNone": "This meeting has no transcript to include.",
  "share.format": "Format",
  "share.formatHint": "Used for both sending and saving.",
  "share.formatPlaceholder": "Choose a format…",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "Send to an app…",
  "share.sendToAppTitle": "Hand the file to another app",
  "share.saveToDevice": "Save to this device…",
  "share.saveToDeviceTitle": "Save the file on this device",
  "share.gateHint": "Choose a format above to send or save.",
  "share.nothingToShare":
    "This meeting has no summary or transcript to put in a file yet.",
  "share.copyGroup": "Copy to clipboard",
  "share.copySummary": "Copy summary",
  "share.copySummaryTitle": "Copy the AI summary as Markdown",
  "share.copyTranscript": "Copy transcript",
  "share.copyTranscriptTitle": "Copy the raw transcript text",

  /* ---------------- Toasts ---------------- */
  "toast.exportedMarkdown": "Exported Markdown file.",
  "toast.exportedWord": "Exported Word document.",
  "toast.exportedPdf": "Exported PDF.",
  "toast.copiedSummary": "Copied summary to clipboard.",
  "toast.copiedTranscript": "Copied transcript to clipboard.",
  "toast.meetingDeleted": "Meeting deleted.",
  "toast.transcription": "Transcription: {message}",
  "toast.audio": "Audio: {message}",
  "toast.serverNotSetUp":
    "The Minutes summary server isn't set up yet. Set DESKSEC_TOKEN in .env or contact IT.",
  "toast.downloadModelFirst":
    "Download the “{model}” transcription model in Settings before recording.",
  "toast.configureOnline":
    "Configure online transcription in Settings (server token and DEEPGRAM_API_KEY on the server).",
  "toast.summarizeFailed": "Couldn't summarize that meeting: {message}",

  /* ---------------- Delete confirmation (native dialog) ---------------- */
  "confirm.deleteTitle": "Delete meeting",
  "confirm.deleteBody":
    "Delete {name} and its transcript and summary? This cannot be undone.",
  "confirm.deleteThis": "this meeting",

  /* ---------------- Loading settings ---------------- */
  "settingsLoading.label": "Loading settings",
  "settingsLoading.message": "Loading settings…",

  /* ---------------- Meeting prompt window ---------------- */
  "prompt.dismiss": "Dismiss",
  "prompt.callDetected": "{app} detected",
  "prompt.newMeeting": "New meeting",
  "prompt.callHeading": "Take notes for this call?",
  "prompt.callSub":
    "Minutes will capture the conversation and write your notes.",
  "prompt.manualHeading": "Start a meeting",
  "prompt.manualSub": "Name it now, or leave it and rename later.",
  "prompt.takeNotes": "Take notes",
  "prompt.startRecording": "Start recording",
  "prompt.notNow": "Not now",
  "prompt.meetingTitle": "Meeting title",
  "prompt.callPlaceholder": "{app} notes",
  "prompt.manualPlaceholder": "Untitled meeting",
  "prompt.hintStart": "start",
  "prompt.hintClose": "close",
  "prompt.errorHeading": "Meeting prompt",
  "prompt.errorBody": "Something went wrong loading this prompt.",
  "prompt.loadFailed": "Could not load meeting prompt. Close and try again.",
  "prompt.listening": "Listening",
  "prompt.call": "Call",

  /* ---------------- Settings: chrome ---------------- */
  "settings.title": "Settings",
  "settings.sectionsLabel": "Settings sections",
  "settings.applyImmediately": "Changes apply as you make them",
  "settings.saving": "Saving…",
  "settings.saved": "Saved",
  "settings.server": "Minutes summarization server",
  "settings.checking": "Checking…",
  "settings.unknown": "Unknown",
  "settings.serverUnreachableConfigured":
    "AI summaries need a working connection. Transcription still runs fully on-device. Contact IT if this persists.",
  "settings.serverUnlinked":
    "Summaries aren't linked to the server yet. Transcription still works offline. Contact IT for setup.",

  /* ---------------- Settings: tabs ---------------- */
  "settings.tab.appearance": "Appearance",
  "settings.blurb.appearance": "Light, dark, or follow your system.",
  "settings.tab.reading": "Reading comfort",
  "settings.blurb.reading":
    "Text size and spacing for transcripts, saved on this device.",
  "settings.tab.audio": "Audio",
  "settings.blurb.audio": "Choose what Minutes captures while recording.",
  "settings.tab.callDetection": "Call detection",
  "settings.blurb.callDetection":
    "Offer to take notes when a call app uses your microphone.",
  "settings.tab.transcription": "Transcription",
  "settings.blurb.transcription":
    "Engine, accuracy model, speakers, and spoken language.",
  "settings.tab.summary": "Summary",
  "settings.blurb.summary": "When AI summaries are written, and how.",
  "settings.tab.privacy": "Privacy",
  "settings.blurb.privacy": "What leaves this device.",
  "settings.tab.advanced": "Advanced",
  "settings.blurb.advanced":
    "For IT and developer setup. Most people can leave these unchanged.",

  /* ---------------- Settings: appearance ---------------- */
  "settings.language": "Language",
  "settings.languageHint":
    "The language of the app's own labels and messages, saved on this device. Messages that come from the server are not translated.",

  /* ---------------- Settings: reading comfort ---------------- */
  "settings.textSize": "Transcript text size",
  "settings.textSizeHint": "Applies to the transcript view.",
  "settings.sizeDefault": "Default",
  "settings.sizeLarge": "Large",
  "settings.sizeXLarge": "Extra large",
  "settings.lineSpacing": "Line spacing",
  "settings.spacingDefault": "Default",
  "settings.spacingRelaxed": "Relaxed",
  "settings.spacingLoose": "Loose",
  "settings.highContrast": "High-contrast text",
  "settings.reduceMotion": "Reduce motion",
  "settings.reduceMotionHint": "Less animation across the app.",
  "settings.readingOnThisDevice": "These preferences are saved on this device.",

  /* ---------------- Settings: audio ---------------- */
  "settings.captureMic": "Capture my microphone",
  "settings.microphone": "Microphone",
  "settings.microphoneHint":
    "Recording follows the device: if a Bluetooth headset drops out mid-meeting, capture continues on whichever microphone takes over.",
  "settings.systemDefault": "System default",
  "settings.captureSystemAudio": "Also capture system audio",
  "settings.captureSystemAudioHint":
    "Records what you hear in Zoom, Meet, Teams, and other apps — no meeting bot required. While this is on, everything playing on this device is recorded.",
  "settings.systemAudioSource": "System audio source",
  "settings.defaultOutput": "Default output",
  "settings.noCaptureSource":
    "Enable the microphone, system audio, or both — a recording needs something to capture.",
  "settings.loopbackLinux":
    "No system-audio monitor found. With PipeWire or PulseAudio, look for a source named “Monitor of …” in your sound settings, then reopen Settings.",
  "settings.loopbackWindows":
    "No system audio source found. Connect speakers or headphones, then reopen Settings. Stereo Mix or VB-Audio Cable also work if listed.",
  "settings.loopbackMacos":
    "No loopback device found. macOS needs a virtual audio driver (e.g. BlackHole). Install one, then reopen Settings.",
  "settings.loopbackUnknown":
    "No system-audio loopback device detected. A monitor/loopback source is needed to capture meeting audio without a bot.",

  /* ---------------- Settings: call detection ---------------- */
  "settings.callPrompt": "Prompt when a call app uses the microphone",
  "settings.callPromptHint":
    "Shows a floating Take notes card when Zoom, Teams (app or browser), Google Meet, Slack, FaceTime, WhatsApp, or Webex uses the mic while Minutes is open. Browser Meet/Teams need Automation access for Chrome/Safari in System Settings.",
  "settings.callCooldown": "Cooldown after dismiss",
  "settings.callCooldownHint": "Minutes to wait before prompting again.",
  "settings.callUnsupported":
    "Call detection is available on macOS. You can still start meetings manually with New Meeting.",

  /* ---------------- Settings: transcription ---------------- */
  "settings.engine": "Engine",
  "settings.engineWhisperHint":
    "Speech-to-text runs locally with a Whisper model. Your audio never leaves this device for transcription.",
  "settings.engineCloudHint":
    "Audio streams live to your Minutes server (Deepgram Live) for low-latency captions. Uses the same server URL and access token as AI summaries.",
  "settings.engineCloud": "Online (Minutes server · Deepgram)",
  "settings.engineWhisper": "On-device (Whisper)",
  "settings.statusLabel": "Status",
  "settings.onlineReady": "Online transcription is ready ({model}).",
  "settings.onlineNotConfigured":
    "Configure DESKSEC_TOKEN and ensure the server has DEEPGRAM_API_KEY.",
  "settings.accuracyModel": "Accuracy model",
  "settings.modelFiles": "Model files",
  "settings.modelDownloading": "Downloading {label}…",
  "settings.modelReady": "Model “{model}” is downloaded and ready.",
  "settings.modelMissing":
    "Model “{model}” is not downloaded yet — required before recording.",
  "settings.redownload": "Re-download",
  "settings.downloadModel": "Download model",
  "settings.downloadProgress": "Download progress",
  "settings.downloadOnce":
    "The {model} model is ~{size}. This runs once — keep this window open until it finishes.",
  "settings.downloadedModels":
    "Downloaded models ({size} on disk). Tap here to delete",
  "settings.downloadedModelsHint":
    "Remove models you no longer need. Use Download model above to fetch them again.",
  "settings.inUse": " · in use",
  "settings.deleteQuestion": "Delete?",
  "settings.deleting": "Deleting…",
  "settings.stopBeforeDeletingModels": "Stop recording before deleting models.",
  "settings.identifySpeakers": "Identify speakers",
  "settings.identifySpeakersWhisper":
    "Labels who spoke each segment. Downloads a small speaker model on first use.",
  "settings.identifySpeakersCloud":
    "Labels who spoke each segment using cloud diarization on the server.",
  "settings.spokenLanguage": "Spoken language",
  "settings.spokenLanguageHint":
    "The language spoken in your meetings. Auto-detect works for most recordings.",
  "settings.autoDetect": "Auto-detect",

  /* ---------------- Settings: summary ---------------- */
  "settings.autoSummarize": "Summarize meetings automatically",
  "settings.autoSummarizeHint":
    "When a meeting ends, write its summary without being asked. Meetings shorter than a minute are skipped. Turning this off means a transcript is only ever sent to the summarization server when you press Generate summary yourself.",
  "settings.summaryLanguage": "Summary language",
  "settings.summaryLanguageHint":
    "“Match the transcript” keeps the summary in the meeting's own language.",
  "settings.matchTranscript": "Match the transcript",
  "settings.summaryInstructions": "Summary instructions (optional)",
  "settings.summaryInstructionsHint":
    "Applied to every summary you generate. Leave blank for the default behavior. You can also add per-meeting instructions before generating a summary.",

  /* ---------------- Settings: privacy ---------------- */
  "settings.telemetry": "Share anonymous usage statistics",
  "settings.telemetryHint":
    "Helps us see which features are used, how fast they are, and which errors happen.",
  "settings.telemetryDetail":
    "What is sent: feature usage counts, duration ranges, error categories, app version, operating system and version, CPU type and core count, and a random install ID you can reset. What is never sent: your recordings, transcripts, summaries, meeting titles, participant names, file paths, or anything you type or say. If the app is offline, reports wait in a small file on this device and are sent later. Reports are kept for 12 months. Turning this off stops all reporting immediately, deletes anything still waiting on this device, and deletes the install ID.",

  /* ---------------- Settings: advanced ---------------- */
  "settings.startAtLogin": "Start at login",
  "settings.startAtLoginHint":
    "Runs Minutes in the background from login so meetings are detected before you open the app.",
  "settings.startAtLoginHintNoDetection":
    "Runs Minutes in the background from login so it is ready immediately. Automatic meeting detection is not available on this platform.",
  "settings.serverUrl": "Server URL",
  "settings.serverUrlLocked":
    "Locked — configured at build time from CI ({url}).",
  "settings.serverUrlEmbedded": "embedded",
  "settings.serverUrlHint":
    "Remote servers must use https:// — http:// only works for localhost.",
  "settings.accessToken": "Access token",
  "settings.tokenFromBuild":
    "Configured at build time from CI and stored in the OS keychain.",
  "settings.tokenFromEnv": "Set from DESKSEC_TOKEN in .env.",
  "settings.tokenInKeychain": "Stored in the OS keychain.",
  "settings.tokenMissing":
    "Set DESKSEC_TOKEN in .env (see .env.example) for AI summaries.",
  "settings.deviceId": "Device ID",
  "settings.deviceIdHint":
    "Identifies this install to the server. Quote it to IT when asking for this device's access to be revoked.",
  "settings.summaryModel": "Summary model",
  "settings.chunkLength": "Chunk length",
  "settings.chunkLengthHint":
    "Seconds. Finalized transcript segments are produced every chunk.",
  "settings.partialInterval": "Partial interval",
  "settings.partialIntervalHint":
    "Seconds, 0 = off. Interim text refreshes at this interval. Both run on-device.",
  "settings.exportMarkdown": "Export finished meetings to ~/meetings",
  "settings.exportMarkdownHint":
    "Mirrors each completed meeting as markdown so the bundled Minutes CLI, MCP tools, and relationship graph can read it.",
  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "Transcript appears as people speak.",
  "settings.connectionCheckFailed": "Could not check connection",
  "serverUrl.enterFull": "Enter a full URL, e.g. https://minutes.example.com or http://localhost:8787.",
  "serverUrl.onlyHttp": "Only http:// and https:// URLs are supported.",
  "serverUrl.httpsRequired": "Remote servers must use https:// — plain http:// would send your token and transcript in cleartext. (http:// is only allowed for localhost.)",
  "model.tiny": "Tiny — fastest, lowest accuracy",
  "model.base": "Base — fast",
  "model.small": "Small — recommended",
  "model.medium": "Medium — more accurate, slower",
  "model.largeV3": "Large v3 — best accuracy, slowest",
  "model.unknownSize": "unknown size",
  "error.unknown": "An unknown error occurred.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Word document",
  "dialog.pdf": "PDF",
  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Language & Region",
  "settings.blurb.languageRegion": "The language the app speaks, and how dates are shown.",
  "settings.dateFormat": "Dates and times",
  "settings.dateFormatHint": "Shown in this device's regional format, taken from the operating system. Change it in your system settings.",
  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Connected",
  "settings.notConfigured": "Not set up",
  "settings.unreachable": "Unreachable",
  "model.vad": "Voice activity detection",
  "model.diarization": "Speaker identification",
  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "That meeting is being recorded — stop it before deleting.",
  "error.noTranscriptCheckLanguage":
    "Audio is reaching the server but nothing is being transcribed. Check the transcription language in Settings — it must match the language being spoken.",
  "error.meetingNotFound": "That meeting could not be found.",
  "error.nothingToShare": "There is nothing to share for this meeting yet.",
  "error.shareUnsupported": "Sharing to another app isn't available on this platform — save the file instead.",
  "error.noWindowToShare": "The main window is not available to share from.",
  "error.stopBeforeEngineChange": "Stop recording before changing the transcription engine.",
  "error.noCaptureSource": "Enable the microphone, system audio, or both.",
  "error.exportPathNotAbsolute": "That save location could not be used.",
  "error.exportExtension": "That file type is not one Minutes can write.",
  "error.stopBeforeDeletingModels": "Stop recording before deleting models.",
  "error.serverTokenMissing": "The Minutes server access token isn't set up. Check Settings, or contact IT.",
  "error.deviceRevoked":
    "This device's access has been revoked. Contact your IT team to have it restored.",
  "error.serverRejectedToken": "The Minutes server rejected the access token. Check Settings, or contact IT.",
  "error.onlineNotConfiguredOnServer": "Online transcription isn't configured on the Minutes server. Contact IT.",
  "error.unknownBrowser": "That browser is not one Minutes can detect meetings in.",
  "error.noPrivacyPane": "This system has no settings page for that permission.",

  /* ---------------- First-run setup ----------------
     Shown once after installation. Every step is optional: a wizard that blocks
     the app on a decline would be worse than the mid-meeting prompts it
     replaces. Copy states what each permission is for and what it is not, since
     "allow this app to control Chrome" reads alarmingly without that. */
  "onboarding.stepOf": "Step {current} of {total}",
  "onboarding.skipAll": "Skip setup",
  "onboarding.back": "Back",
  "onboarding.continue": "Continue",
  "onboarding.skipStep": "Not now",
  "onboarding.openSettings": "Open System Settings",
  "onboarding.allowed": "Allowed",
  "onboarding.notAllowed": "Not allowed",
  "onboarding.notSetUp": "Not set up",
  "onboarding.checking": "Checking…",

  "onboarding.welcomeTitle": "Welcome to Minutes",
  "onboarding.welcomeBody":
    "Minutes records your meetings and writes up the notes. Before you start, it needs permission for a couple of things.",
  "onboarding.welcomeOptional": "Every step is optional, and you can change any of it later in Settings.",
  "onboarding.getStarted": "Get started",

  "onboarding.microphoneTitle": "Microphone",
  "onboarding.microphoneBody":
    "Minutes records meeting audio from your microphone. Nothing is recorded until you start a meeting.",
  "onboarding.microphoneAllow": "Allow microphone",
  "onboarding.microphoneDeniedHint":
    "Microphone access was turned down, and macOS only asks once. You can switch it on under Privacy & Security → Microphone.",
  "onboarding.microphoneWindowsHint":
    "Windows doesn't ask apps for this directly. If recording picks up nothing, check that microphone access is on for desktop apps in Privacy & security → Microphone.",

  "onboarding.browserTitle": "Meetings opened in a browser",
  "onboarding.browserBody":
    "To offer to take notes when you join a Google Meet or Teams call from a link, Minutes checks whether a meeting is open in your browser.",
  "onboarding.browserPrivacy":
    "It looks only at whether a tab is a meeting — not at page contents, and nothing leaves your device.",
  "onboarding.browserPerApp":
    "macOS grants this one browser at a time, so each is listed separately. Only browsers you have installed are shown.",
  "onboarding.browserAllow": "Allow",
  "onboarding.browserNone":
    "No supported browser was found, so there's nothing to set up here. Meetings in Zoom, Teams and Slack are detected without this.",
  "onboarding.browserDeniedHint":
    "macOS only asks once per browser. To change it, go to Privacy & Security → Automation and tick Minutes under that browser.",


  "onboarding.detectionUnavailableTitle": "Starting a meeting",
  "onboarding.detectionUnavailableBody":
    "Automatic meeting detection is only available on macOS at the moment. On this system, start recording yourself with New Meeting whenever you want notes taken.",

  "onboarding.doneTitle": "You're ready",
  "onboarding.doneBody": "Here's where everything stands. You can revisit any of this in Settings.",
  "onboarding.doneSkipped": "Skipped — you can set this up later in Settings.",
  "onboarding.finish": "Start using Minutes",

  "settings.rerunOnboarding": "Permissions and setup",
  "settings.rerunOnboardingHint":
    "Walk through the microphone and browser detection setup again.",
  "settings.rerunOnboardingAction": "Run setup",
} as const;
