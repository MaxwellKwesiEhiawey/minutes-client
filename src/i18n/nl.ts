import type { Translations } from "./index";

/**
 * Dutch. Addressed with "je", which is what Dutch software UIs use even in a
 * work context, and infinitive-style labels ("Verwijderen") for actions.
 *
 * The privacy strings say exactly what the English says: `share.includeOff` must
 * make clear nothing spoken verbatim is in the file, and
 * `settings.telemetryDetail` must not soften what is and is not transmitted.
 */
export const nl: Translations = {
  "common.close": "Sluiten",
  "common.cancel": "Annuleren",
  "common.delete": "Verwijderen",
  "common.done": "Klaar",
  "common.open": "Openen",
  "common.tryAgain": "Opnieuw proberen",
  "common.retrying": "Nieuwe poging…",
  "common.loading": "Laden…",
  "common.yes": "Ja",
  "common.none": "—",
  "common.starting": "Starten…",

  "nav.home": "Start",
  "nav.myNotes": "Mijn notities",
  "nav.settings": "Instellingen",
  "nav.newMeeting": "Nieuwe vergadering",
  "nav.brandHome": "Startpagina van Minutes",
  "nav.main": "Hoofdnavigatie",

  "topbar.toggleSidebar": "Zijbalk in- of uitklappen",
  "topbar.search": "Zoeken in vergaderingen en transcripties…",
  "topbar.searchLabel": "Zoeken in vergaderingen en transcripties",
  "topbar.themeTitle": "Thema: {theme} — klik om te wijzigen",
  "topbar.recordingOpen": "Naar de vergadering die wordt opgenomen",
  "topbar.recordingStop": "De vergadering die wordt opgenomen beëindigen",
  "topbar.stop": "Stoppen",
  "theme.light": "Licht",
  "theme.dark": "Donker",
  "theme.system": "Systeem",

  "page.home": "Start",
  "page.notes": "Mijn notities",
  "page.settings": "Instellingen",
  "page.recording": "Opname",
  "page.meeting": "Vergadering",

  "home.greeting": "Hallo, welkom!",
  "home.sub": "Klaar om je volgende gesprek in iets nuttigs te veranderen?",
  "home.recent": "Recente vergaderingen",
  "home.viewAll": "Alle notities bekijken →",
  "home.summaryReady": "Samenvatting klaar",
  "home.transcriptOnly": "Alleen transcriptie",
  "home.emptyTitle": "Hier verschijnen je vergaderingen",
  "home.emptyBody":
    "Start je eerste vergadering en Minutes legt het gesprek vast, maakt een samenvatting en regelt de rest voor je.",
  "home.emptyCta": "Een vergadering starten",

  "notes.title": "Mijn notities",
  "notes.sub": "Alle vergaderingen die Minutes op dit apparaat heeft vastgelegd.",
  "notes.results": "Resultaten voor ‘{query}’",
  "notes.clearSearchText": "Zoektekst wissen",
  "notes.colMeeting": "Vergadering",
  "notes.colDate": "Datum",
  "notes.colDuration": "Duur",
  "notes.colSummary": "Samenvatting",
  "notes.colStatus": "Status",
  "notes.moreActions": "Meer acties voor {title}",
  "notes.stopBeforeDelete": "Stop de opname voordat je verwijdert",
  "notes.emptySearchTitle": "Geen vergaderingen passen bij je zoekopdracht",
  "notes.emptySearchBody":
    "Probeer een ander woord of een andere zin — de zoekopdracht omvat titels en transcriptietekst.",
  "notes.clearSearch": "Zoekopdracht wissen",
  "notes.emptyTitle": "Nog geen vergaderingen",
  "notes.emptyBody":
    "Start een vergadering en die verschijnt hier met transcriptie en samenvatting.",

  "status.recording": "Opnemen",
  "status.completed": "Afgerond",
  "status.interrupted": "Onderbroken",

  "detail.back": "Terug naar Mijn notities",
  "detail.share": "Delen en exporteren",
  "detail.delete": "Vergadering verwijderen",
  "detail.tabSummary": "Samenvatting",
  "detail.tabTranscript": "Transcriptie",
  "detail.tabsLabel": "Vergaderpanelen",
  "detail.generate": "Samenvatting maken",
  "detail.regenerate": "Samenvatting opnieuw maken",
  "detail.summarizing": "Samenvatten…",
  "detail.generateTitle": "Een AI-samenvatting maken",
  "detail.generateDisabled":
    "Voor deze vergadering is nog geen transcriptie vastgelegd",
  "detail.instructionsToggle": "Instructies toevoegen",
  "detail.instructionsLabel": "Instructies voor deze samenvatting (optioneel)",
  "detail.instructionsPlaceholder":
    "bijv. Neem de namen van de genoemde personen niet op.",
  "detail.instructionsCombined":
    "Worden gecombineerd met je standaardinstructies uit Instellingen.",
  "detail.instructionsApplied":
    "Worden toegepast wanneer je de samenvatting maakt of opnieuw maakt.",
  "detail.writingSummary":
    "Je samenvatting wordt geschreven — dat duurt meestal ongeveer een minuut.",
  "detail.noSummaryTitle": "Nog geen samenvatting",
  "detail.noSummaryReady":
    "Maak er een uit de transcriptie wanneer je wil.",
  "detail.noSummaryNoTranscript":
    "Een samenvatting heeft een transcriptie nodig — voor deze vergadering is niets vastgelegd.",
  "detail.noTranscriptTitle": "Geen transcriptie vastgelegd",
  "detail.noTranscriptBody":
    "Deze vergadering heeft geen transcriptiefragmenten.",
  "detail.speaker": "Spreker",

  "summaryError.networkTitle":
    "De server voor samenvattingen was niet bereikbaar.",
  "summaryError.networkHint":
    "Controleer je netwerkverbinding en de server-URL in Instellingen en probeer het opnieuw.",
  "summaryError.timeoutTitle":
    "De server voor samenvattingen deed er te lang over om te antwoorden.",
  "summaryError.timeoutHint":
    "Dat kan gebeuren bij een langzame verbinding of een heel lange transcriptie. Probeer het opnieuw.",
  "summaryError.authTitle":
    "De server voor samenvattingen weigerde het verzoek (niet gemachtigd).",
  "summaryError.authHint":
    "Je Minutes-toegangstoken ontbreekt mogelijk of is ongeldig — bekijk Instellingen of neem contact op met IT.",
  "summaryError.serverTitle":
    "De server voor samenvattingen gaf een fout terug.",
  "summaryError.genericTitle": "Kon geen samenvatting maken.",

  "summary.aiNote":
    "Door AI gemaakt op basis van de transcriptie · controleer voor het delen",
  "summary.overview": "Overzicht",
  "summary.keyPoints": "Belangrijkste gesprekspunten",
  "summary.decisions": "Besluiten",
  "summary.actionItems": "Actiepunten",
  "summary.openQuestions": "Openstaande vragen",
  "summary.openQuestion": "Openstaande vraag",
  "summary.owner": "verantwoordelijk: {name}",
  "summary.assignedTo": "Toegewezen aan: {name}",
  "summary.due": "Deadline: {date}",
  "summary.generatedBy": "Gemaakt door {model} · {date}",

  "recording.back": "Terug naar Mijn notities",
  "recording.transcriptSaved": "De transcriptie wordt live opgeslagen",
  "recording.endMeeting": "Vergadering beëindigen",
  "recording.inputLevel": "Ingangsniveau",
  "recording.liveTranscript": "Live transcriptie",
  "recording.savedAsCaptured": "Wordt opgeslagen terwijl het wordt vastgelegd",
  "recording.nothingYet": "Nog niets vastgelegd",
  "recording.listening":
    "Aan het luisteren — de transcriptie verschijnt terwijl er gesproken wordt.",
  "recording.interim": "Voorlopige transcriptie",
  "recording.live": "Live",

  "engine.onDevice": "Privé · op dit apparaat",
  "engine.onDeviceTitle":
    "De transcriptie loopt op dit apparaat (Whisper-model: {model})",
  "engine.cloud": "Transcriptie in de cloud",
  "engine.cloudTitle": "De transcriptie loopt online (Deepgram)",

  "palette.label": "Zoeken",
  "palette.placeholder": "Zoeken in vergaderingen en transcripties…",
  "palette.recent": "Recent",
  "palette.meetings": "Vergaderingen",
  "palette.transcripts": "Transcripties",
  "palette.noResults": "Geen resultaten voor ‘{query}’",
  "palette.noResultsHint":
    "Probeer de naam van een spreker of een zin uit het gesprek.",
  "palette.nothingYet": "Nog niets om te zoeken",
  "palette.nothingYetHint":
    "Neem een vergadering op en die wordt hier doorzoekbaar.",

  "share.title": "Delen en exporteren",
  "share.includeTranscript": "De volledige transcriptie meesturen",
  "share.includeOn":
    "Het bestand bevat de samenvatting en alles wat er gezegd is.",
  "share.includeOff":
    "Het bestand bevat alleen de samenvatting — niets van wat er letterlijk gezegd is.",
  "share.includeForced":
    "Er is nog geen samenvatting, dus de transcriptie is het hele document.",
  "share.includeNone":
    "Deze vergadering heeft geen transcriptie om mee te sturen.",
  "share.format": "Formaat",
  "share.formatHint": "Wordt gebruikt voor zowel versturen als opslaan.",
  "share.formatPlaceholder": "Kies een formaat…",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "Naar een app versturen…",
  "share.sendToAppTitle": "Het bestand aan een andere app overdragen",
  "share.saveToDevice": "Op dit apparaat opslaan…",
  "share.saveToDeviceTitle": "Het bestand op dit apparaat opslaan",
  "share.gateHint": "Kies hierboven een formaat om te versturen of op te slaan.",
  "share.nothingToShare":
    "Deze vergadering heeft nog geen samenvatting of transcriptie voor een bestand.",
  "share.copyGroup": "Naar het klembord kopiëren",
  "share.copySummary": "Samenvatting kopiëren",
  "share.copySummaryTitle": "De AI-samenvatting als Markdown kopiëren",
  "share.copyTranscript": "Transcriptie kopiëren",
  "share.copyTranscriptTitle": "De ruwe transcriptietekst kopiëren",

  "toast.exportedMarkdown": "Markdown-bestand geëxporteerd.",
  "toast.exportedWord": "Word-document geëxporteerd.",
  "toast.exportedPdf": "PDF geëxporteerd.",
  "toast.copiedSummary": "Samenvatting naar het klembord gekopieerd.",
  "toast.copiedTranscript": "Transcriptie naar het klembord gekopieerd.",
  "toast.meetingDeleted": "Vergadering verwijderd.",
  "toast.transcription": "Transcriptie: {message}",
  "toast.audio": "Audio: {message}",
  "toast.serverNotSetUp":
    "De Minutes-server voor samenvattingen is nog niet ingesteld. Zet DESKSEC_TOKEN in .env of neem contact op met IT.",
  "toast.downloadModelFirst":
    "Download het transcriptiemodel ‘{model}’ in Instellingen voordat je opneemt.",
  "toast.configureOnline":
    "Stel online transcriptie in bij Instellingen (servertoken en DEEPGRAM_API_KEY op de server).",
  "toast.summarizeFailed":
    "Kon die vergadering niet samenvatten: {message}",

  "confirm.deleteTitle": "Vergadering verwijderen",
  "confirm.deleteBody":
    "{name} met transcriptie en samenvatting verwijderen? Dit kan niet ongedaan worden gemaakt.",
  "confirm.deleteThis": "deze vergadering",

  "settingsLoading.label": "Instellingen laden",
  "settingsLoading.message": "Instellingen laden…",

  "prompt.dismiss": "Negeren",
  "prompt.callDetected": "{app} gedetecteerd",
  "prompt.newMeeting": "Nieuwe vergadering",
  "prompt.callHeading": "Notities maken van dit gesprek?",
  "prompt.callSub":
    "Minutes legt het gesprek vast en schrijft je notities.",
  "prompt.manualHeading": "Een vergadering starten",
  "prompt.manualSub":
    "Geef het nu een naam, of laat het en wijzig de naam later.",
  "prompt.takeNotes": "Notities maken",
  "prompt.startRecording": "Opname starten",
  "prompt.notNow": "Niet nu",
  "prompt.meetingTitle": "Titel van de vergadering",
  "prompt.callPlaceholder": "{app}-notities",
  "prompt.manualPlaceholder": "Vergadering zonder titel",
  "prompt.hintStart": "starten",
  "prompt.hintClose": "sluiten",
  "prompt.errorHeading": "Vergadermelding",
  "prompt.errorBody": "Er ging iets mis bij het laden van deze melding.",
  "prompt.loadFailed":
    "Kon de vergadermelding niet laden. Sluit af en probeer het opnieuw.",
  "prompt.listening": "Aan het luisteren",
  "prompt.call": "Gesprek",

  "settings.title": "Instellingen",
  "settings.sectionsLabel": "Onderdelen van de instellingen",
  "settings.applyImmediately": "Wijzigingen worden direct toegepast",
  "settings.saving": "Opslaan…",
  "settings.saved": "Opgeslagen",
  "settings.server": "Minutes-server voor samenvattingen",
  "settings.checking": "Controleren…",
  "settings.unknown": "Onbekend",
  "settings.serverUnreachableConfigured":
    "AI-samenvattingen hebben een werkende verbinding nodig. De transcriptie loopt nog volledig op dit apparaat. Neem contact op met IT als dit blijft gebeuren.",
  "settings.serverUnlinked":
    "Samenvattingen zijn nog niet met de server verbonden. De transcriptie werkt nog offline. Neem contact op met IT om dit in te stellen.",

  "settings.tab.appearance": "Weergave",
  "settings.blurb.appearance": "Licht, donker of het systeem volgen.",
  "settings.tab.reading": "Leescomfort",
  "settings.blurb.reading":
    "Tekstgrootte en regelafstand voor transcripties, opgeslagen op dit apparaat.",
  "settings.tab.audio": "Audio",
  "settings.blurb.audio": "Kies wat Minutes vastlegt tijdens het opnemen.",
  "settings.tab.callDetection": "Gespreksdetectie",
  "settings.blurb.callDetection":
    "Notities aanbieden wanneer een gespreksapp je microfoon gebruikt.",
  "settings.tab.transcription": "Transcriptie",
  "settings.blurb.transcription":
    "Engine, nauwkeurigheidsmodel, sprekers en gesproken taal.",
  "settings.tab.summary": "Samenvatting",
  "settings.blurb.summary":
    "Wanneer AI-samenvattingen worden geschreven, en hoe.",
  "settings.tab.privacy": "Privacy",
  "settings.blurb.privacy": "Wat dit apparaat verlaat.",
  "settings.tab.advanced": "Geavanceerd",
  "settings.blurb.advanced":
    "Voor IT en ontwikkeling. De meeste mensen kunnen dit ongewijzigd laten.",

  "settings.language": "Taal",
  "settings.languageHint":
    "De taal van de labels en meldingen van de app, opgeslagen op dit apparaat. Meldingen die van de server komen worden niet vertaald.",

  "settings.textSize": "Tekstgrootte van de transcriptie",
  "settings.textSizeHint": "Geldt voor de transcriptieweergave.",
  "settings.sizeDefault": "Standaard",
  "settings.sizeLarge": "Groot",
  "settings.sizeXLarge": "Extra groot",
  "settings.lineSpacing": "Regelafstand",
  "settings.spacingDefault": "Standaard",
  "settings.spacingRelaxed": "Ruim",
  "settings.spacingLoose": "Heel ruim",
  "settings.highContrast": "Tekst met hoog contrast",
  "settings.reduceMotion": "Beweging beperken",
  "settings.reduceMotionHint": "Minder animatie in de hele app.",
  "settings.readingOnThisDevice":
    "Deze voorkeuren worden op dit apparaat opgeslagen.",

  "settings.captureMic": "Mijn microfoon vastleggen",
  "settings.microphone": "Microfoon",
  "settings.microphoneHint":
    "De opname volgt het apparaat: valt een bluetooth-headset midden in de vergadering weg, dan gaat het vastleggen verder via de microfoon die het overneemt.",
  "settings.systemDefault": "Systeemstandaard",
  "settings.captureSystemAudio": "Ook systeemaudio vastleggen",
  "settings.captureSystemAudioHint":
    "Neemt op wat je hoort in Zoom, Meet, Teams en andere apps — zonder vergaderbot. Zolang dit aanstaat wordt alles opgenomen wat op dit apparaat te horen is.",
  "settings.systemAudioSource": "Bron voor systeemaudio",
  "settings.defaultOutput": "Standaarduitvoer",
  "settings.noCaptureSource":
    "Zet de microfoon aan, systeemaudio, of beide — een opname heeft iets nodig om vast te leggen.",
  "settings.loopbackLinux":
    "Geen monitor voor systeemaudio gevonden. Zoek met PipeWire of PulseAudio in je geluidsinstellingen naar een bron met de naam ‘Monitor of …’ en open Instellingen opnieuw.",
  "settings.loopbackWindows":
    "Geen bron voor systeemaudio gevonden. Sluit luidsprekers of een koptelefoon aan en open Instellingen opnieuw. Stereo Mix of VB-Audio Cable werken ook, als ze in de lijst staan.",
  "settings.loopbackMacos":
    "Geen loopback-apparaat gevonden. macOS heeft een virtueel audiostuurprogramma nodig (bijv. BlackHole). Installeer er een en open Instellingen opnieuw.",
  "settings.loopbackUnknown":
    "Geen loopback-apparaat voor systeemaudio gevonden. Om vergaderaudio zonder bot vast te leggen is een monitor- of loopbackbron nodig.",

  "settings.callPrompt":
    "Melden wanneer een gespreksapp de microfoon gebruikt",
  "settings.callPromptHint":
    "Toont een zwevende kaart ‘Notities maken’ wanneer Zoom, Teams (app of browser), Google Meet, Slack, FaceTime, WhatsApp of Webex de microfoon gebruikt terwijl Minutes open is. Meet/Teams in de browser hebben Automatiseringstoegang voor Chrome/Safari nodig in Systeeminstellingen.",
  "settings.callCooldown": "Wachttijd na negeren",
  "settings.callCooldownHint":
    "Minuten wachten voordat er opnieuw wordt gemeld.",
  "settings.callUnsupported":
    "Gespreksdetectie is beschikbaar op macOS. Je kunt vergaderingen nog altijd handmatig starten met ‘Nieuwe vergadering’.",

  "settings.engine": "Engine",
  "settings.engineWhisperHint":
    "Spraakherkenning loopt lokaal met een Whisper-model. Je audio verlaat dit apparaat nooit voor transcriptie.",
  "settings.engineCloudHint":
    "Audio wordt live naar je Minutes-server gestuurd (Deepgram Live) voor ondertitels met lage vertraging. Gebruikt dezelfde server-URL en hetzelfde toegangstoken als AI-samenvattingen.",
  "settings.engineCloud": "Online (Minutes-server · Deepgram)",
  "settings.engineWhisper": "Op het apparaat (Whisper)",
  "settings.statusLabel": "Status",
  "settings.onlineReady": "Online transcriptie is klaar ({model}).",
  "settings.onlineNotConfigured":
    "Stel DESKSEC_TOKEN in en zorg dat de server DEEPGRAM_API_KEY heeft.",
  "settings.accuracyModel": "Nauwkeurigheidsmodel",
  "settings.modelFiles": "Modelbestanden",
  "settings.modelDownloading": "{label} wordt gedownload…",
  "settings.modelReady": "Model ‘{model}’ is gedownload en klaar.",
  "settings.modelMissing":
    "Model ‘{model}’ is nog niet gedownload — vereist voordat je opneemt.",
  "settings.redownload": "Opnieuw downloaden",
  "settings.downloadModel": "Model downloaden",
  "settings.downloadProgress": "Voortgang van de download",
  "settings.downloadOnce":
    "Het model {model} is ongeveer {size}. Dit gebeurt eenmalig — laat dit venster open tot het klaar is.",
  "settings.downloadedModels":
    "Gedownloade modellen ({size} op schijf). Tik hier om te verwijderen",
  "settings.downloadedModelsHint":
    "Verwijder modellen die je niet meer nodig hebt. Gebruik ‘Model downloaden’ hierboven om ze opnieuw op te halen.",
  "settings.inUse": " · in gebruik",
  "settings.deleteQuestion": "Verwijderen?",
  "settings.deleting": "Verwijderen…",
  "settings.stopBeforeDeletingModels":
    "Stop de opname voordat je modellen verwijdert.",
  "settings.identifySpeakers": "Sprekers identificeren",
  "settings.identifySpeakersWhisper":
    "Geeft aan wie elk fragment sprak. Downloadt bij het eerste gebruik een klein sprekermodel.",
  "settings.identifySpeakersCloud":
    "Geeft aan wie elk fragment sprak via diarisatie in de cloud, op de server.",
  "settings.spokenLanguage": "Gesproken taal",
  "settings.spokenLanguageHint":
    "De taal die in je vergaderingen wordt gesproken. Automatisch detecteren werkt voor de meeste opnamen.",
  "settings.autoDetect": "Automatisch detecteren",

  "settings.autoSummarize": "Vergaderingen automatisch samenvatten",
  "settings.autoSummarizeHint":
    "Schrijft na het einde van een vergadering de samenvatting zonder dat erom gevraagd wordt. Vergaderingen korter dan een minuut worden overgeslagen. Zet je dit uit, dan wordt een transcriptie alleen naar de server voor samenvattingen gestuurd wanneer je zelf op ‘Samenvatting maken’ drukt.",
  "settings.summaryLanguage": "Taal van de samenvatting",
  "settings.summaryLanguageHint":
    "‘Zoals de transcriptie’ houdt de taal van de vergadering aan.",
  "settings.matchTranscript": "Zoals de transcriptie",
  "settings.summaryInstructions": "Instructies voor de samenvatting (optioneel)",
  "settings.summaryInstructionsHint":
    "Gelden voor elke samenvatting die je maakt. Laat leeg voor het standaardgedrag. Je kunt ook per vergadering instructies toevoegen voordat je een samenvatting maakt.",

  "settings.telemetry": "Anonieme gebruiksstatistieken delen",
  "settings.telemetryHint":
    "Helpt ons te zien welke functies worden gebruikt, hoe snel ze zijn en welke fouten optreden.",
  "settings.telemetryDetail":
    "Wat wordt verstuurd: hoe vaak functies worden gebruikt, tijdsduurcategorieën, foutcategorieën, de versie van de app, het besturingssysteem en de versie ervan, het processortype en het aantal cores, en een willekeurige installatie-ID die je kunt vernieuwen. Wat nooit wordt verstuurd: je opnamen, transcripties, samenvattingen, vergadertitels, namen van deelnemers, bestandspaden, of iets van wat je typt of zegt. Is de app offline, dan wachten de rapporten in een klein bestand op dit apparaat en worden ze later verstuurd. Rapporten worden 12 maanden bewaard. Zet je dit uit, dan stopt alle verzending onmiddellijk, wordt alles wat nog op dit apparaat wacht verwijderd en wordt de installatie-ID verwijderd.",

  "settings.startAtLogin": "Starten bij aanmelden",
  "settings.startAtLoginHint":
    "Voert Minutes vanaf het aanmelden op de achtergrond uit, zodat vergaderingen worden gedetecteerd voordat u de app opent.",
  "settings.startAtLoginHintNoDetection":
    "Voert Minutes vanaf het aanmelden op de achtergrond uit, zodat het meteen klaar is. Automatische vergaderdetectie werkt alleen op macOS.",
  "settings.serverUrl": "Server-URL",
  "settings.serverUrlLocked":
    "Vergrendeld — bij het bouwen ingesteld door CI ({url}).",
  "settings.serverUrlEmbedded": "ingebouwd",
  "settings.serverUrlHint":
    "Servers op afstand moeten https:// gebruiken — http:// werkt alleen voor localhost.",
  "settings.accessToken": "Toegangstoken",
  "settings.tokenFromBuild":
    "Bij het bouwen ingesteld door CI en opgeslagen in de sleutelhanger van het systeem.",
  "settings.tokenFromEnv": "Ingesteld via DESKSEC_TOKEN in .env.",
  "settings.tokenInKeychain":
    "Opgeslagen in de sleutelhanger van het systeem.",
  "settings.tokenMissing":
    "Zet DESKSEC_TOKEN in .env (zie .env.example) voor AI-samenvattingen.",
  "settings.deviceId": "Apparaat-ID",
  "settings.deviceIdHint":
    "Identificeert deze installatie bij de server. Geef dit door aan IT om de toegang van dit apparaat te laten intrekken.",
  "settings.summaryModel": "Model voor samenvattingen",
  "settings.chunkLength": "Bloklengte",
  "settings.chunkLengthHint":
    "Seconden. Per blok worden definitieve transcriptiefragmenten gemaakt.",
  "settings.partialInterval": "Interval voor voorlopige tekst",
  "settings.partialIntervalHint":
    "Seconden, 0 = uit. De voorlopige tekst wordt met dit interval verversd. Beide lopen op het apparaat.",
  "settings.exportMarkdown": "Afgeronde vergaderingen exporteren naar ~/meetings",
  "settings.exportMarkdownHint":
    "Spiegelt elke afgeronde vergadering als markdown, zodat de meegeleverde Minutes-CLI, de MCP-tools en de relatiegraaf die kunnen lezen.",

  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "De transcriptie verschijnt terwijl er gesproken wordt.",
  "settings.connectionCheckFailed": "Kon de verbinding niet controleren",
  "serverUrl.enterFull": "Voer een volledige URL in, bijv. https://minutes.example.com of http://localhost:8787.",
  "serverUrl.onlyHttp": "Alleen http:// en https://-URL's worden ondersteund.",
  "serverUrl.httpsRequired": "Servers op afstand moeten https:// gebruiken — met gewoon http:// zouden je token en transcriptie onversleuteld worden verstuurd. (http:// mag alleen voor localhost.)",
  "model.tiny": "Tiny — snelst, laagste nauwkeurigheid",
  "model.base": "Base — snel",
  "model.small": "Small — aanbevolen",
  "model.medium": "Medium — nauwkeuriger, langzamer",
  "model.largeV3": "Large v3 — hoogste nauwkeurigheid, langzaamst",
  "model.unknownSize": "onbekende grootte",
  "error.unknown": "Er is een onbekende fout opgetreden.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Word-document",
  "dialog.pdf": "PDF",

  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Taal en regio",
  "settings.blurb.languageRegion": "De taal van de app, en hoe datums worden weergegeven.",
  "settings.dateFormat": "Datums en tijden",
  "settings.dateFormatHint": "Worden weergegeven in de regio-instelling van dit apparaat, overgenomen van het besturingssysteem. Wijzig die in je systeeminstellingen.",

  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Verbonden",
  "settings.notConfigured": "Niet ingesteld",
  "settings.unreachable": "Niet bereikbaar",

  "model.vad": "Spraakactiviteitsdetectie",
  "model.diarization": "Sprekeridentificatie",

  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "Die vergadering wordt opgenomen — stop die eerst voordat je verwijdert.",
  "error.noTranscriptCheckLanguage":
    "Audio bereikt de server, maar er wordt niets getranscribeerd. Controleer de transcriptietaal in Instellingen — die moet overeenkomen met de gesproken taal.",
  "error.meetingNotFound": "Die vergadering is niet gevonden.",
  "error.nothingToShare": "Er is voor deze vergadering nog niets om te delen.",
  "error.shareUnsupported": "Delen naar een andere app is op dit platform niet beschikbaar — sla het bestand in plaats daarvan op.",
  "error.noWindowToShare": "Het hoofdvenster is niet beschikbaar om vanuit te delen.",
  "error.stopBeforeEngineChange": "Stop de opname voordat je van transcriptie-engine wisselt.",
  "error.noCaptureSource": "Zet de microfoon aan, systeemaudio, of beide.",
  "error.exportPathNotAbsolute": "Die opslaglocatie kon niet worden gebruikt.",
  "error.exportExtension": "Minutes kan dat bestandstype niet schrijven.",
  "error.stopBeforeDeletingModels": "Stop de opname voordat je modellen verwijdert.",

  "error.serverTokenMissing": "Het toegangstoken voor de Minutes-server is niet ingesteld. Bekijk Instellingen of neem contact op met IT.",
  "error.serverRejectedToken": "De Minutes-server heeft het toegangstoken geweigerd. Bekijk Instellingen of neem contact op met IT.",
  "error.onlineNotConfiguredOnServer": "Online transcriptie is niet ingesteld op de Minutes-server. Neem contact op met IT.",
  "error.unknownBrowser": "In die browser kan Minutes geen vergaderingen herkennen.",
  "error.noPrivacyPane": "Dit systeem heeft geen instellingenpagina voor die toestemming.",

  /* ---------------- Eerste installatie ---------------- */
  "onboarding.stepOf": "Stap {current} van {total}",
  "onboarding.skipAll": "Installatie overslaan",
  "onboarding.back": "Terug",
  "onboarding.continue": "Doorgaan",
  "onboarding.skipStep": "Niet nu",
  "onboarding.openSettings": "Systeeminstellingen openen",
  "onboarding.allowed": "Toegestaan",
  "onboarding.notAllowed": "Niet toegestaan",
  "onboarding.notSetUp": "Niet ingesteld",
  "onboarding.checking": "Controleren…",

  "onboarding.welcomeTitle": "Welkom bij Minutes",
  "onboarding.welcomeBody":
    "Minutes neemt je vergaderingen op en schrijft de notulen. Voor je begint zijn er een paar toestemmingen nodig.",
  "onboarding.welcomeOptional": "Elke stap is optioneel en je kunt alles later in de instellingen wijzigen.",
  "onboarding.getStarted": "Aan de slag",

  "onboarding.microphoneTitle": "Microfoon",
  "onboarding.microphoneBody":
    "Minutes neemt het geluid van de vergadering op via je microfoon. Er wordt niets opgenomen voordat je een vergadering start.",
  "onboarding.microphoneAllow": "Microfoon toestaan",
  "onboarding.microphoneDeniedHint":
    "De microfoontoegang is geweigerd, en macOS vraagt het maar één keer. Je kunt het aanzetten bij «Privacy en beveiliging» → «Microfoon».",
  "onboarding.microphoneWindowsHint":
    "Windows vraagt apps hier niet rechtstreeks om. Als de opname niets oppikt, controleer dan of microfoontoegang aan staat voor bureaubladapps bij «Privacy en beveiliging» → «Microfoon».",

  "onboarding.browserTitle": "Vergaderingen in een browser",
  "onboarding.browserBody":
    "Om aan te bieden notulen te maken wanneer je via een link aan een Google Meet- of Teams-vergadering deelneemt, kijkt Minutes of er een vergadering in je browser open staat.",
  "onboarding.browserPrivacy":
    "Het kijkt alleen of een tabblad een vergadering is — niet naar de inhoud van pagina’s, en er verlaat niets je apparaat.",
  "onboarding.browserPerApp":
    "macOS geeft deze toestemming per browser, daarom staan ze los vermeld. Alleen browsers die je hebt geïnstalleerd worden getoond.",
  "onboarding.browserAllow": "Toestaan",
  "onboarding.browserNone":
    "Er is geen ondersteunde browser gevonden, dus hier is niets in te stellen. Vergaderingen in Zoom, Teams en Slack worden hier zonder herkend.",
  "onboarding.browserDeniedHint":
    "macOS vraagt het maar één keer per browser. Ga om dit te wijzigen naar «Privacy en beveiliging» → «Automatisering» en vink Minutes aan onder die browser.",


  "onboarding.detectionUnavailableTitle": "Een vergadering starten",
  "onboarding.detectionUnavailableBody":
    "Automatische vergaderdetectie is voorlopig alleen op macOS beschikbaar. Start op dit systeem zelf de opname met «Nieuwe vergadering» wanneer je notulen wilt.",

  "onboarding.doneTitle": "Je bent klaar",
  "onboarding.doneBody": "Dit is de huidige stand. Je kunt hier in de instellingen op terugkomen.",
  "onboarding.doneSkipped": "Overgeslagen — je kunt dit later in de instellingen instellen.",
  "onboarding.finish": "Minutes gaan gebruiken",

  "settings.rerunOnboarding": "Toestemmingen en installatie",
  "settings.rerunOnboardingHint":
    "Loop de instelling van microfoon en browserdetectie opnieuw door.",
  "settings.rerunOnboardingAction": "Installatie starten",
};
