import type { Translations } from "./index";

/**
 * Spanish (neutral, usable in both Spain and Latin America). Addressed with
 * "usted" implicitly by preferring impersonal phrasing, which avoids choosing
 * between "tú" and "vos".
 *
 * The privacy strings say exactly what the English says: `share.includeOff` must
 * make clear nothing spoken verbatim is in the file, and
 * `settings.telemetryDetail` must not soften what is and is not transmitted.
 */
export const es: Translations = {
  "common.close": "Cerrar",
  "common.cancel": "Cancelar",
  "common.delete": "Eliminar",
  "common.done": "Listo",
  "common.open": "Abrir",
  "common.tryAgain": "Intentar de nuevo",
  "common.retrying": "Reintentando…",
  "common.loading": "Cargando…",
  "common.yes": "Sí",
  "common.none": "—",
  "common.starting": "Iniciando…",

  "nav.home": "Inicio",
  "nav.myNotes": "Mis notas",
  "nav.settings": "Ajustes",
  "nav.newMeeting": "Nueva reunión",
  "nav.brandHome": "Inicio de Minutes",
  "nav.main": "Navegación principal",

  "topbar.toggleSidebar": "Mostrar u ocultar la barra lateral",
  "topbar.search": "Buscar en reuniones y transcripciones…",
  "topbar.searchLabel": "Buscar en reuniones y transcripciones",
  "topbar.themeTitle": "Tema: {theme} — haz clic para cambiar",
  "topbar.recordingOpen": "Ir a la reunión que se está grabando",
  "topbar.recordingStop": "Finalizar la reunión que se está grabando",
  "topbar.stop": "Detener",
  "theme.light": "Claro",
  "theme.dark": "Oscuro",
  "theme.system": "Sistema",

  "page.home": "Inicio",
  "page.notes": "Mis notas",
  "page.settings": "Ajustes",
  "page.recording": "Grabación",
  "page.meeting": "Reunión",

  "home.greeting": "¡Hola, bienvenido!",
  "home.sub": "¿Listo para convertir tu próxima conversación en algo útil?",
  "home.recent": "Reuniones recientes",
  "home.viewAll": "Ver todas las notas →",
  "home.summaryReady": "Resumen listo",
  "home.transcriptOnly": "Solo transcripción",
  "home.emptyTitle": "Aquí aparecerán tus reuniones",
  "home.emptyBody":
    "Inicia tu primera reunión y Minutes captará la conversación, generará un resumen y lo organizará todo por ti.",
  "home.emptyCta": "Iniciar una reunión",

  "notes.title": "Mis notas",
  "notes.sub": "Todas las reuniones que Minutes ha captado en este dispositivo.",
  "notes.results": "Resultados de «{query}»",
  "notes.clearSearchText": "Borrar el texto de búsqueda",
  "notes.colMeeting": "Reunión",
  "notes.colDate": "Fecha",
  "notes.colDuration": "Duración",
  "notes.colSummary": "Resumen",
  "notes.colStatus": "Estado",
  "notes.moreActions": "Más acciones para {title}",
  "notes.stopBeforeDelete": "Detén la grabación antes de eliminar",
  "notes.emptySearchTitle": "Ninguna reunión coincide con tu búsqueda",
  "notes.emptySearchBody":
    "Prueba otra palabra o frase: la búsqueda abarca títulos y texto de las transcripciones.",
  "notes.clearSearch": "Borrar la búsqueda",
  "notes.emptyTitle": "Todavía no hay reuniones",
  "notes.emptyBody":
    "Inicia una reunión y aparecerá aquí con su transcripción y su resumen.",

  "status.recording": "Grabando",
  "status.completed": "Finalizada",
  "status.interrupted": "Interrumpida",

  "detail.back": "Volver a Mis notas",
  "detail.share": "Compartir y exportar",
  "detail.delete": "Eliminar la reunión",
  "detail.tabSummary": "Resumen",
  "detail.tabTranscript": "Transcripción",
  "detail.tabsLabel": "Paneles de la reunión",
  "detail.generate": "Generar resumen",
  "detail.regenerate": "Volver a generar el resumen",
  "detail.summarizing": "Resumiendo…",
  "detail.generateTitle": "Generar un resumen con IA",
  "detail.generateDisabled":
    "Aún no se ha captado ninguna transcripción de esta reunión",
  "detail.instructionsToggle": "Añadir instrucciones",
  "detail.instructionsLabel": "Instrucciones para este resumen (opcional)",
  "detail.instructionsPlaceholder":
    "p. ej. No incluir los nombres de las personas mencionadas en la reunión.",
  "detail.instructionsCombined":
    "Se combinan con tus instrucciones de resumen predeterminadas de los Ajustes.",
  "detail.instructionsApplied":
    "Se aplican cuando generas o vuelves a generar el resumen.",
  "detail.writingSummary":
    "Escribiendo tu resumen: normalmente tarda alrededor de un minuto.",
  "detail.noSummaryTitle": "Todavía no hay resumen",
  "detail.noSummaryReady":
    "Genera uno a partir de la transcripción cuando quieras.",
  "detail.noSummaryNoTranscript":
    "Un resumen necesita una transcripción: no se captó nada de esta reunión.",
  "detail.noTranscriptTitle": "No se captó ninguna transcripción",
  "detail.noTranscriptBody":
    "Esta reunión no tiene segmentos de transcripción.",
  "detail.speaker": "Hablante",

  "summaryError.networkTitle":
    "No se pudo contactar con el servidor de resúmenes.",
  "summaryError.networkHint":
    "Comprueba tu conexión de red y la URL del servidor en los Ajustes, y vuelve a intentarlo.",
  "summaryError.timeoutTitle":
    "El servidor de resúmenes tardó demasiado en responder.",
  "summaryError.timeoutHint":
    "Puede ocurrir con una conexión lenta o una transcripción muy larga. Vuelve a intentarlo.",
  "summaryError.authTitle":
    "El servidor de resúmenes rechazó la solicitud (no autorizada).",
  "summaryError.authHint":
    "Puede que falte tu token de acceso de Minutes o que no sea válido: revisa los Ajustes o contacta con TI.",
  "summaryError.serverTitle": "El servidor de resúmenes devolvió un error.",
  "summaryError.genericTitle": "No se pudo generar un resumen.",

  "summary.aiNote":
    "Generado por IA a partir de la transcripción · revísalo antes de compartir",
  "summary.overview": "Resumen general",
  "summary.keyPoints": "Puntos clave de la conversación",
  "summary.decisions": "Decisiones",
  "summary.actionItems": "Tareas",
  "summary.openQuestions": "Preguntas abiertas",
  "summary.openQuestion": "Pregunta abierta",
  "summary.owner": "responsable: {name}",
  "summary.assignedTo": "Asignado a: {name}",
  "summary.due": "Fecha límite: {date}",
  "summary.generatedBy": "Generado por {model} · {date}",

  "recording.back": "Volver a Mis notas",
  "recording.transcriptSaved": "La transcripción se guarda en directo",
  "recording.endMeeting": "Finalizar la reunión",
  "recording.inputLevel": "Nivel de entrada",
  "recording.liveTranscript": "Transcripción en directo",
  "recording.savedAsCaptured": "Se guarda a medida que se capta",
  "recording.nothingYet": "Todavía no se ha captado nada",
  "recording.listening":
    "Escuchando: la transcripción aparece a medida que se habla.",
  "recording.interim": "Transcripción provisional",
  "recording.live": "En directo",

  "engine.onDevice": "Privado · en este dispositivo",
  "engine.onDeviceTitle":
    "La transcripción se ejecuta en este dispositivo (modelo Whisper: {model})",
  "engine.cloud": "Transcripción en la nube",
  "engine.cloudTitle": "La transcripción se ejecuta en línea (Deepgram)",

  "palette.label": "Búsqueda",
  "palette.placeholder": "Buscar en reuniones y transcripciones…",
  "palette.recent": "Recientes",
  "palette.meetings": "Reuniones",
  "palette.transcripts": "Transcripciones",
  "palette.noResults": "Sin resultados para «{query}»",
  "palette.noResultsHint":
    "Prueba el nombre de un hablante o una frase de la conversación.",
  "palette.nothingYet": "Todavía no hay nada que buscar",
  "palette.nothingYetHint":
    "Graba una reunión y se podrá buscar aquí.",

  "share.title": "Compartir y exportar",
  "share.includeTranscript": "Incluir la transcripción completa",
  "share.includeOn":
    "El archivo contendrá el resumen y todo lo que se dijo.",
  "share.includeOff":
    "El archivo contendrá solo el resumen: nada de lo que se dijo textualmente.",
  "share.includeForced":
    "Todavía no hay resumen, así que la transcripción es todo el documento.",
  "share.includeNone": "Esta reunión no tiene transcripción que incluir.",
  "share.format": "Formato",
  "share.formatHint": "Se usa tanto para enviar como para guardar.",
  "share.formatPlaceholder": "Elige un formato…",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "Enviar a una app…",
  "share.sendToAppTitle": "Entregar el archivo a otra aplicación",
  "share.saveToDevice": "Guardar en este dispositivo…",
  "share.saveToDeviceTitle": "Guardar el archivo en este dispositivo",
  "share.gateHint": "Elige un formato arriba para enviar o guardar.",
  "share.nothingToShare":
    "Esta reunión todavía no tiene resumen ni transcripción para poner en un archivo.",
  "share.copyGroup": "Copiar al portapapeles",
  "share.copySummary": "Copiar el resumen",
  "share.copySummaryTitle": "Copiar el resumen de IA como Markdown",
  "share.copyTranscript": "Copiar la transcripción",
  "share.copyTranscriptTitle": "Copiar el texto sin formato de la transcripción",

  "toast.exportedMarkdown": "Archivo Markdown exportado.",
  "toast.exportedWord": "Documento de Word exportado.",
  "toast.exportedPdf": "PDF exportado.",
  "toast.copiedSummary": "Resumen copiado al portapapeles.",
  "toast.copiedTranscript": "Transcripción copiada al portapapeles.",
  "toast.meetingDeleted": "Reunión eliminada.",
  "toast.transcription": "Transcripción: {message}",
  "toast.audio": "Audio: {message}",
  "toast.serverNotSetUp":
    "El servidor de resúmenes de Minutes aún no está configurado. Define DESKSEC_TOKEN en .env o contacta con TI.",
  "toast.downloadModelFirst":
    "Descarga el modelo de transcripción «{model}» en los Ajustes antes de grabar.",
  "toast.configureOnline":
    "Configura la transcripción en línea en los Ajustes (token del servidor y DEEPGRAM_API_KEY en el servidor).",
  "toast.summarizeFailed": "No se pudo resumir esa reunión: {message}",

  "confirm.deleteTitle": "Eliminar la reunión",
  "confirm.deleteBody":
    "¿Eliminar {name} junto con su transcripción y su resumen? Esto no se puede deshacer.",
  "confirm.deleteThis": "esta reunión",

  "settingsLoading.label": "Cargando los ajustes",
  "settingsLoading.message": "Cargando los ajustes…",

  "prompt.dismiss": "Descartar",
  "prompt.callDetected": "{app} detectado",
  "prompt.newMeeting": "Nueva reunión",
  "prompt.callHeading": "¿Tomar notas de esta llamada?",
  "prompt.callSub": "Minutes captará la conversación y escribirá tus notas.",
  "prompt.manualHeading": "Iniciar una reunión",
  "prompt.manualSub": "Ponle nombre ahora, o déjalo y cámbialo más tarde.",
  "prompt.takeNotes": "Tomar notas",
  "prompt.startRecording": "Empezar a grabar",
  "prompt.notNow": "Ahora no",
  "prompt.meetingTitle": "Título de la reunión",
  "prompt.callPlaceholder": "Notas de {app}",
  "prompt.manualPlaceholder": "Reunión sin título",
  "prompt.hintStart": "iniciar",
  "prompt.hintClose": "cerrar",
  "prompt.errorHeading": "Aviso de reunión",
  "prompt.errorBody": "Algo salió mal al cargar este aviso.",
  "prompt.loadFailed":
    "No se pudo cargar el aviso de reunión. Ciérralo e inténtalo de nuevo.",
  "prompt.listening": "Escuchando",
  "prompt.call": "Llamada",

  "settings.title": "Ajustes",
  "settings.sectionsLabel": "Secciones de ajustes",
  "settings.applyImmediately": "Los cambios se aplican a medida que los haces",
  "settings.saving": "Guardando…",
  "settings.saved": "Guardado",
  "settings.server": "Servidor de resúmenes de Minutes",
  "settings.checking": "Comprobando…",
  "settings.unknown": "Desconocido",
  "settings.serverUnreachableConfigured":
    "Los resúmenes con IA necesitan una conexión que funcione. La transcripción sigue ejecutándose íntegramente en el dispositivo. Contacta con TI si esto persiste.",
  "settings.serverUnlinked":
    "Los resúmenes aún no están vinculados al servidor. La transcripción sigue funcionando sin conexión. Contacta con TI para configurarlo.",

  "settings.tab.appearance": "Apariencia",
  "settings.blurb.appearance": "Claro, oscuro o seguir al sistema.",
  "settings.tab.reading": "Comodidad de lectura",
  "settings.blurb.reading":
    "Tamaño del texto e interlineado de las transcripciones, guardados en este dispositivo.",
  "settings.tab.audio": "Audio",
  "settings.blurb.audio": "Elige qué capta Minutes mientras graba.",
  "settings.tab.callDetection": "Detección de llamadas",
  "settings.blurb.callDetection":
    "Ofrecer tomar notas cuando una app de llamadas use tu micrófono.",
  "settings.tab.transcription": "Transcripción",
  "settings.blurb.transcription":
    "Motor, modelo de precisión, hablantes e idioma hablado.",
  "settings.tab.summary": "Resumen",
  "settings.blurb.summary": "Cuándo se escriben los resúmenes con IA, y cómo.",
  "settings.tab.privacy": "Privacidad",
  "settings.blurb.privacy": "Qué sale de este dispositivo.",
  "settings.tab.advanced": "Avanzado",
  "settings.blurb.advanced":
    "Para TI y desarrollo. La mayoría puede dejar estos valores sin cambiar.",

  "settings.language": "Idioma",
  "settings.languageHint":
    "El idioma de las etiquetas y los mensajes de la app, guardado en este dispositivo. Los mensajes que vienen del servidor no se traducen.",

  "settings.textSize": "Tamaño del texto de la transcripción",
  "settings.textSizeHint": "Se aplica a la vista de transcripción.",
  "settings.sizeDefault": "Predeterminado",
  "settings.sizeLarge": "Grande",
  "settings.sizeXLarge": "Muy grande",
  "settings.lineSpacing": "Interlineado",
  "settings.spacingDefault": "Predeterminado",
  "settings.spacingRelaxed": "Amplio",
  "settings.spacingLoose": "Muy amplio",
  "settings.highContrast": "Texto de alto contraste",
  "settings.reduceMotion": "Reducir el movimiento",
  "settings.reduceMotionHint": "Menos animación en toda la app.",
  "settings.readingOnThisDevice":
    "Estas preferencias se guardan en este dispositivo.",

  "settings.captureMic": "Captar mi micrófono",
  "settings.microphone": "Micrófono",
  "settings.microphoneHint":
    "La grabación sigue al dispositivo: si unos auriculares Bluetooth se desconectan a mitad de la reunión, la captación continúa con el micrófono que tome el relevo.",
  "settings.systemDefault": "Predeterminado del sistema",
  "settings.captureSystemAudio": "Captar también el audio del sistema",
  "settings.captureSystemAudioHint":
    "Graba lo que oyes en Zoom, Meet, Teams y otras apps, sin necesidad de un bot de reunión. Mientras esto esté activado, se graba todo lo que suene en este dispositivo.",
  "settings.systemAudioSource": "Fuente de audio del sistema",
  "settings.defaultOutput": "Salida predeterminada",
  "settings.noCaptureSource":
    "Activa el micrófono, el audio del sistema o ambos: una grabación necesita algo que captar.",
  "settings.loopbackLinux":
    "No se encontró ningún monitor de audio del sistema. Con PipeWire o PulseAudio, busca una fuente llamada «Monitor of …» en tus ajustes de sonido y vuelve a abrir los Ajustes.",
  "settings.loopbackWindows":
    "No se encontró ninguna fuente de audio del sistema. Conecta altavoces o auriculares y vuelve a abrir los Ajustes. Stereo Mix o VB-Audio Cable también funcionan si aparecen en la lista.",
  "settings.loopbackMacos":
    "No se encontró ningún dispositivo de bucle. macOS necesita un controlador de audio virtual (p. ej. BlackHole). Instala uno y vuelve a abrir los Ajustes.",
  "settings.loopbackUnknown":
    "No se detectó ningún dispositivo de bucle para el audio del sistema. Se necesita una fuente de monitor o bucle para captar el audio de una reunión sin un bot.",

  "settings.callPrompt": "Avisar cuando una app de llamadas use el micrófono",
  "settings.callPromptHint":
    "Muestra una tarjeta flotante «Tomar notas» cuando Zoom, Teams (app o navegador), Google Meet, Slack, FaceTime, WhatsApp o Webex use el micrófono mientras Minutes está abierto. Meet/Teams en el navegador necesitan acceso de Automatización para Chrome/Safari en Ajustes del Sistema.",
  "settings.callCooldown": "Espera tras descartar",
  "settings.callCooldownHint": "Minutos de espera antes de volver a avisar.",
  "settings.callUnsupported":
    "La detección de llamadas está disponible en macOS. Aún puedes iniciar reuniones manualmente con «Nueva reunión».",

  "settings.engine": "Motor",
  "settings.engineWhisperHint":
    "El reconocimiento de voz se ejecuta localmente con un modelo Whisper. Tu audio nunca sale de este dispositivo para transcribirse.",
  "settings.engineCloudHint":
    "El audio se transmite en directo a tu servidor de Minutes (Deepgram Live) para obtener subtítulos con poca latencia. Usa la misma URL de servidor y el mismo token de acceso que los resúmenes con IA.",
  "settings.engineCloud": "En línea (servidor de Minutes · Deepgram)",
  "settings.engineWhisper": "En el dispositivo (Whisper)",
  "settings.statusLabel": "Estado",
  "settings.onlineReady": "La transcripción en línea está lista ({model}).",
  "settings.onlineNotConfigured":
    "Configura DESKSEC_TOKEN y asegúrate de que el servidor tenga DEEPGRAM_API_KEY.",
  "settings.accuracyModel": "Modelo de precisión",
  "settings.modelFiles": "Archivos del modelo",
  "settings.modelDownloading": "Descargando {label}…",
  "settings.modelReady": "El modelo «{model}» está descargado y listo.",
  "settings.modelMissing":
    "El modelo «{model}» todavía no está descargado: es necesario antes de grabar.",
  "settings.redownload": "Volver a descargar",
  "settings.downloadModel": "Descargar el modelo",
  "settings.downloadProgress": "Progreso de la descarga",
  "settings.downloadOnce":
    "El modelo {model} pesa unos {size}. Esto solo ocurre una vez: deja esta ventana abierta hasta que termine.",
  "settings.downloadedModels":
    "Modelos descargados ({size} en disco). Toca aquí para eliminar",
  "settings.downloadedModelsHint":
    "Elimina los modelos que ya no necesites. Usa «Descargar el modelo» arriba para volver a obtenerlos.",
  "settings.inUse": " · en uso",
  "settings.deleteQuestion": "¿Eliminar?",
  "settings.deleting": "Eliminando…",
  "settings.stopBeforeDeletingModels":
    "Detén la grabación antes de eliminar modelos.",
  "settings.identifySpeakers": "Identificar a los hablantes",
  "settings.identifySpeakersWhisper":
    "Indica quién habló en cada segmento. Descarga un modelo de hablantes pequeño la primera vez que se usa.",
  "settings.identifySpeakersCloud":
    "Indica quién habló en cada segmento usando la diarización en la nube, en el servidor.",
  "settings.spokenLanguage": "Idioma hablado",
  "settings.spokenLanguageHint":
    "El idioma que se habla en tus reuniones. La detección automática funciona en la mayoría de las grabaciones.",
  "settings.autoDetect": "Detección automática",

  "settings.autoSummarize": "Resumir las reuniones automáticamente",
  "settings.autoSummarizeHint":
    "Cuando una reunión termina, escribe su resumen sin que se lo pidas. Las reuniones de menos de un minuto se omiten. Si desactivas esto, una transcripción solo se envía al servidor de resúmenes cuando pulsas «Generar resumen» tú mismo.",
  "settings.summaryLanguage": "Idioma del resumen",
  "settings.summaryLanguageHint":
    "«Igual que la transcripción» mantiene el idioma propio de la reunión.",
  "settings.matchTranscript": "Igual que la transcripción",
  "settings.summaryInstructions": "Instrucciones del resumen (opcional)",
  "settings.summaryInstructionsHint":
    "Se aplican a cada resumen que generes. Déjalo en blanco para el comportamiento predeterminado. También puedes añadir instrucciones por reunión antes de generar un resumen.",

  "settings.telemetry": "Compartir estadísticas de uso anónimas",
  "settings.telemetryHint":
    "Nos ayuda a ver qué funciones se usan, con qué rapidez funcionan y qué errores ocurren.",
  "settings.telemetryDetail":
    "Qué se envía: recuentos de uso de funciones, rangos de duración, categorías de error, versión de la app, sistema operativo y versión, tipo de CPU y número de núcleos, y un identificador de instalación aleatorio que puedes restablecer. Qué nunca se envía: tus grabaciones, transcripciones, resúmenes, títulos de reunión, nombres de participantes, rutas de archivos ni nada de lo que escribas o digas. Si la app está sin conexión, los informes esperan en un archivo pequeño en este dispositivo y se envían más tarde. Los informes se conservan 12 meses. Desactivar esto detiene de inmediato todo envío, elimina lo que aún esté esperando en este dispositivo y elimina el identificador de instalación.",

  "settings.serverUrl": "URL del servidor",
  "settings.serverUrlLocked":
    "Bloqueado: configurado en la compilación por CI ({url}).",
  "settings.serverUrlEmbedded": "incorporado",
  "settings.serverUrlHint":
    "Los servidores remotos deben usar https://; http:// solo funciona para localhost.",
  "settings.accessToken": "Token de acceso",
  "settings.tokenFromBuild":
    "Configurado en la compilación por CI y guardado en el llavero del sistema.",
  "settings.tokenFromEnv": "Definido desde DESKSEC_TOKEN en .env.",
  "settings.tokenInKeychain": "Guardado en el llavero del sistema.",
  "settings.tokenMissing":
    "Define DESKSEC_TOKEN en .env (consulta .env.example) para los resúmenes con IA.",
  "settings.deviceId": "ID del dispositivo",
  "settings.deviceIdHint":
    "Identifica esta instalación ante el servidor. Indíquelo a TI para solicitar que se revoque el acceso de este dispositivo.",
  "settings.summaryModel": "Modelo de resumen",
  "settings.chunkLength": "Longitud del bloque",
  "settings.chunkLengthHint":
    "Segundos. En cada bloque se producen segmentos definitivos de transcripción.",
  "settings.partialInterval": "Intervalo provisional",
  "settings.partialIntervalHint":
    "Segundos, 0 = desactivado. El texto provisional se actualiza en este intervalo. Ambos se ejecutan en el dispositivo.",
  "settings.exportMarkdown": "Exportar las reuniones terminadas a ~/meetings",
  "settings.exportMarkdownHint":
    "Refleja cada reunión finalizada como markdown para que la CLI de Minutes incluida, las herramientas MCP y el grafo de relaciones puedan leerla.",

  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "La transcripción aparece a medida que se habla.",
  "settings.connectionCheckFailed": "No se pudo comprobar la conexión",
  "serverUrl.enterFull": "Introduce una URL completa, p. ej. https://minutes.example.com o http://localhost:8787.",
  "serverUrl.onlyHttp": "Solo se admiten URL http:// y https://.",
  "serverUrl.httpsRequired": "Los servidores remotos deben usar https://; con http:// simple tu token y tu transcripción se enviarían sin cifrar. (http:// solo se permite para localhost.)",
  "model.tiny": "Tiny — el más rápido, menor precisión",
  "model.base": "Base — rápido",
  "model.small": "Small — recomendado",
  "model.medium": "Medium — más preciso, más lento",
  "model.largeV3": "Large v3 — mejor precisión, el más lento",
  "model.unknownSize": "tamaño desconocido",
  "error.unknown": "Se ha producido un error desconocido.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Documento de Word",
  "dialog.pdf": "PDF",

  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Idioma y región",
  "settings.blurb.languageRegion": "El idioma de la app y cómo se muestran las fechas.",
  "settings.dateFormat": "Fechas y horas",
  "settings.dateFormatHint": "Se muestran con el formato regional de este dispositivo, tomado del sistema operativo. Cámbialo en los ajustes del sistema.",

  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Conectado",
  "settings.notConfigured": "Sin configurar",
  "settings.unreachable": "No accesible",

  "model.vad": "Detección de actividad de voz",
  "model.diarization": "Identificación de hablantes",

  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "Esa reunión se está grabando: deténla antes de eliminarla.",
  "error.noTranscriptCheckLanguage":
    "El audio llega al servidor, pero no se está transcribiendo nada. Compruebe el idioma de transcripción en Ajustes: debe coincidir con el idioma que se habla.",
  "error.meetingNotFound": "No se ha encontrado esa reunión.",
  "error.nothingToShare": "Todavía no hay nada que compartir de esta reunión.",
  "error.shareUnsupported": "Compartir con otra app no está disponible en esta plataforma: guarda el archivo en su lugar.",
  "error.noWindowToShare": "La ventana principal no está disponible para compartir.",
  "error.stopBeforeEngineChange": "Detén la grabación antes de cambiar el motor de transcripción.",
  "error.noCaptureSource": "Activa el micrófono, el audio del sistema o ambos.",
  "error.exportPathNotAbsolute": "No se ha podido usar esa ubicación de guardado.",
  "error.exportExtension": "Minutes no puede escribir ese tipo de archivo.",
  "error.stopBeforeDeletingModels": "Detén la grabación antes de eliminar modelos.",

  "error.serverTokenMissing": "El token de acceso al servidor de Minutes no está configurado. Revisa los Ajustes o contacta con TI.",
  "error.serverRejectedToken": "El servidor de Minutes ha rechazado el token de acceso. Revisa los Ajustes o contacta con TI.",
  "error.onlineNotConfiguredOnServer": "La transcripción en línea no está configurada en el servidor de Minutes. Contacta con TI.",
  "error.unknownBrowser": "Minutes no puede detectar reuniones en ese navegador.",
  "error.noPrivacyPane": "Este sistema no tiene una página de ajustes para ese permiso.",

  /* ---------------- Configuración inicial ---------------- */
  "onboarding.stepOf": "Paso {current} de {total}",
  "onboarding.skipAll": "Omitir la configuración",
  "onboarding.back": "Atrás",
  "onboarding.continue": "Continuar",
  "onboarding.skipStep": "Ahora no",
  "onboarding.openSettings": "Abrir Ajustes del Sistema",
  "onboarding.allowed": "Permitido",
  "onboarding.notAllowed": "No permitido",
  "onboarding.notSetUp": "Sin configurar",
  "onboarding.checking": "Comprobando…",

  "onboarding.welcomeTitle": "Bienvenido a Minutes",
  "onboarding.welcomeBody":
    "Minutes graba tus reuniones y redacta el acta. Antes de empezar, necesita permiso para un par de cosas.",
  "onboarding.welcomeOptional": "Cada paso es opcional y podrás cambiarlo más adelante en los ajustes.",
  "onboarding.getStarted": "Empezar",

  "onboarding.microphoneTitle": "Micrófono",
  "onboarding.microphoneBody":
    "Minutes graba el audio de la reunión desde tu micrófono. No se graba nada hasta que inicias una reunión.",
  "onboarding.microphoneAllow": "Permitir el micrófono",
  "onboarding.microphoneDeniedHint":
    "Se denegó el acceso al micrófono y macOS solo lo pregunta una vez. Puedes activarlo en «Privacidad y seguridad» → «Micrófono».",
  "onboarding.microphoneWindowsHint":
    "Windows no pide este permiso a las aplicaciones. Si la grabación no capta nada, comprueba que el acceso al micrófono esté activado para aplicaciones de escritorio en «Privacidad y seguridad» → «Micrófono».",

  "onboarding.browserTitle": "Reuniones abiertas en el navegador",
  "onboarding.browserBody":
    "Para ofrecerte tomar notas cuando te unes a una reunión de Google Meet o Teams desde un enlace, Minutes comprueba si hay una reunión abierta en tu navegador.",
  "onboarding.browserPrivacy":
    "Solo mira si una pestaña es una reunión, no el contenido de las páginas, y nada sale de tu dispositivo.",
  "onboarding.browserPerApp":
    "macOS concede este permiso navegador por navegador, por eso aparecen por separado. Solo se muestran los navegadores que tienes instalados.",
  "onboarding.browserAllow": "Permitir",
  "onboarding.browserNone":
    "No se encontró ningún navegador compatible, así que aquí no hay nada que configurar. Las reuniones en Zoom, Teams y Slack se detectan sin esto.",
  "onboarding.browserDeniedHint":
    "macOS solo pregunta una vez por navegador. Para cambiarlo, ve a «Privacidad y seguridad» → «Automatización» y marca Minutes bajo ese navegador.",


  "onboarding.detectionUnavailableTitle": "Iniciar una reunión",
  "onboarding.detectionUnavailableBody":
    "La detección automática de reuniones solo está disponible en macOS por ahora. En este sistema, inicia tú la grabación con «Nueva reunión» cuando quieras que se tomen notas.",

  "onboarding.doneTitle": "Todo listo",
  "onboarding.doneBody": "Este es el estado actual. Puedes volver a todo esto en los ajustes.",
  "onboarding.doneSkipped": "Omitido: puedes configurarlo más adelante en los ajustes.",
  "onboarding.finish": "Empezar a usar Minutes",

  "settings.rerunOnboarding": "Permisos y configuración",
  "settings.rerunOnboardingHint":
    "Repasar de nuevo la configuración del micrófono y la detección en el navegador.",
  "settings.rerunOnboardingAction": "Iniciar la configuración",
};
