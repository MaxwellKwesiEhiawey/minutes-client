import type { Translations } from "./index";

/**
 * French. Addressed formally ("vous"), with the spacing conventions French
 * typography expects — a non-breaking space before `:` and `?`, and « » for
 * quotations.
 *
 * The privacy strings say exactly what the English says: `share.includeOff` must
 * make clear nothing spoken verbatim is in the file, and
 * `settings.telemetryDetail` must not soften what is and is not transmitted.
 */
export const fr: Translations = {
  "common.close": "Fermer",
  "common.cancel": "Annuler",
  "common.delete": "Supprimer",
  "common.done": "Terminé",
  "common.open": "Ouvrir",
  "common.tryAgain": "Réessayer",
  "common.retrying": "Nouvelle tentative…",
  "common.loading": "Chargement…",
  "common.yes": "Oui",
  "common.none": "—",
  "common.starting": "Démarrage…",

  "nav.home": "Accueil",
  "nav.myNotes": "Mes notes",
  "nav.settings": "Réglages",
  "nav.newMeeting": "Nouvelle réunion",
  "nav.brandHome": "Accueil Minutes",
  "nav.main": "Navigation principale",

  "topbar.toggleSidebar": "Afficher ou masquer le panneau latéral",
  "topbar.search": "Rechercher dans les réunions et les transcriptions…",
  "topbar.searchLabel": "Rechercher dans les réunions et les transcriptions",
  "topbar.themeTitle": "Thème : {theme} — cliquez pour changer",
  "topbar.recordingOpen": "Aller à la réunion en cours d'enregistrement",
  "topbar.recordingStop": "Terminer la réunion en cours d'enregistrement",
  "topbar.stop": "Arrêter",
  "theme.light": "Clair",
  "theme.dark": "Sombre",
  "theme.system": "Système",

  "page.home": "Accueil",
  "page.notes": "Mes notes",
  "page.settings": "Réglages",
  "page.recording": "Enregistrement",
  "page.meeting": "Réunion",

  "home.greeting": "Bonjour, bienvenue !",
  "home.sub":
    "Prêt à transformer votre prochaine conversation en quelque chose d'utile ?",
  "home.recent": "Réunions récentes",
  "home.viewAll": "Voir toutes les notes →",
  "home.summaryReady": "Résumé prêt",
  "home.transcriptOnly": "Transcription seule",
  "home.emptyTitle": "Vos réunions apparaîtront ici",
  "home.emptyBody":
    "Démarrez votre première réunion et Minutes capte la conversation, rédige un résumé et organise tout pour vous.",
  "home.emptyCta": "Démarrer une réunion",

  "notes.title": "Mes notes",
  "notes.sub": "Toutes les réunions que Minutes a captées sur cet appareil.",
  "notes.results": "Résultats pour « {query} »",
  "notes.clearSearchText": "Effacer le texte de recherche",
  "notes.colMeeting": "Réunion",
  "notes.colDate": "Date",
  "notes.colDuration": "Durée",
  "notes.colSummary": "Résumé",
  "notes.colStatus": "Statut",
  "notes.moreActions": "Autres actions pour {title}",
  "notes.stopBeforeDelete": "Arrêtez l'enregistrement avant de supprimer",
  "notes.emptySearchTitle": "Aucune réunion ne correspond à votre recherche",
  "notes.emptySearchBody":
    "Essayez un autre mot ou une autre expression — la recherche couvre les titres et le texte des transcriptions.",
  "notes.clearSearch": "Effacer la recherche",
  "notes.emptyTitle": "Aucune réunion pour l'instant",
  "notes.emptyBody":
    "Démarrez une réunion et elle apparaîtra ici avec sa transcription et son résumé.",

  "status.recording": "Enregistrement",
  "status.completed": "Terminée",
  "status.interrupted": "Interrompue",

  "detail.back": "Retour à Mes notes",
  "detail.share": "Partager et exporter",
  "detail.delete": "Supprimer la réunion",
  "detail.tabSummary": "Résumé",
  "detail.tabTranscript": "Transcription",
  "detail.tabsLabel": "Panneaux de la réunion",
  "detail.generate": "Générer le résumé",
  "detail.regenerate": "Régénérer le résumé",
  "detail.summarizing": "Résumé en cours…",
  "detail.generateTitle": "Générer un résumé par IA",
  "detail.generateDisabled":
    "Aucune transcription n'a encore été captée pour cette réunion",
  "detail.instructionsToggle": "Ajouter des instructions",
  "detail.instructionsLabel": "Instructions pour ce résumé (facultatif)",
  "detail.instructionsPlaceholder":
    "ex. Ne pas inclure les noms des personnes mentionnées dans la réunion.",
  "detail.instructionsCombined":
    "Combinées avec vos instructions de résumé par défaut dans les Réglages.",
  "detail.instructionsApplied":
    "Appliquées lorsque vous générez ou régénérez le résumé.",
  "detail.writingSummary":
    "Rédaction de votre résumé — cela prend généralement une minute environ.",
  "detail.noSummaryTitle": "Pas encore de résumé",
  "detail.noSummaryReady":
    "Générez-en un à partir de la transcription quand vous voulez.",
  "detail.noSummaryNoTranscript":
    "Un résumé a besoin d'une transcription — rien n'a été capté pour cette réunion.",
  "detail.noTranscriptTitle": "Aucune transcription captée",
  "detail.noTranscriptBody":
    "Cette réunion ne contient aucun segment de transcription.",
  "detail.speaker": "Intervenant",

  "summaryError.networkTitle": "Le serveur de résumés est injoignable.",
  "summaryError.networkHint":
    "Vérifiez votre connexion réseau et l'URL du serveur dans les Réglages, puis réessayez.",
  "summaryError.timeoutTitle":
    "Le serveur de résumés a mis trop de temps à répondre.",
  "summaryError.timeoutHint":
    "Cela peut arriver sur une connexion lente ou avec une très longue transcription. Réessayez.",
  "summaryError.authTitle":
    "Le serveur de résumés a refusé la requête (non autorisée).",
  "summaryError.authHint":
    "Votre jeton d'accès Minutes est peut-être absent ou invalide — vérifiez les Réglages ou contactez le service informatique.",
  "summaryError.serverTitle": "Le serveur de résumés a renvoyé une erreur.",
  "summaryError.genericTitle": "Impossible de générer un résumé.",

  "summary.aiNote":
    "Généré par IA à partir de la transcription · à relire avant partage",
  "summary.overview": "Vue d'ensemble",
  "summary.keyPoints": "Points clés de la discussion",
  "summary.decisions": "Décisions",
  "summary.actionItems": "Actions à mener",
  "summary.openQuestions": "Questions en suspens",
  "summary.openQuestion": "Question en suspens",
  "summary.owner": "responsable : {name}",
  "summary.assignedTo": "Assigné à : {name}",
  "summary.due": "Échéance : {date}",
  "summary.generatedBy": "Généré par {model} · {date}",

  "recording.back": "Retour à Mes notes",
  "recording.transcriptSaved": "La transcription est enregistrée en direct",
  "recording.endMeeting": "Terminer la réunion",
  "recording.inputLevel": "Niveau d'entrée",
  "recording.liveTranscript": "Transcription en direct",
  "recording.savedAsCaptured": "Enregistrée au fil de la captation",
  "recording.nothingYet": "Rien de capté pour l'instant",
  "recording.listening":
    "À l'écoute — la transcription apparaît à mesure que les gens parlent.",
  "recording.interim": "Transcription provisoire",
  "recording.live": "En direct",

  "engine.onDevice": "Privé · sur cet appareil",
  "engine.onDeviceTitle":
    "La transcription s'exécute sur cet appareil (modèle Whisper : {model})",
  "engine.cloud": "Transcription dans le cloud",
  "engine.cloudTitle": "La transcription s'exécute en ligne (Deepgram)",

  "palette.label": "Recherche",
  "palette.placeholder": "Rechercher dans les réunions et les transcriptions…",
  "palette.recent": "Récent",
  "palette.meetings": "Réunions",
  "palette.transcripts": "Transcriptions",
  "palette.noResults": "Aucun résultat pour « {query} »",
  "palette.noResultsHint":
    "Essayez le nom d'un intervenant ou une expression de la conversation.",
  "palette.nothingYet": "Rien à rechercher pour l'instant",
  "palette.nothingYetHint":
    "Enregistrez une réunion et elle deviendra consultable ici.",

  "share.title": "Partager et exporter",
  "share.includeTranscript": "Inclure la transcription complète",
  "share.includeOn":
    "Le fichier contiendra le résumé et tout ce qui a été dit.",
  "share.includeOff":
    "Le fichier contiendra uniquement le résumé — rien de ce qui a été dit mot pour mot.",
  "share.includeForced":
    "Il n'y a pas encore de résumé : la transcription constitue tout le document.",
  "share.includeNone": "Cette réunion n'a aucune transcription à inclure.",
  "share.format": "Format",
  "share.formatHint": "Utilisé pour l'envoi comme pour l'enregistrement.",
  "share.formatPlaceholder": "Choisir un format…",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "Envoyer vers une app…",
  "share.sendToAppTitle": "Transmettre le fichier à une autre application",
  "share.saveToDevice": "Enregistrer sur cet appareil…",
  "share.saveToDeviceTitle": "Enregistrer le fichier sur cet appareil",
  "share.gateHint":
    "Choisissez un format ci-dessus pour envoyer ou enregistrer.",
  "share.nothingToShare":
    "Cette réunion n'a encore ni résumé ni transcription à mettre dans un fichier.",
  "share.copyGroup": "Copier dans le presse-papiers",
  "share.copySummary": "Copier le résumé",
  "share.copySummaryTitle": "Copier le résumé IA au format Markdown",
  "share.copyTranscript": "Copier la transcription",
  "share.copyTranscriptTitle": "Copier le texte brut de la transcription",

  "toast.exportedMarkdown": "Fichier Markdown exporté.",
  "toast.exportedWord": "Document Word exporté.",
  "toast.exportedPdf": "PDF exporté.",
  "toast.copiedSummary": "Résumé copié dans le presse-papiers.",
  "toast.copiedTranscript": "Transcription copiée dans le presse-papiers.",
  "toast.meetingDeleted": "Réunion supprimée.",
  "toast.transcription": "Transcription : {message}",
  "toast.audio": "Audio : {message}",
  "toast.serverNotSetUp":
    "Le serveur de résumés Minutes n'est pas encore configuré. Définissez DESKSEC_TOKEN dans .env ou contactez le service informatique.",
  "toast.downloadModelFirst":
    "Téléchargez le modèle de transcription « {model} » dans les Réglages avant d'enregistrer.",
  "toast.configureOnline":
    "Configurez la transcription en ligne dans les Réglages (jeton du serveur et DEEPGRAM_API_KEY sur le serveur).",
  "toast.summarizeFailed": "Impossible de résumer cette réunion : {message}",

  "confirm.deleteTitle": "Supprimer la réunion",
  "confirm.deleteBody":
    "Supprimer {name} ainsi que sa transcription et son résumé ? Cette action est irréversible.",
  "confirm.deleteThis": "cette réunion",

  "settingsLoading.label": "Chargement des réglages",
  "settingsLoading.message": "Chargement des réglages…",

  "prompt.dismiss": "Ignorer",
  "prompt.callDetected": "{app} détecté",
  "prompt.newMeeting": "Nouvelle réunion",
  "prompt.callHeading": "Prendre des notes pour cet appel ?",
  "prompt.callSub": "Minutes captera la conversation et rédigera vos notes.",
  "prompt.manualHeading": "Démarrer une réunion",
  "prompt.manualSub":
    "Nommez-la maintenant, ou laissez et renommez plus tard.",
  "prompt.takeNotes": "Prendre des notes",
  "prompt.startRecording": "Démarrer l'enregistrement",
  "prompt.notNow": "Pas maintenant",
  "prompt.meetingTitle": "Titre de la réunion",
  "prompt.callPlaceholder": "Notes {app}",
  "prompt.manualPlaceholder": "Réunion sans titre",
  "prompt.hintStart": "démarrer",
  "prompt.hintClose": "fermer",
  "prompt.errorHeading": "Invite de réunion",
  "prompt.errorBody":
    "Une erreur s'est produite lors du chargement de cette invite.",
  "prompt.loadFailed":
    "Impossible de charger l'invite de réunion. Fermez et réessayez.",
  "prompt.listening": "À l'écoute",
  "prompt.call": "Appel",

  "settings.title": "Réglages",
  "settings.sectionsLabel": "Sections des réglages",
  "settings.applyImmediately":
    "Les modifications sont appliquées au fur et à mesure",
  "settings.saving": "Enregistrement…",
  "settings.saved": "Enregistré",
  "settings.server": "Serveur de résumés Minutes",
  "settings.checking": "Vérification…",
  "settings.unknown": "Inconnu",
  "settings.serverUnreachableConfigured":
    "Les résumés par IA nécessitent une connexion fonctionnelle. La transcription continue de s'exécuter entièrement sur l'appareil. Contactez le service informatique si cela persiste.",
  "settings.serverUnlinked":
    "Les résumés ne sont pas encore reliés au serveur. La transcription fonctionne toujours hors ligne. Contactez le service informatique pour la configuration.",

  "settings.tab.appearance": "Apparence",
  "settings.blurb.appearance": "Clair, sombre, ou suivre le système.",
  "settings.tab.reading": "Confort de lecture",
  "settings.blurb.reading":
    "Taille du texte et interligne des transcriptions, enregistrés sur cet appareil.",
  "settings.tab.audio": "Audio",
  "settings.blurb.audio":
    "Choisissez ce que Minutes capte pendant l'enregistrement.",
  "settings.tab.callDetection": "Détection d'appel",
  "settings.blurb.callDetection":
    "Proposer de prendre des notes quand une application d'appel utilise votre micro.",
  "settings.tab.transcription": "Transcription",
  "settings.blurb.transcription":
    "Moteur, modèle de précision, intervenants et langue parlée.",
  "settings.tab.summary": "Résumé",
  "settings.blurb.summary": "Quand les résumés par IA sont rédigés, et comment.",
  "settings.tab.privacy": "Confidentialité",
  "settings.blurb.privacy": "Ce qui quitte cet appareil.",
  "settings.tab.advanced": "Avancé",
  "settings.blurb.advanced":
    "Pour l'informatique et le développement. La plupart des utilisateurs peuvent laisser ces valeurs telles quelles.",

  "settings.language": "Langue",
  "settings.languageHint":
    "La langue des libellés et messages de l'application, enregistrée sur cet appareil. Les messages provenant du serveur ne sont pas traduits.",

  "settings.textSize": "Taille du texte de la transcription",
  "settings.textSizeHint": "S'applique à la vue transcription.",
  "settings.sizeDefault": "Par défaut",
  "settings.sizeLarge": "Grande",
  "settings.sizeXLarge": "Très grande",
  "settings.lineSpacing": "Interligne",
  "settings.spacingDefault": "Par défaut",
  "settings.spacingRelaxed": "Aéré",
  "settings.spacingLoose": "Très aéré",
  "settings.highContrast": "Texte à contraste élevé",
  "settings.reduceMotion": "Réduire les animations",
  "settings.reduceMotionHint": "Moins d'animation dans toute l'application.",
  "settings.readingOnThisDevice":
    "Ces préférences sont enregistrées sur cet appareil.",

  "settings.captureMic": "Capter mon microphone",
  "settings.microphone": "Microphone",
  "settings.microphoneHint":
    "L'enregistrement suit l'appareil : si un casque Bluetooth se déconnecte en pleine réunion, la captation continue sur le microphone qui prend le relais.",
  "settings.systemDefault": "Réglage du système",
  "settings.captureSystemAudio": "Capter aussi l'audio du système",
  "settings.captureSystemAudioHint":
    "Enregistre ce que vous entendez dans Zoom, Meet, Teams et d'autres applications — sans robot de réunion. Tant que ceci est activé, tout ce qui joue sur cet appareil est enregistré.",
  "settings.systemAudioSource": "Source de l'audio système",
  "settings.defaultOutput": "Sortie par défaut",
  "settings.noCaptureSource":
    "Activez le microphone, l'audio du système, ou les deux — un enregistrement a besoin d'une source.",
  "settings.loopbackLinux":
    "Aucun moniteur d'audio système trouvé. Avec PipeWire ou PulseAudio, cherchez une source nommée « Monitor of … » dans vos réglages audio, puis réouvrez les Réglages.",
  "settings.loopbackWindows":
    "Aucune source d'audio système trouvée. Branchez des haut-parleurs ou un casque, puis réouvrez les Réglages. Stereo Mix ou VB-Audio Cable fonctionnent aussi s'ils sont listés.",
  "settings.loopbackMacos":
    "Aucun périphérique de rebouclage trouvé. macOS nécessite un pilote audio virtuel (par ex. BlackHole). Installez-en un, puis réouvrez les Réglages.",
  "settings.loopbackUnknown":
    "Aucun périphérique de rebouclage détecté pour l'audio système. Une source moniteur/rebouclage est nécessaire pour capter l'audio d'une réunion sans robot.",

  "settings.callPrompt":
    "Proposer quand une application d'appel utilise le microphone",
  "settings.callPromptHint":
    "Affiche une carte flottante « Prendre des notes » quand Zoom, Teams (application ou navigateur), Google Meet, Slack, FaceTime, WhatsApp ou Webex utilise le micro pendant que Minutes est ouvert. Meet/Teams dans le navigateur nécessitent l'accès Automatisation pour Chrome/Safari dans les Réglages Système.",
  "settings.callCooldown": "Délai après avoir ignoré",
  "settings.callCooldownHint":
    "Minutes à attendre avant de proposer à nouveau.",
  "settings.callUnsupported":
    "La détection d'appel est disponible sur macOS. Vous pouvez toujours démarrer des réunions manuellement avec « Nouvelle réunion ».",

  "settings.engine": "Moteur",
  "settings.engineWhisperHint":
    "La reconnaissance vocale s'exécute localement avec un modèle Whisper. Votre audio ne quitte jamais cet appareil pour la transcription.",
  "settings.engineCloudHint":
    "L'audio est diffusé en direct vers votre serveur Minutes (Deepgram Live) pour des sous-titres à faible latence. Utilise la même URL de serveur et le même jeton d'accès que les résumés par IA.",
  "settings.engineCloud": "En ligne (serveur Minutes · Deepgram)",
  "settings.engineWhisper": "Sur l'appareil (Whisper)",
  "settings.statusLabel": "État",
  "settings.onlineReady": "La transcription en ligne est prête ({model}).",
  "settings.onlineNotConfigured":
    "Configurez DESKSEC_TOKEN et assurez-vous que le serveur possède DEEPGRAM_API_KEY.",
  "settings.accuracyModel": "Modèle de précision",
  "settings.modelFiles": "Fichiers du modèle",
  "settings.modelDownloading": "Téléchargement de {label}…",
  "settings.modelReady": "Le modèle « {model} » est téléchargé et prêt.",
  "settings.modelMissing":
    "Le modèle « {model} » n'est pas encore téléchargé — requis avant d'enregistrer.",
  "settings.redownload": "Télécharger à nouveau",
  "settings.downloadModel": "Télécharger le modèle",
  "settings.downloadProgress": "Progression du téléchargement",
  "settings.downloadOnce":
    "Le modèle {model} pèse environ {size}. Cela n'a lieu qu'une fois — laissez cette fenêtre ouverte jusqu'à la fin.",
  "settings.downloadedModels":
    "Modèles téléchargés ({size} sur le disque). Touchez ici pour supprimer",
  "settings.downloadedModelsHint":
    "Supprimez les modèles dont vous n'avez plus besoin. Utilisez « Télécharger le modèle » ci-dessus pour les récupérer.",
  "settings.inUse": " · en cours d'utilisation",
  "settings.deleteQuestion": "Supprimer ?",
  "settings.deleting": "Suppression…",
  "settings.stopBeforeDeletingModels":
    "Arrêtez l'enregistrement avant de supprimer des modèles.",
  "settings.identifySpeakers": "Identifier les intervenants",
  "settings.identifySpeakersWhisper":
    "Indique qui a parlé dans chaque segment. Télécharge un petit modèle d'intervenants à la première utilisation.",
  "settings.identifySpeakersCloud":
    "Indique qui a parlé dans chaque segment via la diarisation dans le cloud, sur le serveur.",
  "settings.spokenLanguage": "Langue parlée",
  "settings.spokenLanguageHint":
    "La langue parlée dans vos réunions. La détection automatique convient à la plupart des enregistrements.",
  "settings.autoDetect": "Détection automatique",

  "settings.autoSummarize": "Résumer les réunions automatiquement",
  "settings.autoSummarizeHint":
    "À la fin d'une réunion, rédige son résumé sans qu'on le demande. Les réunions de moins d'une minute sont ignorées. Si vous désactivez ceci, une transcription n'est envoyée au serveur de résumés que lorsque vous appuyez vous-même sur « Générer le résumé ».",
  "settings.summaryLanguage": "Langue du résumé",
  "settings.summaryLanguageHint":
    "« Comme la transcription » conserve la langue de la réunion.",
  "settings.matchTranscript": "Comme la transcription",
  "settings.summaryInstructions": "Instructions de résumé (facultatif)",
  "settings.summaryInstructionsHint":
    "Appliquées à chaque résumé que vous générez. Laissez vide pour le comportement par défaut. Vous pouvez aussi ajouter des instructions par réunion avant de générer un résumé.",

  "settings.telemetry": "Partager des statistiques d'usage anonymes",
  "settings.telemetryHint":
    "Nous aide à voir quelles fonctionnalités sont utilisées, à quelle vitesse elles fonctionnent et quelles erreurs se produisent.",
  "settings.telemetryDetail":
    "Ce qui est envoyé : le nombre d'utilisations des fonctionnalités, des plages de durée, des catégories d'erreur, la version de l'application, le système d'exploitation et sa version, le type de processeur et son nombre de cœurs, ainsi qu'un identifiant d'installation aléatoire que vous pouvez réinitialiser. Ce qui n'est jamais envoyé : vos enregistrements, transcriptions, résumés, titres de réunion, noms de participants, chemins de fichiers, ni rien de ce que vous tapez ou dites. Si l'application est hors ligne, les rapports attendent dans un petit fichier sur cet appareil et sont envoyés plus tard. Les rapports sont conservés 12 mois. Désactiver ceci arrête immédiatement tout envoi, supprime tout ce qui attend encore sur cet appareil et supprime l'identifiant d'installation.",

  "settings.startAtLogin": "Lancer à l'ouverture de session",
  "settings.startAtLoginHint":
    "Exécute Minutes en arrière-plan dès l'ouverture de session afin de détecter les réunions avant que vous n'ouvriez l'application.",
  "settings.startAtLoginHintNoDetection":
    "Exécute Minutes en arrière-plan dès l'ouverture de session pour qu'il soit prêt immédiatement. La détection automatique des réunions n'est pas disponible sur cette plateforme.",
  "settings.serverUrl": "URL du serveur",
  "settings.serverUrlLocked":
    "Verrouillé — configuré au moment de la compilation par la CI ({url}).",
  "settings.serverUrlEmbedded": "intégré",
  "settings.serverUrlHint":
    "Les serveurs distants doivent utiliser https:// — http:// ne fonctionne que pour localhost.",
  "settings.accessToken": "Jeton d'accès",
  "settings.tokenFromBuild":
    "Configuré au moment de la compilation par la CI et stocké dans le trousseau du système.",
  "settings.tokenFromEnv": "Défini depuis DESKSEC_TOKEN dans .env.",
  "settings.tokenInKeychain": "Stocké dans le trousseau du système.",
  "settings.tokenMissing":
    "Définissez DESKSEC_TOKEN dans .env (voir .env.example) pour les résumés par IA.",
  "settings.deviceId": "Identifiant de l'appareil",
  "settings.deviceIdHint":
    "Identifie cette installation auprès du serveur. Communiquez-le au service informatique pour faire révoquer l'accès de cet appareil.",
  "settings.summaryModel": "Modèle de résumé",
  "settings.chunkLength": "Longueur des blocs",
  "settings.chunkLengthHint":
    "Secondes. Des segments de transcription définitifs sont produits à chaque bloc.",
  "settings.partialInterval": "Intervalle provisoire",
  "settings.partialIntervalHint":
    "Secondes, 0 = désactivé. Le texte provisoire est actualisé à cet intervalle. Les deux s'exécutent sur l'appareil.",
  "settings.exportMarkdown": "Exporter les réunions terminées vers ~/meetings",
  "settings.exportMarkdownHint":
    "Reproduit chaque réunion terminée en markdown afin que la CLI Minutes fournie, les outils MCP et le graphe de relations puissent la lire.",

  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "La transcription apparaît à mesure que les gens parlent.",
  "settings.connectionCheckFailed": "Impossible de vérifier la connexion",
  "serverUrl.enterFull": "Saisissez une URL complète, par ex. https://minutes.example.com ou http://localhost:8787.",
  "serverUrl.onlyHttp": "Seules les URL http:// et https:// sont prises en charge.",
  "serverUrl.httpsRequired": "Les serveurs distants doivent utiliser https:// — en http:// simple, votre jeton et votre transcription seraient envoyés en clair. (http:// n'est autorisé que pour localhost.)",
  "model.tiny": "Tiny — le plus rapide, précision la plus faible",
  "model.base": "Base — rapide",
  "model.small": "Small — recommandé",
  "model.medium": "Medium — plus précis, plus lent",
  "model.largeV3": "Large v3 — meilleure précision, le plus lent",
  "model.unknownSize": "taille inconnue",
  "error.unknown": "Une erreur inconnue s'est produite.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Document Word",
  "dialog.pdf": "PDF",

  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Langue et région",
  "settings.blurb.languageRegion": "La langue de l'application, et l'affichage des dates.",
  "settings.dateFormat": "Dates et heures",
  "settings.dateFormatHint": "Affichées au format régional de cet appareil, repris du système d'exploitation. Modifiez-le dans les réglages de votre système.",

  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Connecté",
  "settings.notConfigured": "Non configuré",
  "settings.unreachable": "Injoignable",

  "model.vad": "Détection d'activité vocale",
  "model.diarization": "Identification des intervenants",

  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "Cette réunion est en cours d'enregistrement — arrêtez-la avant de la supprimer.",
  "error.noTranscriptCheckLanguage":
    "L'audio parvient au serveur mais rien n'est transcrit. Vérifiez la langue de transcription dans les Réglages : elle doit correspondre à la langue parlée.",
  "error.meetingNotFound": "Cette réunion est introuvable.",
  "error.nothingToShare": "Il n'y a encore rien à partager pour cette réunion.",
  "error.shareUnsupported": "Le partage vers une autre application n'est pas disponible sur cette plateforme — enregistrez le fichier à la place.",
  "error.noWindowToShare": "La fenêtre principale n'est pas disponible pour partager.",
  "error.stopBeforeEngineChange": "Arrêtez l'enregistrement avant de changer de moteur de transcription.",
  "error.noCaptureSource": "Activez le microphone, l'audio du système, ou les deux.",
  "error.exportPathNotAbsolute": "Cet emplacement d'enregistrement n'a pas pu être utilisé.",
  "error.exportExtension": "Minutes ne peut pas écrire ce type de fichier.",
  "error.stopBeforeDeletingModels": "Arrêtez l'enregistrement avant de supprimer des modèles.",

  "error.serverTokenMissing": "Le jeton d'accès au serveur Minutes n'est pas configuré. Vérifiez les Réglages ou contactez le service informatique.",
  "error.deviceRevoked":
    "L'accès de cet appareil a été révoqué. Contactez votre service informatique pour le rétablir.",
  "error.serverRejectedToken": "Le serveur Minutes a refusé le jeton d'accès. Vérifiez les Réglages ou contactez le service informatique.",
  "error.onlineNotConfiguredOnServer": "La transcription en ligne n'est pas configurée sur le serveur Minutes. Contactez le service informatique.",
  "error.unknownBrowser": "Minutes ne peut pas détecter de réunions dans ce navigateur.",
  "error.noPrivacyPane": "Ce système n’a pas de page de réglages pour cette autorisation.",

  /* ---------------- Première configuration ---------------- */
  "onboarding.stepOf": "Étape {current} sur {total}",
  "onboarding.skipAll": "Ignorer la configuration",
  "onboarding.back": "Retour",
  "onboarding.continue": "Continuer",
  "onboarding.skipStep": "Plus tard",
  "onboarding.openSettings": "Ouvrir les Réglages Système",
  "onboarding.allowed": "Autorisé",
  "onboarding.notAllowed": "Non autorisé",
  "onboarding.notSetUp": "Non configuré",
  "onboarding.checking": "Vérification…",

  "onboarding.welcomeTitle": "Bienvenue dans Minutes",
  "onboarding.welcomeBody":
    "Minutes enregistre vos réunions et rédige le compte rendu. Avant de commencer, quelques autorisations sont nécessaires.",
  "onboarding.welcomeOptional": "Chaque étape est facultative et vous pourrez tout modifier plus tard dans les réglages.",
  "onboarding.getStarted": "Commencer",

  "onboarding.microphoneTitle": "Microphone",
  "onboarding.microphoneBody":
    "Minutes enregistre le son de la réunion depuis votre microphone. Rien n’est enregistré avant que vous ne démarriez une réunion.",
  "onboarding.microphoneAllow": "Autoriser le microphone",
  "onboarding.microphoneDeniedHint":
    "L’accès au microphone a été refusé, et macOS ne le demande qu’une seule fois. Vous pouvez l’activer dans « Confidentialité et sécurité » → « Microphone ».",
  "onboarding.microphoneWindowsHint":
    "Windows ne demande pas cette autorisation aux applications. Si l’enregistrement reste muet, vérifiez que l’accès au microphone est activé pour les applications de bureau dans « Confidentialité et sécurité » → « Microphone ».",

  "onboarding.browserTitle": "Réunions ouvertes dans un navigateur",
  "onboarding.browserBody":
    "Pour vous proposer de prendre des notes quand vous rejoignez une réunion Google Meet ou Teams depuis un lien, Minutes vérifie si une réunion est ouverte dans votre navigateur.",
  "onboarding.browserPrivacy":
    "Il regarde seulement si un onglet est une réunion — pas le contenu des pages, et rien ne quitte votre appareil.",
  "onboarding.browserPerApp":
    "macOS accorde cette autorisation navigateur par navigateur, d’où cette liste séparée. Seuls les navigateurs installés apparaissent.",
  "onboarding.browserAllow": "Autoriser",
  "onboarding.browserNone":
    "Aucun navigateur pris en charge n’a été trouvé : rien à configurer ici. Les réunions dans Zoom, Teams et Slack sont détectées sans cela.",
  "onboarding.browserDeniedHint":
    "macOS ne demande qu’une fois par navigateur. Pour changer, allez dans « Confidentialité et sécurité » → « Automatisation » et cochez Minutes sous ce navigateur.",


  "onboarding.detectionUnavailableTitle": "Démarrer une réunion",
  "onboarding.detectionUnavailableBody":
    "La détection automatique des réunions n’est disponible que sur macOS pour le moment. Sur ce système, lancez vous-même l’enregistrement avec « Nouvelle réunion » quand vous voulez des notes.",

  "onboarding.doneTitle": "Vous êtes prêt",
  "onboarding.doneBody": "Voici où en sont les choses. Vous pouvez revenir sur tout cela dans les réglages.",
  "onboarding.doneSkipped": "Ignoré — vous pourrez le configurer plus tard dans les réglages.",
  "onboarding.finish": "Utiliser Minutes",

  "settings.rerunOnboarding": "Autorisations et configuration",
  "settings.rerunOnboardingHint":
    "Reprendre la configuration du microphone et de la détection dans le navigateur.",
  "settings.rerunOnboardingAction": "Lancer la configuration",
};
