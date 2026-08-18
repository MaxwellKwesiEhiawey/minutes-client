import type { Translations } from "./index";

/**
 * German. Addressed formally ("Sie"), which is what a work tool is expected to
 * use, and noun-style for labels ("Aufzeichnung" rather than "Aufzeichnen").
 *
 * The privacy strings are translated to say exactly what the English says — in
 * particular `share.includeOff` must make clear that nothing spoken verbatim is
 * in the file, and `settings.telemetryDetail` must not soften what is and is not
 * transmitted.
 */
export const de: Translations = {
  "common.close": "Schließen",
  "common.cancel": "Abbrechen",
  "common.delete": "Löschen",
  "common.done": "Fertig",
  "common.open": "Öffnen",
  "common.tryAgain": "Erneut versuchen",
  "common.retrying": "Neuer Versuch …",
  "common.loading": "Wird geladen …",
  "common.yes": "Ja",
  "common.none": "—",
  "common.starting": "Wird gestartet …",

  "nav.home": "Start",
  "nav.myNotes": "Meine Notizen",
  "nav.settings": "Einstellungen",
  "nav.newMeeting": "Neues Meeting",
  "nav.brandHome": "Minutes-Startseite",
  "nav.main": "Hauptnavigation",

  "topbar.toggleSidebar": "Seitenleiste ein-/ausblenden",
  "topbar.search": "Meetings und Transkripte durchsuchen …",
  "topbar.searchLabel": "Meetings und Transkripte durchsuchen",
  "topbar.themeTitle": "Design: {theme} — zum Ändern klicken",
  "topbar.recordingOpen": "Zum laufenden Meeting wechseln",
  "topbar.recordingStop": "Laufendes Meeting beenden",
  "topbar.stop": "Stopp",
  "theme.light": "Hell",
  "theme.dark": "Dunkel",
  "theme.system": "System",

  "page.home": "Start",
  "page.notes": "Meine Notizen",
  "page.settings": "Einstellungen",
  "page.recording": "Aufzeichnung",
  "page.meeting": "Meeting",

  "home.greeting": "Hallo, willkommen!",
  "home.sub":
    "Bereit, Ihr nächstes Gespräch in etwas Nützliches zu verwandeln?",
  "home.recent": "Letzte Meetings",
  "home.viewAll": "Alle Notizen ansehen →",
  "home.summaryReady": "Zusammenfassung fertig",
  "home.transcriptOnly": "Nur Transkript",
  "home.emptyTitle": "Hier erscheinen Ihre Meetings",
  "home.emptyBody":
    "Starten Sie Ihr erstes Meeting und Minutes erfasst das Gespräch, erstellt eine Zusammenfassung und ordnet alles für Sie.",
  "home.emptyCta": "Meeting starten",

  "notes.title": "Meine Notizen",
  "notes.sub": "Alle Meetings, die Minutes auf diesem Gerät erfasst hat.",
  "notes.results": "Ergebnisse für „{query}“",
  "notes.clearSearchText": "Suchtext löschen",
  "notes.colMeeting": "Meeting",
  "notes.colDate": "Datum",
  "notes.colDuration": "Dauer",
  "notes.colSummary": "Zusammenfassung",
  "notes.colStatus": "Status",
  "notes.moreActions": "Weitere Aktionen für {title}",
  "notes.stopBeforeDelete": "Aufzeichnung vor dem Löschen beenden",
  "notes.emptySearchTitle": "Keine Meetings passen zu Ihrer Suche",
  "notes.emptySearchBody":
    "Versuchen Sie ein anderes Wort oder eine andere Formulierung — die Suche umfasst Titel und Transkripttext.",
  "notes.clearSearch": "Suche zurücksetzen",
  "notes.emptyTitle": "Noch keine Meetings",
  "notes.emptyBody":
    "Starten Sie ein Meeting und es erscheint hier mit Transkript und Zusammenfassung.",

  "status.recording": "Zeichnet auf",
  "status.completed": "Abgeschlossen",
  "status.interrupted": "Unterbrochen",

  "detail.back": "Zurück zu Meine Notizen",
  "detail.share": "Teilen und exportieren",
  "detail.delete": "Meeting löschen",
  "detail.tabSummary": "Zusammenfassung",
  "detail.tabTranscript": "Transkript",
  "detail.tabsLabel": "Meeting-Bereiche",
  "detail.generate": "Zusammenfassung erstellen",
  "detail.regenerate": "Zusammenfassung neu erstellen",
  "detail.summarizing": "Wird zusammengefasst …",
  "detail.generateTitle": "KI-Zusammenfassung erstellen",
  "detail.generateDisabled":
    "Für dieses Meeting wurde noch kein Transkript erfasst",
  "detail.instructionsToggle": "Anweisungen hinzufügen",
  "detail.instructionsLabel":
    "Anweisungen für diese Zusammenfassung (optional)",
  "detail.instructionsPlaceholder":
    "z. B. Nennen Sie keine Namen der im Meeting erwähnten Personen.",
  "detail.instructionsCombined":
    "Wird mit Ihren Standardanweisungen aus den Einstellungen kombiniert.",
  "detail.instructionsApplied":
    "Wird angewendet, wenn Sie die Zusammenfassung erstellen oder neu erstellen.",
  "detail.writingSummary":
    "Ihre Zusammenfassung wird geschrieben — das dauert meist etwa eine Minute.",
  "detail.noSummaryTitle": "Noch keine Zusammenfassung",
  "detail.noSummaryReady":
    "Erstellen Sie eine aus dem Transkript, wenn Sie bereit sind.",
  "detail.noSummaryNoTranscript":
    "Eine Zusammenfassung braucht ein Transkript — für dieses Meeting wurde nichts erfasst.",
  "detail.noTranscriptTitle": "Kein Transkript erfasst",
  "detail.noTranscriptBody": "Dieses Meeting hat keine Transkriptabschnitte.",
  "detail.speaker": "Sprecher/in",

  "summaryError.networkTitle":
    "Der Server für Zusammenfassungen war nicht erreichbar.",
  "summaryError.networkHint":
    "Prüfen Sie Ihre Netzwerkverbindung und die Server-URL in den Einstellungen und versuchen Sie es erneut.",
  "summaryError.timeoutTitle":
    "Der Server für Zusammenfassungen hat zu lange nicht geantwortet.",
  "summaryError.timeoutHint":
    "Das kann bei langsamer Verbindung oder einem sehr langen Transkript passieren. Versuchen Sie es erneut.",
  "summaryError.authTitle":
    "Der Server für Zusammenfassungen hat die Anfrage abgelehnt (nicht autorisiert).",
  "summaryError.authHint":
    "Ihr Minutes-Zugriffstoken fehlt möglicherweise oder ist ungültig — prüfen Sie die Einstellungen oder wenden Sie sich an die IT.",
  "summaryError.serverTitle":
    "Der Server für Zusammenfassungen hat einen Fehler zurückgegeben.",
  "summaryError.genericTitle":
    "Die Zusammenfassung konnte nicht erstellt werden.",

  "summary.aiNote":
    "KI-generiert aus dem Transkript · vor dem Teilen prüfen",
  "summary.overview": "Überblick",
  "summary.keyPoints": "Wichtigste Punkte",
  "summary.decisions": "Entscheidungen",
  "summary.actionItems": "Aufgaben",
  "summary.openQuestions": "Offene Fragen",
  "summary.openQuestion": "Offene Frage",
  "summary.owner": "verantwortlich: {name}",
  "summary.assignedTo": "Zugewiesen an: {name}",
  "summary.due": "Fällig: {date}",
  "summary.generatedBy": "Erstellt von {model} · {date}",

  "recording.back": "Zurück zu Meine Notizen",
  "recording.transcriptSaved": "Transkript wird live gespeichert",
  "recording.endMeeting": "Meeting beenden",
  "recording.inputLevel": "Eingangspegel",
  "recording.liveTranscript": "Live-Transkript",
  "recording.savedAsCaptured": "Wird beim Erfassen gespeichert",
  "recording.nothingYet": "Noch nichts erfasst",
  "recording.listening":
    "Hört zu — das Transkript erscheint, während gesprochen wird.",
  "recording.interim": "Vorläufiges Transkript",
  "recording.live": "Live",

  "engine.onDevice": "Privat · auf diesem Gerät",
  "engine.onDeviceTitle":
    "Die Transkription läuft auf diesem Gerät (Whisper-Modell: {model})",
  "engine.cloud": "Cloud-Transkription",
  "engine.cloudTitle": "Die Transkription läuft online (Deepgram)",

  "palette.label": "Suche",
  "palette.placeholder": "Meetings und Transkripte durchsuchen …",
  "palette.recent": "Zuletzt",
  "palette.meetings": "Meetings",
  "palette.transcripts": "Transkripte",
  "palette.noResults": "Keine Ergebnisse für „{query}“",
  "palette.noResultsHint":
    "Versuchen Sie einen Sprechernamen oder eine Formulierung aus dem Gespräch.",
  "palette.nothingYet": "Noch nichts zum Durchsuchen",
  "palette.nothingYetHint":
    "Zeichnen Sie ein Meeting auf und es wird hier durchsuchbar.",

  "share.title": "Teilen & Exportieren",
  "share.includeTranscript": "Vollständiges Transkript einschließen",
  "share.includeOn":
    "Die Datei enthält die Zusammenfassung und alles, was gesagt wurde.",
  "share.includeOff":
    "Die Datei enthält nur die Zusammenfassung — nichts im Wortlaut Gesagtes.",
  "share.includeForced":
    "Es gibt noch keine Zusammenfassung, daher ist das Transkript das gesamte Dokument.",
  "share.includeNone":
    "Dieses Meeting hat kein Transkript, das eingeschlossen werden könnte.",
  "share.format": "Format",
  "share.formatHint": "Gilt für Senden und Speichern.",
  "share.formatPlaceholder": "Format wählen …",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "An eine App senden …",
  "share.sendToAppTitle": "Die Datei an eine andere App übergeben",
  "share.saveToDevice": "Auf diesem Gerät speichern …",
  "share.saveToDeviceTitle": "Die Datei auf diesem Gerät speichern",
  "share.gateHint": "Wählen Sie oben ein Format zum Senden oder Speichern.",
  "share.nothingToShare":
    "Dieses Meeting hat noch keine Zusammenfassung und kein Transkript für eine Datei.",
  "share.copyGroup": "In die Zwischenablage kopieren",
  "share.copySummary": "Zusammenfassung kopieren",
  "share.copySummaryTitle": "Die KI-Zusammenfassung als Markdown kopieren",
  "share.copyTranscript": "Transkript kopieren",
  "share.copyTranscriptTitle": "Den reinen Transkripttext kopieren",

  "toast.exportedMarkdown": "Markdown-Datei exportiert.",
  "toast.exportedWord": "Word-Dokument exportiert.",
  "toast.exportedPdf": "PDF exportiert.",
  "toast.copiedSummary": "Zusammenfassung in die Zwischenablage kopiert.",
  "toast.copiedTranscript": "Transkript in die Zwischenablage kopiert.",
  "toast.meetingDeleted": "Meeting gelöscht.",
  "toast.transcription": "Transkription: {message}",
  "toast.audio": "Audio: {message}",
  "toast.serverNotSetUp":
    "Der Minutes-Server für Zusammenfassungen ist noch nicht eingerichtet. Setzen Sie DESKSEC_TOKEN in der .env-Datei oder wenden Sie sich an die IT.",
  "toast.downloadModelFirst":
    "Laden Sie das Transkriptionsmodell „{model}“ in den Einstellungen herunter, bevor Sie aufzeichnen.",
  "toast.configureOnline":
    "Richten Sie die Online-Transkription in den Einstellungen ein (Server-Token und DEEPGRAM_API_KEY auf dem Server).",
  "toast.summarizeFailed":
    "Dieses Meeting konnte nicht zusammengefasst werden: {message}",

  "confirm.deleteTitle": "Meeting löschen",
  "confirm.deleteBody":
    "{name} samt Transkript und Zusammenfassung löschen? Das lässt sich nicht rückgängig machen.",
  "confirm.deleteThis": "dieses Meeting",

  "settingsLoading.label": "Einstellungen werden geladen",
  "settingsLoading.message": "Einstellungen werden geladen …",

  "prompt.dismiss": "Verwerfen",
  "prompt.callDetected": "{app} erkannt",
  "prompt.newMeeting": "Neues Meeting",
  "prompt.callHeading": "Für dieses Gespräch Notizen machen?",
  "prompt.callSub":
    "Minutes erfasst das Gespräch und schreibt Ihre Notizen.",
  "prompt.manualHeading": "Meeting starten",
  "prompt.manualSub":
    "Benennen Sie es jetzt oder später — umbenennen geht jederzeit.",
  "prompt.takeNotes": "Notizen machen",
  "prompt.startRecording": "Aufzeichnung starten",
  "prompt.notNow": "Nicht jetzt",
  "prompt.meetingTitle": "Meeting-Titel",
  "prompt.callPlaceholder": "{app}-Notizen",
  "prompt.manualPlaceholder": "Meeting ohne Titel",
  "prompt.hintStart": "starten",
  "prompt.hintClose": "schließen",
  "prompt.errorHeading": "Meeting-Hinweis",
  "prompt.errorBody": "Beim Laden dieses Hinweises ist etwas schiefgegangen.",
  "prompt.loadFailed":
    "Der Meeting-Hinweis konnte nicht geladen werden. Schließen und erneut versuchen.",
  "prompt.listening": "Hört zu",
  "prompt.call": "Anruf",

  "settings.title": "Einstellungen",
  "settings.sectionsLabel": "Einstellungsbereiche",
  "settings.applyImmediately": "Änderungen werden sofort übernommen",
  "settings.saving": "Wird gespeichert …",
  "settings.saved": "Gespeichert",
  "settings.server": "Minutes-Server für Zusammenfassungen",
  "settings.checking": "Wird geprüft …",
  "settings.unknown": "Unbekannt",
  "settings.serverUnreachableConfigured":
    "KI-Zusammenfassungen brauchen eine funktionierende Verbindung. Die Transkription läuft weiterhin vollständig auf dem Gerät. Wenden Sie sich an die IT, wenn das anhält.",
  "settings.serverUnlinked":
    "Zusammenfassungen sind noch nicht mit dem Server verbunden. Die Transkription funktioniert weiterhin offline. Wenden Sie sich zur Einrichtung an die IT.",

  "settings.tab.appearance": "Erscheinungsbild",
  "settings.blurb.appearance": "Hell, dunkel oder wie das System.",
  "settings.tab.reading": "Lesekomfort",
  "settings.blurb.reading":
    "Textgröße und Zeilenabstand für Transkripte, auf diesem Gerät gespeichert.",
  "settings.tab.audio": "Audio",
  "settings.blurb.audio":
    "Legen Sie fest, was Minutes bei der Aufzeichnung erfasst.",
  "settings.tab.callDetection": "Anruferkennung",
  "settings.blurb.callDetection":
    "Notizen anbieten, wenn eine Anruf-App Ihr Mikrofon verwendet.",
  "settings.tab.transcription": "Transkription",
  "settings.blurb.transcription":
    "Engine, Genauigkeitsmodell, Sprecher und gesprochene Sprache.",
  "settings.tab.summary": "Zusammenfassung",
  "settings.blurb.summary":
    "Wann KI-Zusammenfassungen geschrieben werden und wie.",
  "settings.tab.privacy": "Datenschutz",
  "settings.blurb.privacy": "Was dieses Gerät verlässt.",
  "settings.tab.advanced": "Erweitert",
  "settings.blurb.advanced":
    "Für IT und Entwicklung. Die meisten können diese Werte unverändert lassen.",

  "settings.language": "Sprache",
  "settings.languageHint":
    "Die Sprache der Bezeichnungen und Meldungen der App, auf diesem Gerät gespeichert. Meldungen vom Server werden nicht übersetzt.",

  "settings.textSize": "Textgröße im Transkript",
  "settings.textSizeHint": "Gilt für die Transkriptansicht.",
  "settings.sizeDefault": "Standard",
  "settings.sizeLarge": "Groß",
  "settings.sizeXLarge": "Sehr groß",
  "settings.lineSpacing": "Zeilenabstand",
  "settings.spacingDefault": "Standard",
  "settings.spacingRelaxed": "Locker",
  "settings.spacingLoose": "Weit",
  "settings.highContrast": "Text mit hohem Kontrast",
  "settings.reduceMotion": "Bewegung reduzieren",
  "settings.reduceMotionHint": "Weniger Animation in der ganzen App.",
  "settings.readingOnThisDevice":
    "Diese Einstellungen werden auf diesem Gerät gespeichert.",

  "settings.captureMic": "Mein Mikrofon erfassen",
  "settings.microphone": "Mikrofon",
  "settings.microphoneHint":
    "Die Aufzeichnung folgt dem Gerät: Fällt mitten im Meeting ein Bluetooth-Headset aus, läuft die Erfassung über das Mikrofon weiter, das übernimmt.",
  "settings.systemDefault": "Systemstandard",
  "settings.captureSystemAudio": "Auch Systemaudio erfassen",
  "settings.captureSystemAudioHint":
    "Zeichnet auf, was Sie in Zoom, Meet, Teams und anderen Apps hören — ohne Meeting-Bot. Solange dies aktiv ist, wird alles aufgezeichnet, was auf diesem Gerät abgespielt wird.",
  "settings.systemAudioSource": "Quelle für Systemaudio",
  "settings.defaultOutput": "Standardausgabe",
  "settings.noCaptureSource":
    "Aktivieren Sie Mikrofon, Systemaudio oder beides — eine Aufzeichnung braucht eine Quelle.",
  "settings.loopbackLinux":
    "Kein Monitor für Systemaudio gefunden. Suchen Sie mit PipeWire oder PulseAudio in Ihren Audioeinstellungen nach einer Quelle namens „Monitor of …“ und öffnen Sie die Einstellungen erneut.",
  "settings.loopbackWindows":
    "Keine Systemaudioquelle gefunden. Schließen Sie Lautsprecher oder Kopfhörer an und öffnen Sie die Einstellungen erneut. Stereo Mix oder VB-Audio Cable funktionieren ebenfalls, wenn sie aufgeführt sind.",
  "settings.loopbackMacos":
    "Kein Loopback-Gerät gefunden. macOS benötigt einen virtuellen Audiotreiber (z. B. BlackHole). Installieren Sie einen und öffnen Sie die Einstellungen erneut.",
  "settings.loopbackUnknown":
    "Kein Loopback-Gerät für Systemaudio erkannt. Für die Aufzeichnung von Meeting-Audio ohne Bot ist eine Monitor-/Loopback-Quelle nötig.",

  "settings.callPrompt": "Hinweis, wenn eine Anruf-App das Mikrofon verwendet",
  "settings.callPromptHint":
    "Zeigt eine schwebende Karte „Notizen machen“, wenn Zoom, Teams (App oder Browser), Google Meet, Slack, FaceTime, WhatsApp oder Webex das Mikrofon verwendet, während Minutes geöffnet ist. Für Meet/Teams im Browser ist in den Systemeinstellungen Automatisierungszugriff für Chrome/Safari nötig.",
  "settings.callCooldown": "Wartezeit nach dem Verwerfen",
  "settings.callCooldownHint":
    "Minuten, die vor einem erneuten Hinweis gewartet wird.",
  "settings.callUnsupported":
    "Die Anruferkennung ist unter macOS verfügbar. Meetings können Sie weiterhin manuell über „Neues Meeting“ starten.",

  "settings.engine": "Engine",
  "settings.engineWhisperHint":
    "Die Spracherkennung läuft lokal mit einem Whisper-Modell. Ihr Audio verlässt dieses Gerät für die Transkription nie.",
  "settings.engineCloudHint":
    "Audio wird live an Ihren Minutes-Server gestreamt (Deepgram Live), für Untertitel mit geringer Latenz. Verwendet dieselbe Server-URL und dasselbe Zugriffstoken wie KI-Zusammenfassungen.",
  "settings.engineCloud": "Online (Minutes-Server · Deepgram)",
  "settings.engineWhisper": "Auf dem Gerät (Whisper)",
  "settings.statusLabel": "Status",
  "settings.onlineReady": "Die Online-Transkription ist bereit ({model}).",
  "settings.onlineNotConfigured":
    "Setzen Sie DESKSEC_TOKEN und stellen Sie sicher, dass der Server DEEPGRAM_API_KEY hat.",
  "settings.accuracyModel": "Genauigkeitsmodell",
  "settings.modelFiles": "Modelldateien",
  "settings.modelDownloading": "{label} wird heruntergeladen …",
  "settings.modelReady":
    "Modell „{model}“ ist heruntergeladen und einsatzbereit.",
  "settings.modelMissing":
    "Modell „{model}“ ist noch nicht heruntergeladen — vor dem Aufzeichnen erforderlich.",
  "settings.redownload": "Erneut herunterladen",
  "settings.downloadModel": "Modell herunterladen",
  "settings.downloadProgress": "Download-Fortschritt",
  "settings.downloadOnce":
    "Das Modell {model} ist etwa {size} groß. Das passiert nur einmal — lassen Sie dieses Fenster offen, bis es fertig ist.",
  "settings.downloadedModels":
    "Heruntergeladene Modelle ({size} auf der Festplatte). Zum Löschen hier tippen",
  "settings.downloadedModelsHint":
    "Entfernen Sie Modelle, die Sie nicht mehr brauchen. Mit „Modell herunterladen“ oben holen Sie sie wieder.",
  "settings.inUse": " · in Verwendung",
  "settings.deleteQuestion": "Löschen?",
  "settings.deleting": "Wird gelöscht …",
  "settings.stopBeforeDeletingModels":
    "Beenden Sie die Aufzeichnung, bevor Sie Modelle löschen.",
  "settings.identifySpeakers": "Sprecher erkennen",
  "settings.identifySpeakersWhisper":
    "Kennzeichnet, wer welchen Abschnitt gesprochen hat. Lädt bei der ersten Verwendung ein kleines Sprechermodell herunter.",
  "settings.identifySpeakersCloud":
    "Kennzeichnet mithilfe der Cloud-Diarisierung auf dem Server, wer welchen Abschnitt gesprochen hat.",
  "settings.spokenLanguage": "Gesprochene Sprache",
  "settings.spokenLanguageHint":
    "Die in Ihren Meetings gesprochene Sprache. Die automatische Erkennung funktioniert für die meisten Aufzeichnungen.",
  "settings.autoDetect": "Automatisch erkennen",

  "settings.autoSummarize": "Meetings automatisch zusammenfassen",
  "settings.autoSummarizeHint":
    "Schreibt die Zusammenfassung eines Meetings nach dessen Ende, ohne dass Sie darum bitten. Meetings unter einer Minute werden übersprungen. Ist dies aus, wird ein Transkript nur dann an den Server für Zusammenfassungen gesendet, wenn Sie selbst auf „Zusammenfassung erstellen“ drücken.",
  "settings.summaryLanguage": "Sprache der Zusammenfassung",
  "settings.summaryLanguageHint":
    "„Wie das Transkript“ behält die Sprache des Meetings bei.",
  "settings.matchTranscript": "Wie das Transkript",
  "settings.summaryInstructions": "Anweisungen zur Zusammenfassung (optional)",
  "settings.summaryInstructionsHint":
    "Gilt für jede Zusammenfassung, die Sie erstellen. Leer lassen für das Standardverhalten. Vor dem Erstellen können Sie zusätzlich Anweisungen pro Meeting angeben.",

  "settings.telemetry": "Anonyme Nutzungsstatistiken teilen",
  "settings.telemetryHint":
    "Hilft uns zu sehen, welche Funktionen genutzt werden, wie schnell sie sind und welche Fehler auftreten.",
  "settings.telemetryDetail":
    "Was gesendet wird: Zählwerte zur Funktionsnutzung, Dauerbereiche, Fehlerkategorien, App-Version, Betriebssystem und Version, CPU-Typ und Kernanzahl sowie eine zufällige Installations-ID, die Sie zurücksetzen können. Was nie gesendet wird: Ihre Aufzeichnungen, Transkripte, Zusammenfassungen, Meeting-Titel, Teilnehmernamen, Dateipfade oder irgendetwas, das Sie tippen oder sagen. Ist die App offline, warten die Berichte in einer kleinen Datei auf diesem Gerät und werden später gesendet. Berichte werden 12 Monate aufbewahrt. Wenn Sie dies ausschalten, endet jede Übermittlung sofort, alles noch Wartende auf diesem Gerät wird gelöscht und die Installations-ID wird entfernt.",

  "settings.startAtLogin": "Beim Anmelden starten",
  "settings.startAtLoginHint":
    "Führt Minutes ab der Anmeldung im Hintergrund aus, damit Besprechungen erkannt werden, bevor Sie die App öffnen.",
  "settings.startAtLoginHintNoDetection":
    "Führt Minutes ab der Anmeldung im Hintergrund aus, damit es sofort bereit ist. Die automatische Besprechungserkennung ist auf dieser Plattform nicht verfügbar.",
  "settings.serverUrl": "Server-URL",
  "settings.serverUrlLocked":
    "Gesperrt — beim Build durch CI konfiguriert ({url}).",
  "settings.serverUrlEmbedded": "eingebettet",
  "settings.serverUrlHint":
    "Externe Server müssen https:// verwenden — http:// funktioniert nur für localhost.",
  "settings.accessToken": "Zugriffstoken",
  "settings.tokenFromBuild":
    "Beim Build durch CI konfiguriert und im Schlüsselbund des Systems gespeichert.",
  "settings.tokenFromEnv": "Aus DESKSEC_TOKEN in der .env-Datei gesetzt.",
  "settings.tokenInKeychain": "Im Schlüsselbund des Systems gespeichert.",
  "settings.tokenMissing":
    "Setzen Sie DESKSEC_TOKEN in der .env-Datei (siehe .env.example) für KI-Zusammenfassungen.",
  "settings.deviceId": "Geräte-ID",
  "settings.deviceIdHint":
    "Identifiziert diese Installation gegenüber dem Server. Geben Sie sie der IT an, wenn der Zugriff dieses Geräts widerrufen werden soll.",
  "settings.summaryModel": "Modell für Zusammenfassungen",
  "settings.chunkLength": "Blocklänge",
  "settings.chunkLengthHint":
    "Sekunden. Pro Block werden endgültige Transkriptabschnitte erzeugt.",
  "settings.partialInterval": "Intervall für Vorschau",
  "settings.partialIntervalHint":
    "Sekunden, 0 = aus. In diesem Intervall wird der vorläufige Text aktualisiert. Beides läuft auf dem Gerät.",
  "settings.exportMarkdown": "Fertige Meetings nach ~/meetings exportieren",
  "settings.exportMarkdownHint":
    "Spiegelt jedes abgeschlossene Meeting als Markdown, damit die mitgelieferte Minutes-CLI, die MCP-Tools und der Beziehungsgraph es lesen können.",

  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "Das Transkript erscheint, während gesprochen wird.",
  "settings.connectionCheckFailed": "Verbindung konnte nicht geprüft werden",
  "serverUrl.enterFull": "Geben Sie eine vollständige URL ein, z. B. https://minutes.example.com oder http://localhost:8787.",
  "serverUrl.onlyHttp": "Es werden nur http:// und https:// URLs unterstützt.",
  "serverUrl.httpsRequired": "Externe Server müssen https:// verwenden — bei einfachem http:// würden Ihr Token und Ihr Transkript im Klartext übertragen. (http:// ist nur für localhost erlaubt.)",
  "model.tiny": "Tiny — am schnellsten, geringste Genauigkeit",
  "model.base": "Base — schnell",
  "model.small": "Small — empfohlen",
  "model.medium": "Medium — genauer, langsamer",
  "model.largeV3": "Large v3 — beste Genauigkeit, am langsamsten",
  "model.unknownSize": "unbekannte Größe",
  "error.unknown": "Ein unbekannter Fehler ist aufgetreten.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Word-Dokument",
  "dialog.pdf": "PDF",

  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Sprache & Region",
  "settings.blurb.languageRegion": "Die Sprache der App und die Darstellung von Datumsangaben.",
  "settings.dateFormat": "Datum und Uhrzeit",
  "settings.dateFormatHint": "Werden im regionalen Format dieses Geräts angezeigt, das vom Betriebssystem übernommen wird. Ändern Sie es in den Systemeinstellungen.",

  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Verbunden",
  "settings.notConfigured": "Nicht eingerichtet",
  "settings.unreachable": "Nicht erreichbar",

  "model.vad": "Spracherkennung (Sprachaktivität)",
  "model.diarization": "Sprechererkennung",

  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "Dieses Meeting wird aufgezeichnet — beenden Sie es, bevor Sie es löschen.",
  "error.noTranscriptCheckLanguage":
    "Audio erreicht den Server, es wird aber nichts transkribiert. Prüfen Sie die Transkriptionssprache in den Einstellungen – sie muss der gesprochenen Sprache entsprechen.",
  "error.meetingNotFound": "Dieses Meeting wurde nicht gefunden.",
  "error.nothingToShare": "Für dieses Meeting gibt es noch nichts zu teilen.",
  "error.shareUnsupported": "Das Teilen an eine andere App ist auf dieser Plattform nicht verfügbar — speichern Sie die Datei stattdessen.",
  "error.noWindowToShare": "Das Hauptfenster ist zum Teilen nicht verfügbar.",
  "error.stopBeforeEngineChange": "Beenden Sie die Aufzeichnung, bevor Sie die Transkriptions-Engine wechseln.",
  "error.noCaptureSource": "Aktivieren Sie Mikrofon, Systemaudio oder beides.",
  "error.exportPathNotAbsolute": "Dieser Speicherort konnte nicht verwendet werden.",
  "error.exportExtension": "Dieses Dateiformat kann Minutes nicht schreiben.",
  "error.stopBeforeDeletingModels": "Beenden Sie die Aufzeichnung, bevor Sie Modelle löschen.",

  "error.serverTokenMissing": "Das Zugriffstoken für den Minutes-Server ist nicht eingerichtet. Prüfen Sie die Einstellungen oder wenden Sie sich an die IT.",
  "error.serverRejectedToken": "Der Minutes-Server hat das Zugriffstoken abgelehnt. Prüfen Sie die Einstellungen oder wenden Sie sich an die IT.",
  "error.onlineNotConfiguredOnServer": "Die Online-Transkription ist auf dem Minutes-Server nicht eingerichtet. Wenden Sie sich an die IT.",
  "error.unknownBrowser": "In diesem Browser kann Minutes keine Besprechungen erkennen.",
  "error.noPrivacyPane": "Dieses System hat keine Einstellungsseite für diese Berechtigung.",

  /* ---------------- Erste Einrichtung ---------------- */
  "onboarding.stepOf": "Schritt {current} von {total}",
  "onboarding.skipAll": "Einrichtung überspringen",
  "onboarding.back": "Zurück",
  "onboarding.continue": "Weiter",
  "onboarding.skipStep": "Später",
  "onboarding.openSettings": "Systemeinstellungen öffnen",
  "onboarding.allowed": "Erlaubt",
  "onboarding.notAllowed": "Nicht erlaubt",
  "onboarding.notSetUp": "Nicht eingerichtet",
  "onboarding.checking": "Wird geprüft …",

  "onboarding.welcomeTitle": "Willkommen bei Minutes",
  "onboarding.welcomeBody":
    "Minutes zeichnet Ihre Besprechungen auf und verfasst das Protokoll. Vorher werden einige Berechtigungen benötigt.",
  "onboarding.welcomeOptional": "Jeder Schritt ist freiwillig und lässt sich später in den Einstellungen ändern.",
  "onboarding.getStarted": "Los geht’s",

  "onboarding.microphoneTitle": "Mikrofon",
  "onboarding.microphoneBody":
    "Minutes zeichnet den Besprechungston über Ihr Mikrofon auf. Es wird nichts aufgezeichnet, bevor Sie eine Besprechung starten.",
  "onboarding.microphoneAllow": "Mikrofon erlauben",
  "onboarding.microphoneDeniedHint":
    "Der Mikrofonzugriff wurde abgelehnt, und macOS fragt nur einmal. Sie können ihn unter „Datenschutz & Sicherheit“ → „Mikrofon“ aktivieren.",
  "onboarding.microphoneWindowsHint":
    "Windows fragt Apps nicht direkt danach. Wenn die Aufzeichnung stumm bleibt, prüfen Sie unter „Datenschutz und Sicherheit“ → „Mikrofon“, ob der Zugriff für Desktop-Apps aktiviert ist.",

  "onboarding.browserTitle": "Im Browser geöffnete Besprechungen",
  "onboarding.browserBody":
    "Damit Minutes anbieten kann, mitzuschreiben, wenn Sie einer Google-Meet- oder Teams-Besprechung über einen Link beitreten, prüft es, ob im Browser eine Besprechung geöffnet ist.",
  "onboarding.browserPrivacy":
    "Dabei wird nur erkannt, ob ein Tab eine Besprechung ist – nicht der Seiteninhalt, und nichts verlässt Ihr Gerät.",
  "onboarding.browserPerApp":
    "macOS erteilt diese Berechtigung je Browser einzeln, deshalb sind sie getrennt aufgeführt. Angezeigt werden nur installierte Browser.",
  "onboarding.browserAllow": "Erlauben",
  "onboarding.browserNone":
    "Es wurde kein unterstützter Browser gefunden, hier ist also nichts einzurichten. Besprechungen in Zoom, Teams und Slack werden ohne dies erkannt.",
  "onboarding.browserDeniedHint":
    "macOS fragt nur einmal pro Browser. Zum Ändern öffnen Sie „Datenschutz & Sicherheit“ → „Automation“ und aktivieren Minutes unter dem jeweiligen Browser.",


  "onboarding.detectionUnavailableTitle": "Eine Besprechung starten",
  "onboarding.detectionUnavailableBody":
    "Die automatische Besprechungserkennung ist derzeit nur unter macOS verfügbar. Starten Sie die Aufzeichnung auf diesem System selbst über „Neue Besprechung“, wenn mitgeschrieben werden soll.",

  "onboarding.doneTitle": "Sie sind bereit",
  "onboarding.doneBody": "Hier der aktuelle Stand. Sie können all das in den Einstellungen erneut aufrufen.",
  "onboarding.doneSkipped": "Übersprungen – Sie können dies später in den Einstellungen einrichten.",
  "onboarding.finish": "Minutes verwenden",

  "settings.rerunOnboarding": "Berechtigungen und Einrichtung",
  "settings.rerunOnboardingHint":
    "Die Einrichtung von Mikrofon und Browsererkennung erneut durchlaufen.",
  "settings.rerunOnboardingAction": "Einrichtung starten",
};
