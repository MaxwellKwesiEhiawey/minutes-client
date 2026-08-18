import type { Translations } from "./index";

/**
 * Italian. Impersonal phrasing where possible, which is the register Italian
 * software UIs use and avoids choosing between "tu" and "Lei".
 *
 * The privacy strings say exactly what the English says: `share.includeOff` must
 * make clear nothing spoken verbatim is in the file, and
 * `settings.telemetryDetail` must not soften what is and is not transmitted.
 */
export const it: Translations = {
  "common.close": "Chiudi",
  "common.cancel": "Annulla",
  "common.delete": "Elimina",
  "common.done": "Fatto",
  "common.open": "Apri",
  "common.tryAgain": "Riprova",
  "common.retrying": "Nuovo tentativo…",
  "common.loading": "Caricamento…",
  "common.yes": "Sì",
  "common.none": "—",
  "common.starting": "Avvio…",

  "nav.home": "Home",
  "nav.myNotes": "Le mie note",
  "nav.settings": "Impostazioni",
  "nav.newMeeting": "Nuova riunione",
  "nav.brandHome": "Home di Minutes",
  "nav.main": "Navigazione principale",

  "topbar.toggleSidebar": "Mostra o nascondi la barra laterale",
  "topbar.search": "Cerca nelle riunioni e nelle trascrizioni…",
  "topbar.searchLabel": "Cerca nelle riunioni e nelle trascrizioni",
  "topbar.themeTitle": "Tema: {theme} — clicca per cambiare",
  "topbar.recordingOpen": "Vai alla riunione in registrazione",
  "topbar.recordingStop": "Termina la riunione in registrazione",
  "topbar.stop": "Interrompi",
  "theme.light": "Chiaro",
  "theme.dark": "Scuro",
  "theme.system": "Sistema",

  "page.home": "Home",
  "page.notes": "Le mie note",
  "page.settings": "Impostazioni",
  "page.recording": "Registrazione",
  "page.meeting": "Riunione",

  "home.greeting": "Ciao, benvenuto!",
  "home.sub":
    "Pronto a trasformare la prossima conversazione in qualcosa di utile?",
  "home.recent": "Riunioni recenti",
  "home.viewAll": "Vedi tutte le note →",
  "home.summaryReady": "Riepilogo pronto",
  "home.transcriptOnly": "Solo trascrizione",
  "home.emptyTitle": "Qui compariranno le tue riunioni",
  "home.emptyBody":
    "Avvia la tua prima riunione e Minutes registrerà la conversazione, creerà un riepilogo e organizzerà tutto per te.",
  "home.emptyCta": "Avvia una riunione",

  "notes.title": "Le mie note",
  "notes.sub": "Tutte le riunioni che Minutes ha registrato su questo dispositivo.",
  "notes.results": "Risultati per «{query}»",
  "notes.clearSearchText": "Cancella il testo di ricerca",
  "notes.colMeeting": "Riunione",
  "notes.colDate": "Data",
  "notes.colDuration": "Durata",
  "notes.colSummary": "Riepilogo",
  "notes.colStatus": "Stato",
  "notes.moreActions": "Altre azioni per {title}",
  "notes.stopBeforeDelete": "Interrompi la registrazione prima di eliminare",
  "notes.emptySearchTitle": "Nessuna riunione corrisponde alla ricerca",
  "notes.emptySearchBody":
    "Prova un'altra parola o frase: la ricerca comprende i titoli e il testo delle trascrizioni.",
  "notes.clearSearch": "Cancella la ricerca",
  "notes.emptyTitle": "Nessuna riunione per ora",
  "notes.emptyBody":
    "Avvia una riunione e comparirà qui con la sua trascrizione e il suo riepilogo.",

  "status.recording": "In registrazione",
  "status.completed": "Completata",
  "status.interrupted": "Interrotta",

  "detail.back": "Torna a Le mie note",
  "detail.share": "Condividi ed esporta",
  "detail.delete": "Elimina la riunione",
  "detail.tabSummary": "Riepilogo",
  "detail.tabTranscript": "Trascrizione",
  "detail.tabsLabel": "Pannelli della riunione",
  "detail.generate": "Genera il riepilogo",
  "detail.regenerate": "Rigenera il riepilogo",
  "detail.summarizing": "Riepilogo in corso…",
  "detail.generateTitle": "Genera un riepilogo con l'IA",
  "detail.generateDisabled":
    "Per questa riunione non è ancora stata registrata alcuna trascrizione",
  "detail.instructionsToggle": "Aggiungi istruzioni",
  "detail.instructionsLabel": "Istruzioni per questo riepilogo (facoltativo)",
  "detail.instructionsPlaceholder":
    "es. Non includere i nomi delle persone citate nella riunione.",
  "detail.instructionsCombined":
    "Vengono combinate con le istruzioni di riepilogo predefinite nelle Impostazioni.",
  "detail.instructionsApplied":
    "Vengono applicate quando generi o rigeneri il riepilogo.",
  "detail.writingSummary":
    "Sto scrivendo il riepilogo: in genere serve circa un minuto.",
  "detail.noSummaryTitle": "Nessun riepilogo per ora",
  "detail.noSummaryReady":
    "Generane uno dalla trascrizione quando vuoi.",
  "detail.noSummaryNoTranscript":
    "Un riepilogo richiede una trascrizione: per questa riunione non è stato registrato nulla.",
  "detail.noTranscriptTitle": "Nessuna trascrizione registrata",
  "detail.noTranscriptBody":
    "Questa riunione non ha segmenti di trascrizione.",
  "detail.speaker": "Interlocutore",

  "summaryError.networkTitle":
    "Non è stato possibile raggiungere il server dei riepiloghi.",
  "summaryError.networkHint":
    "Controlla la connessione di rete e l'URL del server nelle Impostazioni, poi riprova.",
  "summaryError.timeoutTitle":
    "Il server dei riepiloghi ha risposto troppo lentamente.",
  "summaryError.timeoutHint":
    "Può succedere con una connessione lenta o una trascrizione molto lunga. Riprova.",
  "summaryError.authTitle":
    "Il server dei riepiloghi ha rifiutato la richiesta (non autorizzata).",
  "summaryError.authHint":
    "Il tuo token di accesso Minutes potrebbe essere assente o non valido: controlla le Impostazioni o contatta l'IT.",
  "summaryError.serverTitle": "Il server dei riepiloghi ha restituito un errore.",
  "summaryError.genericTitle": "Non è stato possibile generare un riepilogo.",

  "summary.aiNote":
    "Generato dall'IA dalla trascrizione · rivedilo prima di condividerlo",
  "summary.overview": "Panoramica",
  "summary.keyPoints": "Punti chiave della discussione",
  "summary.decisions": "Decisioni",
  "summary.actionItems": "Attività",
  "summary.openQuestions": "Domande aperte",
  "summary.openQuestion": "Domanda aperta",
  "summary.owner": "responsabile: {name}",
  "summary.assignedTo": "Assegnato a: {name}",
  "summary.due": "Scadenza: {date}",
  "summary.generatedBy": "Generato da {model} · {date}",

  "recording.back": "Torna a Le mie note",
  "recording.transcriptSaved": "La trascrizione viene salvata in diretta",
  "recording.endMeeting": "Termina la riunione",
  "recording.inputLevel": "Livello di ingresso",
  "recording.liveTranscript": "Trascrizione in diretta",
  "recording.savedAsCaptured": "Salvata mentre viene registrata",
  "recording.nothingYet": "Ancora nulla di registrato",
  "recording.listening":
    "In ascolto: la trascrizione compare mentre si parla.",
  "recording.interim": "Trascrizione provvisoria",
  "recording.live": "In diretta",

  "engine.onDevice": "Privato · su questo dispositivo",
  "engine.onDeviceTitle":
    "La trascrizione viene eseguita su questo dispositivo (modello Whisper: {model})",
  "engine.cloud": "Trascrizione nel cloud",
  "engine.cloudTitle": "La trascrizione viene eseguita online (Deepgram)",

  "palette.label": "Ricerca",
  "palette.placeholder": "Cerca nelle riunioni e nelle trascrizioni…",
  "palette.recent": "Recenti",
  "palette.meetings": "Riunioni",
  "palette.transcripts": "Trascrizioni",
  "palette.noResults": "Nessun risultato per «{query}»",
  "palette.noResultsHint":
    "Prova il nome di un interlocutore o una frase della conversazione.",
  "palette.nothingYet": "Ancora nulla da cercare",
  "palette.nothingYetHint":
    "Registra una riunione e diventerà ricercabile qui.",

  "share.title": "Condividi ed esporta",
  "share.includeTranscript": "Includi la trascrizione completa",
  "share.includeOn":
    "Il file conterrà il riepilogo e tutto ciò che è stato detto.",
  "share.includeOff":
    "Il file conterrà solo il riepilogo: nulla di ciò che è stato detto testualmente.",
  "share.includeForced":
    "Non c'è ancora un riepilogo, quindi la trascrizione è l'intero documento.",
  "share.includeNone":
    "Questa riunione non ha alcuna trascrizione da includere.",
  "share.format": "Formato",
  "share.formatHint": "Usato sia per l'invio sia per il salvataggio.",
  "share.formatPlaceholder": "Scegli un formato…",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "Invia a un'app…",
  "share.sendToAppTitle": "Consegna il file a un'altra applicazione",
  "share.saveToDevice": "Salva su questo dispositivo…",
  "share.saveToDeviceTitle": "Salva il file su questo dispositivo",
  "share.gateHint": "Scegli un formato qui sopra per inviare o salvare.",
  "share.nothingToShare":
    "Questa riunione non ha ancora un riepilogo o una trascrizione da mettere in un file.",
  "share.copyGroup": "Copia negli appunti",
  "share.copySummary": "Copia il riepilogo",
  "share.copySummaryTitle": "Copia il riepilogo IA come Markdown",
  "share.copyTranscript": "Copia la trascrizione",
  "share.copyTranscriptTitle": "Copia il testo grezzo della trascrizione",

  "toast.exportedMarkdown": "File Markdown esportato.",
  "toast.exportedWord": "Documento Word esportato.",
  "toast.exportedPdf": "PDF esportato.",
  "toast.copiedSummary": "Riepilogo copiato negli appunti.",
  "toast.copiedTranscript": "Trascrizione copiata negli appunti.",
  "toast.meetingDeleted": "Riunione eliminata.",
  "toast.transcription": "Trascrizione: {message}",
  "toast.audio": "Audio: {message}",
  "toast.serverNotSetUp":
    "Il server dei riepiloghi di Minutes non è ancora configurato. Imposta DESKSEC_TOKEN in .env oppure contatta l'IT.",
  "toast.downloadModelFirst":
    "Scarica il modello di trascrizione «{model}» nelle Impostazioni prima di registrare.",
  "toast.configureOnline":
    "Configura la trascrizione online nelle Impostazioni (token del server e DEEPGRAM_API_KEY sul server).",
  "toast.summarizeFailed":
    "Non è stato possibile riepilogare quella riunione: {message}",

  "confirm.deleteTitle": "Elimina la riunione",
  "confirm.deleteBody":
    "Eliminare {name} insieme alla sua trascrizione e al suo riepilogo? L'operazione non può essere annullata.",
  "confirm.deleteThis": "questa riunione",

  "settingsLoading.label": "Caricamento delle impostazioni",
  "settingsLoading.message": "Caricamento delle impostazioni…",

  "prompt.dismiss": "Ignora",
  "prompt.callDetected": "{app} rilevato",
  "prompt.newMeeting": "Nuova riunione",
  "prompt.callHeading": "Prendere appunti per questa chiamata?",
  "prompt.callSub":
    "Minutes registrerà la conversazione e scriverà i tuoi appunti.",
  "prompt.manualHeading": "Avvia una riunione",
  "prompt.manualSub":
    "Dai un nome adesso, o lascia stare e rinomina più tardi.",
  "prompt.takeNotes": "Prendi appunti",
  "prompt.startRecording": "Avvia la registrazione",
  "prompt.notNow": "Non ora",
  "prompt.meetingTitle": "Titolo della riunione",
  "prompt.callPlaceholder": "Appunti {app}",
  "prompt.manualPlaceholder": "Riunione senza titolo",
  "prompt.hintStart": "avvia",
  "prompt.hintClose": "chiudi",
  "prompt.errorHeading": "Avviso riunione",
  "prompt.errorBody":
    "Si è verificato un problema durante il caricamento di questo avviso.",
  "prompt.loadFailed":
    "Non è stato possibile caricare l'avviso della riunione. Chiudilo e riprova.",
  "prompt.listening": "In ascolto",
  "prompt.call": "Chiamata",

  "settings.title": "Impostazioni",
  "settings.sectionsLabel": "Sezioni delle impostazioni",
  "settings.applyImmediately": "Le modifiche vengono applicate subito",
  "settings.saving": "Salvataggio…",
  "settings.saved": "Salvato",
  "settings.server": "Server dei riepiloghi di Minutes",
  "settings.checking": "Verifica…",
  "settings.unknown": "Sconosciuto",
  "settings.serverUnreachableConfigured":
    "I riepiloghi con IA richiedono una connessione funzionante. La trascrizione continua a essere eseguita interamente sul dispositivo. Contatta l'IT se il problema persiste.",
  "settings.serverUnlinked":
    "I riepiloghi non sono ancora collegati al server. La trascrizione funziona comunque offline. Contatta l'IT per la configurazione.",

  "settings.tab.appearance": "Aspetto",
  "settings.blurb.appearance": "Chiaro, scuro o come il sistema.",
  "settings.tab.reading": "Comfort di lettura",
  "settings.blurb.reading":
    "Dimensione del testo e interlinea delle trascrizioni, salvati su questo dispositivo.",
  "settings.tab.audio": "Audio",
  "settings.blurb.audio":
    "Scegli cosa registra Minutes durante la registrazione.",
  "settings.tab.callDetection": "Rilevamento chiamate",
  "settings.blurb.callDetection":
    "Proponi di prendere appunti quando un'app di chiamate usa il microfono.",
  "settings.tab.transcription": "Trascrizione",
  "settings.blurb.transcription":
    "Motore, modello di precisione, interlocutori e lingua parlata.",
  "settings.tab.summary": "Riepilogo",
  "settings.blurb.summary":
    "Quando vengono scritti i riepiloghi con l'IA, e come.",
  "settings.tab.privacy": "Privacy",
  "settings.blurb.privacy": "Cosa esce da questo dispositivo.",
  "settings.tab.advanced": "Avanzate",
  "settings.blurb.advanced":
    "Per l'IT e lo sviluppo. Nella maggior parte dei casi si possono lasciare invariate.",

  "settings.language": "Lingua",
  "settings.languageHint":
    "La lingua delle etichette e dei messaggi dell'app, salvata su questo dispositivo. I messaggi che arrivano dal server non vengono tradotti.",

  "settings.textSize": "Dimensione del testo della trascrizione",
  "settings.textSizeHint": "Si applica alla vista trascrizione.",
  "settings.sizeDefault": "Predefinita",
  "settings.sizeLarge": "Grande",
  "settings.sizeXLarge": "Molto grande",
  "settings.lineSpacing": "Interlinea",
  "settings.spacingDefault": "Predefinita",
  "settings.spacingRelaxed": "Ampia",
  "settings.spacingLoose": "Molto ampia",
  "settings.highContrast": "Testo a contrasto elevato",
  "settings.reduceMotion": "Riduci le animazioni",
  "settings.reduceMotionHint": "Meno animazioni in tutta l'app.",
  "settings.readingOnThisDevice":
    "Queste preferenze sono salvate su questo dispositivo.",

  "settings.captureMic": "Registra il mio microfono",
  "settings.microphone": "Microfono",
  "settings.microphoneHint":
    "La registrazione segue il dispositivo: se delle cuffie Bluetooth si disconnettono a metà riunione, la registrazione continua con il microfono che subentra.",
  "settings.systemDefault": "Predefinito di sistema",
  "settings.captureSystemAudio": "Registra anche l'audio di sistema",
  "settings.captureSystemAudioHint":
    "Registra ciò che senti in Zoom, Meet, Teams e altre app, senza bisogno di un bot per le riunioni. Mentre è attivo, viene registrato tutto ciò che suona su questo dispositivo.",
  "settings.systemAudioSource": "Sorgente dell'audio di sistema",
  "settings.defaultOutput": "Uscita predefinita",
  "settings.noCaptureSource":
    "Attiva il microfono, l'audio di sistema o entrambi: una registrazione ha bisogno di qualcosa da registrare.",
  "settings.loopbackLinux":
    "Nessun monitor dell'audio di sistema trovato. Con PipeWire o PulseAudio, cerca una sorgente chiamata «Monitor of …» nelle impostazioni audio, poi riapri le Impostazioni.",
  "settings.loopbackWindows":
    "Nessuna sorgente di audio di sistema trovata. Collega altoparlanti o cuffie, poi riapri le Impostazioni. Anche Stereo Mix o VB-Audio Cable funzionano, se compaiono nell'elenco.",
  "settings.loopbackMacos":
    "Nessun dispositivo di loopback trovato. macOS richiede un driver audio virtuale (es. BlackHole). Installane uno, poi riapri le Impostazioni.",
  "settings.loopbackUnknown":
    "Nessun dispositivo di loopback rilevato per l'audio di sistema. Per registrare l'audio di una riunione senza un bot serve una sorgente monitor/loopback.",

  "settings.callPrompt":
    "Avvisa quando un'app di chiamate usa il microfono",
  "settings.callPromptHint":
    "Mostra una scheda mobile «Prendi appunti» quando Zoom, Teams (app o browser), Google Meet, Slack, FaceTime, WhatsApp o Webex usa il microfono mentre Minutes è aperto. Meet/Teams nel browser richiedono l'accesso Automazione per Chrome/Safari in Impostazioni di Sistema.",
  "settings.callCooldown": "Attesa dopo aver ignorato",
  "settings.callCooldownHint":
    "Minuti di attesa prima di avvisare di nuovo.",
  "settings.callUnsupported":
    "Il rilevamento delle chiamate è disponibile su macOS. Puoi comunque avviare le riunioni manualmente con «Nuova riunione».",

  "settings.engine": "Motore",
  "settings.engineWhisperHint":
    "Il riconoscimento vocale viene eseguito localmente con un modello Whisper. Il tuo audio non lascia mai questo dispositivo per la trascrizione.",
  "settings.engineCloudHint":
    "L'audio viene trasmesso in diretta al tuo server Minutes (Deepgram Live) per sottotitoli a bassa latenza. Usa lo stesso URL del server e lo stesso token di accesso dei riepiloghi con IA.",
  "settings.engineCloud": "Online (server Minutes · Deepgram)",
  "settings.engineWhisper": "Sul dispositivo (Whisper)",
  "settings.statusLabel": "Stato",
  "settings.onlineReady": "La trascrizione online è pronta ({model}).",
  "settings.onlineNotConfigured":
    "Configura DESKSEC_TOKEN e assicurati che il server abbia DEEPGRAM_API_KEY.",
  "settings.accuracyModel": "Modello di precisione",
  "settings.modelFiles": "File del modello",
  "settings.modelDownloading": "Download di {label}…",
  "settings.modelReady": "Il modello «{model}» è scaricato e pronto.",
  "settings.modelMissing":
    "Il modello «{model}» non è ancora stato scaricato: è necessario prima di registrare.",
  "settings.redownload": "Scarica di nuovo",
  "settings.downloadModel": "Scarica il modello",
  "settings.downloadProgress": "Avanzamento del download",
  "settings.downloadOnce":
    "Il modello {model} occupa circa {size}. Succede una volta sola: lascia aperta questa finestra fino al termine.",
  "settings.downloadedModels":
    "Modelli scaricati ({size} su disco). Tocca qui per eliminare",
  "settings.downloadedModelsHint":
    "Rimuovi i modelli che non ti servono più. Usa «Scarica il modello» qui sopra per riottenerli.",
  "settings.inUse": " · in uso",
  "settings.deleteQuestion": "Eliminare?",
  "settings.deleting": "Eliminazione…",
  "settings.stopBeforeDeletingModels":
    "Interrompi la registrazione prima di eliminare i modelli.",
  "settings.identifySpeakers": "Identifica gli interlocutori",
  "settings.identifySpeakersWhisper":
    "Indica chi ha parlato in ogni segmento. Al primo utilizzo scarica un piccolo modello per gli interlocutori.",
  "settings.identifySpeakersCloud":
    "Indica chi ha parlato in ogni segmento usando la diarizzazione nel cloud, sul server.",
  "settings.spokenLanguage": "Lingua parlata",
  "settings.spokenLanguageHint":
    "La lingua parlata nelle tue riunioni. Il rilevamento automatico va bene per la maggior parte delle registrazioni.",
  "settings.autoDetect": "Rilevamento automatico",

  "settings.autoSummarize": "Riepiloga le riunioni automaticamente",
  "settings.autoSummarizeHint":
    "Quando una riunione termina, scrive il suo riepilogo senza che venga richiesto. Le riunioni di meno di un minuto vengono ignorate. Se lo disattivi, una trascrizione viene inviata al server dei riepiloghi solo quando premi tu «Genera il riepilogo».",
  "settings.summaryLanguage": "Lingua del riepilogo",
  "settings.summaryLanguageHint":
    "«Come la trascrizione» mantiene la lingua della riunione.",
  "settings.matchTranscript": "Come la trascrizione",
  "settings.summaryInstructions": "Istruzioni per il riepilogo (facoltativo)",
  "settings.summaryInstructionsHint":
    "Si applicano a ogni riepilogo che generi. Lascia vuoto per il comportamento predefinito. Puoi anche aggiungere istruzioni per singola riunione prima di generare un riepilogo.",

  "settings.telemetry": "Condividi statistiche d'uso anonime",
  "settings.telemetryHint":
    "Ci aiuta a vedere quali funzioni vengono usate, quanto sono rapide e quali errori si verificano.",
  "settings.telemetryDetail":
    "Cosa viene inviato: conteggi di utilizzo delle funzioni, intervalli di durata, categorie di errore, versione dell'app, sistema operativo e versione, tipo di CPU e numero di core, e un identificativo di installazione casuale che puoi reimpostare. Cosa non viene mai inviato: le tue registrazioni, trascrizioni, riepiloghi, titoli delle riunioni, nomi dei partecipanti, percorsi dei file, né nulla di ciò che scrivi o dici. Se l'app è offline, i report attendono in un piccolo file su questo dispositivo e vengono inviati più tardi. I report sono conservati per 12 mesi. Disattivando questa opzione ogni invio si interrompe immediatamente, tutto ciò che è ancora in attesa su questo dispositivo viene eliminato e l'identificativo di installazione viene rimosso.",

  "settings.serverUrl": "URL del server",
  "settings.serverUrlLocked":
    "Bloccato: configurato in fase di build dalla CI ({url}).",
  "settings.serverUrlEmbedded": "incorporato",
  "settings.serverUrlHint":
    "I server remoti devono usare https://; http:// funziona solo per localhost.",
  "settings.accessToken": "Token di accesso",
  "settings.tokenFromBuild":
    "Configurato in fase di build dalla CI e salvato nel keychain del sistema.",
  "settings.tokenFromEnv": "Impostato da DESKSEC_TOKEN in .env.",
  "settings.tokenInKeychain": "Salvato nel keychain del sistema.",
  "settings.tokenMissing":
    "Imposta DESKSEC_TOKEN in .env (vedi .env.example) per i riepiloghi con IA.",
  "settings.deviceId": "ID dispositivo",
  "settings.deviceIdHint":
    "Identifica questa installazione sul server. Comunicalo all'IT per richiedere la revoca dell'accesso di questo dispositivo.",
  "settings.summaryModel": "Modello per i riepiloghi",
  "settings.chunkLength": "Lunghezza del blocco",
  "settings.chunkLengthHint":
    "Secondi. A ogni blocco vengono prodotti segmenti di trascrizione definitivi.",
  "settings.partialInterval": "Intervallo provvisorio",
  "settings.partialIntervalHint":
    "Secondi, 0 = disattivato. Il testo provvisorio si aggiorna a questo intervallo. Entrambi vengono eseguiti sul dispositivo.",
  "settings.exportMarkdown": "Esporta le riunioni completate in ~/meetings",
  "settings.exportMarkdownHint":
    "Replica ogni riunione completata in markdown, così la CLI Minutes inclusa, gli strumenti MCP e il grafo delle relazioni possono leggerla.",

  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "La trascrizione compare mentre si parla.",
  "settings.connectionCheckFailed": "Non è stato possibile verificare la connessione",
  "serverUrl.enterFull": "Inserisci un URL completo, es. https://minutes.example.com o http://localhost:8787.",
  "serverUrl.onlyHttp": "Sono supportati solo URL http:// e https://.",
  "serverUrl.httpsRequired": "I server remoti devono usare https://; con il semplice http:// il tuo token e la tua trascrizione verrebbero inviati in chiaro. (http:// è consentito solo per localhost.)",
  "model.tiny": "Tiny — il più rapido, precisione minima",
  "model.base": "Base — rapido",
  "model.small": "Small — consigliato",
  "model.medium": "Medium — più preciso, più lento",
  "model.largeV3": "Large v3 — massima precisione, il più lento",
  "model.unknownSize": "dimensione sconosciuta",
  "error.unknown": "Si è verificato un errore sconosciuto.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Documento Word",
  "dialog.pdf": "PDF",

  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Lingua e regione",
  "settings.blurb.languageRegion": "La lingua dell'app e il modo in cui vengono mostrate le date.",
  "settings.dateFormat": "Date e ore",
  "settings.dateFormatHint": "Vengono mostrate nel formato regionale di questo dispositivo, ripreso dal sistema operativo. Modificalo nelle impostazioni di sistema.",

  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Connesso",
  "settings.notConfigured": "Non configurato",
  "settings.unreachable": "Non raggiungibile",

  "model.vad": "Rilevamento dell'attività vocale",
  "model.diarization": "Identificazione degli interlocutori",

  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "Quella riunione è in registrazione: interrompila prima di eliminarla.",
  "error.meetingNotFound": "Quella riunione non è stata trovata.",
  "error.nothingToShare": "Per questa riunione non c'è ancora nulla da condividere.",
  "error.shareUnsupported": "La condivisione con un'altra app non è disponibile su questa piattaforma: salva il file invece.",
  "error.noWindowToShare": "La finestra principale non è disponibile per condividere.",
  "error.stopBeforeEngineChange": "Interrompi la registrazione prima di cambiare il motore di trascrizione.",
  "error.noCaptureSource": "Attiva il microfono, l'audio di sistema o entrambi.",
  "error.exportPathNotAbsolute": "Non è stato possibile usare quel percorso di salvataggio.",
  "error.exportExtension": "Minutes non può scrivere quel tipo di file.",
  "error.stopBeforeDeletingModels": "Interrompi la registrazione prima di eliminare i modelli.",

  "error.serverTokenMissing": "Il token di accesso al server Minutes non è configurato. Controlla le Impostazioni o contatta l'IT.",
  "error.serverRejectedToken": "Il server Minutes ha rifiutato il token di accesso. Controlla le Impostazioni o contatta l'IT.",
  "error.onlineNotConfiguredOnServer": "La trascrizione online non è configurata sul server Minutes. Contatta l'IT.",
  "error.unknownBrowser": "Minutes non è in grado di rilevare riunioni in quel browser.",
  "error.noPrivacyPane": "Questo sistema non ha una pagina di impostazioni per quell’autorizzazione.",

  /* ---------------- Prima configurazione ---------------- */
  "onboarding.stepOf": "Passaggio {current} di {total}",
  "onboarding.skipAll": "Salta la configurazione",
  "onboarding.back": "Indietro",
  "onboarding.continue": "Continua",
  "onboarding.skipStep": "Non ora",
  "onboarding.openSettings": "Apri Impostazioni di Sistema",
  "onboarding.allowed": "Consentito",
  "onboarding.notAllowed": "Non consentito",
  "onboarding.notSetUp": "Non configurato",
  "onboarding.checking": "Controllo in corso…",

  "onboarding.welcomeTitle": "Benvenuto in Minutes",
  "onboarding.welcomeBody":
    "Minutes registra le tue riunioni e ne redige il verbale. Prima di iniziare servono alcune autorizzazioni.",
  "onboarding.welcomeOptional": "Ogni passaggio è facoltativo e potrai modificare tutto in seguito nelle impostazioni.",
  "onboarding.getStarted": "Iniziamo",

  "onboarding.microphoneTitle": "Microfono",
  "onboarding.microphoneBody":
    "Minutes registra l’audio della riunione dal tuo microfono. Nulla viene registrato prima che avvii una riunione.",
  "onboarding.microphoneAllow": "Consenti il microfono",
  "onboarding.microphoneDeniedHint":
    "L’accesso al microfono è stato rifiutato e macOS lo chiede una sola volta. Puoi attivarlo in «Privacy e sicurezza» → «Microfono».",
  "onboarding.microphoneWindowsHint":
    "Windows non chiede questa autorizzazione alle applicazioni. Se la registrazione risulta muta, verifica che l’accesso al microfono sia attivo per le app desktop in «Privacy e sicurezza» → «Microfono».",

  "onboarding.browserTitle": "Riunioni aperte nel browser",
  "onboarding.browserBody":
    "Per proporti di prendere appunti quando entri in una riunione Google Meet o Teams da un link, Minutes verifica se nel browser è aperta una riunione.",
  "onboarding.browserPrivacy":
    "Guarda soltanto se una scheda è una riunione — non il contenuto delle pagine, e nulla lascia il tuo dispositivo.",
  "onboarding.browserPerApp":
    "macOS concede questa autorizzazione un browser alla volta, perciò sono elencati separatamente. Vengono mostrati solo i browser che hai installato.",
  "onboarding.browserAllow": "Consenti",
  "onboarding.browserNone":
    "Non è stato trovato alcun browser supportato, quindi qui non c’è nulla da configurare. Le riunioni in Zoom, Teams e Slack vengono rilevate senza questo.",
  "onboarding.browserDeniedHint":
    "macOS lo chiede una sola volta per ciascun browser. Per modificarlo vai in «Privacy e sicurezza» → «Automazione» e seleziona Minutes sotto quel browser.",


  "onboarding.detectionUnavailableTitle": "Avviare una riunione",
  "onboarding.detectionUnavailableBody":
    "Il rilevamento automatico delle riunioni è disponibile per ora solo su macOS. Su questo sistema avvia tu la registrazione con «Nuova riunione» quando vuoi che si prendano appunti.",

  "onboarding.doneTitle": "Tutto pronto",
  "onboarding.doneBody": "Ecco la situazione attuale. Puoi rivedere tutto questo nelle impostazioni.",
  "onboarding.doneSkipped": "Saltato: potrai configurarlo più tardi nelle impostazioni.",
  "onboarding.finish": "Inizia a usare Minutes",

  "settings.rerunOnboarding": "Autorizzazioni e configurazione",
  "settings.rerunOnboardingHint":
    "Ripeti la configurazione di microfono e rilevamento nel browser.",
  "settings.rerunOnboardingAction": "Avvia la configurazione",
};
