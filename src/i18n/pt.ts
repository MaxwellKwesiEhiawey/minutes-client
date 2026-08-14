import type { Translations } from "./index";

/**
 * Portuguese (European spelling, understandable in Brazil). Impersonal phrasing
 * where possible, which avoids choosing between "tu", "você" and "o utilizador".
 *
 * The privacy strings say exactly what the English says: `share.includeOff` must
 * make clear nothing spoken verbatim is in the file, and
 * `settings.telemetryDetail` must not soften what is and is not transmitted.
 */
export const pt: Translations = {
  "common.close": "Fechar",
  "common.cancel": "Cancelar",
  "common.delete": "Eliminar",
  "common.done": "Concluído",
  "common.open": "Abrir",
  "common.tryAgain": "Tentar novamente",
  "common.retrying": "A tentar novamente…",
  "common.loading": "A carregar…",
  "common.yes": "Sim",
  "common.none": "—",
  "common.starting": "A iniciar…",

  "nav.home": "Início",
  "nav.myNotes": "As minhas notas",
  "nav.settings": "Definições",
  "nav.newMeeting": "Nova reunião",
  "nav.brandHome": "Início do Minutes",
  "nav.main": "Navegação principal",

  "topbar.toggleSidebar": "Mostrar ou ocultar a barra lateral",
  "topbar.search": "Pesquisar reuniões e transcrições…",
  "topbar.searchLabel": "Pesquisar reuniões e transcrições",
  "topbar.themeTitle": "Tema: {theme} — clique para mudar",
  "topbar.recordingOpen": "Ir para a reunião em gravação",
  "topbar.recordingStop": "Terminar a reunião em gravação",
  "topbar.stop": "Parar",
  "theme.light": "Claro",
  "theme.dark": "Escuro",
  "theme.system": "Sistema",

  "page.home": "Início",
  "page.notes": "As minhas notas",
  "page.settings": "Definições",
  "page.recording": "Gravação",
  "page.meeting": "Reunião",

  "home.greeting": "Olá, bem-vindo!",
  "home.sub": "Pronto para transformar a próxima conversa em algo útil?",
  "home.recent": "Reuniões recentes",
  "home.viewAll": "Ver todas as notas →",
  "home.summaryReady": "Resumo pronto",
  "home.transcriptOnly": "Apenas transcrição",
  "home.emptyTitle": "As suas reuniões aparecerão aqui",
  "home.emptyBody":
    "Inicie a sua primeira reunião e o Minutes capta a conversa, gera um resumo e organiza tudo para si.",
  "home.emptyCta": "Iniciar uma reunião",

  "notes.title": "As minhas notas",
  "notes.sub": "Todas as reuniões que o Minutes captou neste dispositivo.",
  "notes.results": "Resultados para «{query}»",
  "notes.clearSearchText": "Limpar o texto de pesquisa",
  "notes.colMeeting": "Reunião",
  "notes.colDate": "Data",
  "notes.colDuration": "Duração",
  "notes.colSummary": "Resumo",
  "notes.colStatus": "Estado",
  "notes.moreActions": "Mais ações para {title}",
  "notes.stopBeforeDelete": "Pare a gravação antes de eliminar",
  "notes.emptySearchTitle": "Nenhuma reunião corresponde à pesquisa",
  "notes.emptySearchBody":
    "Tente outra palavra ou expressão: a pesquisa abrange títulos e o texto das transcrições.",
  "notes.clearSearch": "Limpar a pesquisa",
  "notes.emptyTitle": "Ainda não há reuniões",
  "notes.emptyBody":
    "Inicie uma reunião e ela aparecerá aqui com a transcrição e o resumo.",

  "status.recording": "A gravar",
  "status.completed": "Concluída",
  "status.interrupted": "Interrompida",

  "detail.back": "Voltar a As minhas notas",
  "detail.share": "Partilhar e exportar",
  "detail.delete": "Eliminar a reunião",
  "detail.tabSummary": "Resumo",
  "detail.tabTranscript": "Transcrição",
  "detail.tabsLabel": "Painéis da reunião",
  "detail.generate": "Gerar resumo",
  "detail.regenerate": "Gerar novamente o resumo",
  "detail.summarizing": "A resumir…",
  "detail.generateTitle": "Gerar um resumo com IA",
  "detail.generateDisabled":
    "Ainda não foi captada qualquer transcrição desta reunião",
  "detail.instructionsToggle": "Adicionar instruções",
  "detail.instructionsLabel": "Instruções para este resumo (opcional)",
  "detail.instructionsPlaceholder":
    "por ex. Não incluir os nomes das pessoas mencionadas na reunião.",
  "detail.instructionsCombined":
    "Combinadas com as suas instruções de resumo predefinidas nas Definições.",
  "detail.instructionsApplied":
    "Aplicadas quando gera ou gera novamente o resumo.",
  "detail.writingSummary":
    "A escrever o seu resumo — normalmente demora cerca de um minuto.",
  "detail.noSummaryTitle": "Ainda não há resumo",
  "detail.noSummaryReady":
    "Gere um a partir da transcrição quando quiser.",
  "detail.noSummaryNoTranscript":
    "Um resumo precisa de uma transcrição — nada foi captado nesta reunião.",
  "detail.noTranscriptTitle": "Nenhuma transcrição captada",
  "detail.noTranscriptBody":
    "Esta reunião não tem segmentos de transcrição.",
  "detail.speaker": "Interveniente",

  "summaryError.networkTitle":
    "Não foi possível contactar o servidor de resumos.",
  "summaryError.networkHint":
    "Verifique a ligação de rede e o URL do servidor nas Definições e tente novamente.",
  "summaryError.timeoutTitle":
    "O servidor de resumos demorou demasiado a responder.",
  "summaryError.timeoutHint":
    "Pode acontecer com uma ligação lenta ou uma transcrição muito longa. Tente novamente.",
  "summaryError.authTitle":
    "O servidor de resumos recusou o pedido (não autorizado).",
  "summaryError.authHint":
    "O seu token de acesso do Minutes pode estar em falta ou ser inválido — verifique as Definições ou contacte a equipa de TI.",
  "summaryError.serverTitle": "O servidor de resumos devolveu um erro.",
  "summaryError.genericTitle": "Não foi possível gerar um resumo.",

  "summary.aiNote":
    "Gerado por IA a partir da transcrição · reveja antes de partilhar",
  "summary.overview": "Visão geral",
  "summary.keyPoints": "Pontos principais da discussão",
  "summary.decisions": "Decisões",
  "summary.actionItems": "Tarefas",
  "summary.openQuestions": "Questões em aberto",
  "summary.openQuestion": "Questão em aberto",
  "summary.owner": "responsável: {name}",
  "summary.assignedTo": "Atribuído a: {name}",
  "summary.due": "Prazo: {date}",
  "summary.generatedBy": "Gerado por {model} · {date}",

  "recording.back": "Voltar a As minhas notas",
  "recording.transcriptSaved": "A transcrição é guardada em direto",
  "recording.endMeeting": "Terminar a reunião",
  "recording.inputLevel": "Nível de entrada",
  "recording.liveTranscript": "Transcrição em direto",
  "recording.savedAsCaptured": "Guardada à medida que é captada",
  "recording.nothingYet": "Ainda nada captado",
  "recording.listening":
    "A ouvir — a transcrição aparece à medida que se fala.",
  "recording.interim": "Transcrição provisória",
  "recording.live": "Em direto",

  "engine.onDevice": "Privado · neste dispositivo",
  "engine.onDeviceTitle":
    "A transcrição é executada neste dispositivo (modelo Whisper: {model})",
  "engine.cloud": "Transcrição na nuvem",
  "engine.cloudTitle": "A transcrição é executada online (Deepgram)",

  "palette.label": "Pesquisa",
  "palette.placeholder": "Pesquisar reuniões e transcrições…",
  "palette.recent": "Recentes",
  "palette.meetings": "Reuniões",
  "palette.transcripts": "Transcrições",
  "palette.noResults": "Sem resultados para «{query}»",
  "palette.noResultsHint":
    "Tente o nome de um interveniente ou uma expressão da conversa.",
  "palette.nothingYet": "Ainda não há nada para pesquisar",
  "palette.nothingYetHint":
    "Grave uma reunião e ela passa a ser pesquisável aqui.",

  "share.title": "Partilhar e exportar",
  "share.includeTranscript": "Incluir a transcrição completa",
  "share.includeOn":
    "O ficheiro conterá o resumo e tudo o que foi dito.",
  "share.includeOff":
    "O ficheiro conterá apenas o resumo — nada do que foi dito literalmente.",
  "share.includeForced":
    "Ainda não existe resumo, por isso a transcrição é o documento completo.",
  "share.includeNone": "Esta reunião não tem transcrição para incluir.",
  "share.format": "Formato",
  "share.formatHint": "Usado tanto para enviar como para guardar.",
  "share.formatPlaceholder": "Escolha um formato…",
  "share.formatPdf": "PDF (.pdf)",
  "share.formatDocx": "Word (.docx)",
  "share.formatMd": "Markdown (.md)",
  "share.sendToApp": "Enviar para uma app…",
  "share.sendToAppTitle": "Entregar o ficheiro a outra aplicação",
  "share.saveToDevice": "Guardar neste dispositivo…",
  "share.saveToDeviceTitle": "Guardar o ficheiro neste dispositivo",
  "share.gateHint": "Escolha um formato acima para enviar ou guardar.",
  "share.nothingToShare":
    "Esta reunião ainda não tem resumo nem transcrição para colocar num ficheiro.",
  "share.copyGroup": "Copiar para a área de transferência",
  "share.copySummary": "Copiar o resumo",
  "share.copySummaryTitle": "Copiar o resumo de IA em Markdown",
  "share.copyTranscript": "Copiar a transcrição",
  "share.copyTranscriptTitle": "Copiar o texto simples da transcrição",

  "toast.exportedMarkdown": "Ficheiro Markdown exportado.",
  "toast.exportedWord": "Documento Word exportado.",
  "toast.exportedPdf": "PDF exportado.",
  "toast.copiedSummary": "Resumo copiado para a área de transferência.",
  "toast.copiedTranscript": "Transcrição copiada para a área de transferência.",
  "toast.meetingDeleted": "Reunião eliminada.",
  "toast.transcription": "Transcrição: {message}",
  "toast.audio": "Áudio: {message}",
  "toast.serverNotSetUp":
    "O servidor de resumos do Minutes ainda não está configurado. Defina DESKSEC_TOKEN no .env ou contacte a equipa de TI.",
  "toast.downloadModelFirst":
    "Descarregue o modelo de transcrição «{model}» nas Definições antes de gravar.",
  "toast.configureOnline":
    "Configure a transcrição online nas Definições (token do servidor e DEEPGRAM_API_KEY no servidor).",
  "toast.summarizeFailed": "Não foi possível resumir essa reunião: {message}",

  "confirm.deleteTitle": "Eliminar a reunião",
  "confirm.deleteBody":
    "Eliminar {name} e a respetiva transcrição e resumo? Não é possível anular.",
  "confirm.deleteThis": "esta reunião",

  "settingsLoading.label": "A carregar as definições",
  "settingsLoading.message": "A carregar as definições…",

  "prompt.dismiss": "Ignorar",
  "prompt.callDetected": "{app} detetado",
  "prompt.newMeeting": "Nova reunião",
  "prompt.callHeading": "Tirar notas desta chamada?",
  "prompt.callSub": "O Minutes capta a conversa e escreve as suas notas.",
  "prompt.manualHeading": "Iniciar uma reunião",
  "prompt.manualSub":
    "Dê-lhe um nome agora, ou deixe e mude o nome mais tarde.",
  "prompt.takeNotes": "Tirar notas",
  "prompt.startRecording": "Começar a gravar",
  "prompt.notNow": "Agora não",
  "prompt.meetingTitle": "Título da reunião",
  "prompt.callPlaceholder": "Notas de {app}",
  "prompt.manualPlaceholder": "Reunião sem título",
  "prompt.hintStart": "iniciar",
  "prompt.hintClose": "fechar",
  "prompt.errorHeading": "Aviso de reunião",
  "prompt.errorBody": "Algo falhou ao carregar este aviso.",
  "prompt.loadFailed":
    "Não foi possível carregar o aviso de reunião. Feche e tente novamente.",
  "prompt.listening": "A ouvir",
  "prompt.call": "Chamada",

  "settings.title": "Definições",
  "settings.sectionsLabel": "Secções das definições",
  "settings.applyImmediately": "As alterações aplicam-se à medida que as faz",
  "settings.saving": "A guardar…",
  "settings.saved": "Guardado",
  "settings.server": "Servidor de resumos do Minutes",
  "settings.checking": "A verificar…",
  "settings.unknown": "Desconhecido",
  "settings.serverUnreachableConfigured":
    "Os resumos com IA precisam de uma ligação a funcionar. A transcrição continua a ser executada inteiramente no dispositivo. Contacte a equipa de TI se isto persistir.",
  "settings.serverUnlinked":
    "Os resumos ainda não estão ligados ao servidor. A transcrição continua a funcionar offline. Contacte a equipa de TI para configurar.",

  "settings.tab.appearance": "Aspeto",
  "settings.blurb.appearance": "Claro, escuro ou seguir o sistema.",
  "settings.tab.reading": "Conforto de leitura",
  "settings.blurb.reading":
    "Tamanho do texto e espaçamento das transcrições, guardados neste dispositivo.",
  "settings.tab.audio": "Áudio",
  "settings.blurb.audio": "Escolha o que o Minutes capta durante a gravação.",
  "settings.tab.callDetection": "Deteção de chamadas",
  "settings.blurb.callDetection":
    "Propor tirar notas quando uma app de chamadas usar o microfone.",
  "settings.tab.transcription": "Transcrição",
  "settings.blurb.transcription":
    "Motor, modelo de precisão, intervenientes e idioma falado.",
  "settings.tab.summary": "Resumo",
  "settings.blurb.summary": "Quando os resumos com IA são escritos, e como.",
  "settings.tab.privacy": "Privacidade",
  "settings.blurb.privacy": "O que sai deste dispositivo.",
  "settings.tab.advanced": "Avançado",
  "settings.blurb.advanced":
    "Para TI e desenvolvimento. A maioria das pessoas pode deixar isto inalterado.",

  "settings.language": "Idioma",
  "settings.languageHint":
    "O idioma das etiquetas e mensagens da aplicação, guardado neste dispositivo. As mensagens que vêm do servidor não são traduzidas.",

  "settings.textSize": "Tamanho do texto da transcrição",
  "settings.textSizeHint": "Aplica-se à vista de transcrição.",
  "settings.sizeDefault": "Predefinido",
  "settings.sizeLarge": "Grande",
  "settings.sizeXLarge": "Muito grande",
  "settings.lineSpacing": "Espaçamento entre linhas",
  "settings.spacingDefault": "Predefinido",
  "settings.spacingRelaxed": "Amplo",
  "settings.spacingLoose": "Muito amplo",
  "settings.highContrast": "Texto de alto contraste",
  "settings.reduceMotion": "Reduzir a animação",
  "settings.reduceMotionHint": "Menos animação em toda a aplicação.",
  "settings.readingOnThisDevice":
    "Estas preferências são guardadas neste dispositivo.",

  "settings.captureMic": "Captar o meu microfone",
  "settings.microphone": "Microfone",
  "settings.microphoneHint":
    "A gravação segue o dispositivo: se um auricular Bluetooth se desligar a meio da reunião, a captação continua no microfone que assumir.",
  "settings.systemDefault": "Predefinição do sistema",
  "settings.captureSystemAudio": "Captar também o áudio do sistema",
  "settings.captureSystemAudioHint":
    "Grava o que ouve no Zoom, Meet, Teams e outras apps — sem necessidade de um bot de reunião. Enquanto isto estiver ativo, tudo o que toca neste dispositivo é gravado.",
  "settings.systemAudioSource": "Fonte de áudio do sistema",
  "settings.defaultOutput": "Saída predefinida",
  "settings.noCaptureSource":
    "Ative o microfone, o áudio do sistema, ou ambos — uma gravação precisa de algo para captar.",
  "settings.loopbackLinux":
    "Não foi encontrado nenhum monitor de áudio do sistema. Com o PipeWire ou o PulseAudio, procure uma fonte chamada «Monitor of …» nas definições de som e reabra as Definições.",
  "settings.loopbackWindows":
    "Não foi encontrada nenhuma fonte de áudio do sistema. Ligue altifalantes ou auscultadores e reabra as Definições. O Stereo Mix ou o VB-Audio Cable também funcionam, se estiverem listados.",
  "settings.loopbackMacos":
    "Não foi encontrado nenhum dispositivo de loopback. O macOS precisa de um controlador de áudio virtual (por ex. BlackHole). Instale um e reabra as Definições.",
  "settings.loopbackUnknown":
    "Não foi detetado nenhum dispositivo de loopback para o áudio do sistema. É necessária uma fonte de monitor/loopback para captar o áudio de uma reunião sem um bot.",

  "settings.callPrompt":
    "Avisar quando uma app de chamadas usar o microfone",
  "settings.callPromptHint":
    "Mostra um cartão flutuante «Tirar notas» quando o Zoom, o Teams (app ou navegador), o Google Meet, o Slack, o FaceTime, o WhatsApp ou o Webex usar o microfone enquanto o Minutes estiver aberto. O Meet/Teams no navegador precisa de acesso de Automatização para o Chrome/Safari nas Definições do Sistema.",
  "settings.callCooldown": "Espera após ignorar",
  "settings.callCooldownHint":
    "Minutos a aguardar antes de avisar novamente.",
  "settings.callUnsupported":
    "A deteção de chamadas está disponível no macOS. Pode continuar a iniciar reuniões manualmente com «Nova reunião».",

  "settings.engine": "Motor",
  "settings.engineWhisperHint":
    "O reconhecimento de voz é executado localmente com um modelo Whisper. O seu áudio nunca sai deste dispositivo para ser transcrito.",
  "settings.engineCloudHint":
    "O áudio é transmitido em direto para o seu servidor Minutes (Deepgram Live) para legendas com baixa latência. Usa o mesmo URL de servidor e o mesmo token de acesso dos resumos com IA.",
  "settings.engineCloud": "Online (servidor Minutes · Deepgram)",
  "settings.engineWhisper": "No dispositivo (Whisper)",
  "settings.statusLabel": "Estado",
  "settings.onlineReady": "A transcrição online está pronta ({model}).",
  "settings.onlineNotConfigured":
    "Configure DESKSEC_TOKEN e garanta que o servidor tem DEEPGRAM_API_KEY.",
  "settings.accuracyModel": "Modelo de precisão",
  "settings.modelFiles": "Ficheiros do modelo",
  "settings.modelDownloading": "A descarregar {label}…",
  "settings.modelReady": "O modelo «{model}» está descarregado e pronto.",
  "settings.modelMissing":
    "O modelo «{model}» ainda não foi descarregado — é necessário antes de gravar.",
  "settings.redownload": "Descarregar novamente",
  "settings.downloadModel": "Descarregar o modelo",
  "settings.downloadProgress": "Progresso do descarregamento",
  "settings.downloadOnce":
    "O modelo {model} tem cerca de {size}. Isto acontece uma única vez — mantenha esta janela aberta até terminar.",
  "settings.downloadedModels":
    "Modelos descarregados ({size} em disco). Toque aqui para eliminar",
  "settings.downloadedModelsHint":
    "Remova os modelos de que já não precisa. Use «Descarregar o modelo» acima para os obter novamente.",
  "settings.inUse": " · em uso",
  "settings.deleteQuestion": "Eliminar?",
  "settings.deleting": "A eliminar…",
  "settings.stopBeforeDeletingModels":
    "Pare a gravação antes de eliminar modelos.",
  "settings.identifySpeakers": "Identificar os intervenientes",
  "settings.identifySpeakersWhisper":
    "Indica quem falou em cada segmento. Descarrega um pequeno modelo de intervenientes na primeira utilização.",
  "settings.identifySpeakersCloud":
    "Indica quem falou em cada segmento através da diarização na nuvem, no servidor.",
  "settings.spokenLanguage": "Idioma falado",
  "settings.spokenLanguageHint":
    "O idioma falado nas suas reuniões. A deteção automática funciona na maioria das gravações.",
  "settings.autoDetect": "Deteção automática",

  "settings.autoSummarize": "Resumir as reuniões automaticamente",
  "settings.autoSummarizeHint":
    "Quando uma reunião termina, escreve o resumo sem ser pedido. As reuniões com menos de um minuto são ignoradas. Se desativar isto, uma transcrição só é enviada para o servidor de resumos quando premir «Gerar resumo».",
  "settings.summaryLanguage": "Idioma do resumo",
  "settings.summaryLanguageHint":
    "«Igual à transcrição» mantém o idioma da própria reunião.",
  "settings.matchTranscript": "Igual à transcrição",
  "settings.summaryInstructions": "Instruções de resumo (opcional)",
  "settings.summaryInstructionsHint":
    "Aplicam-se a cada resumo que gerar. Deixe em branco para o comportamento predefinido. Também pode adicionar instruções por reunião antes de gerar um resumo.",

  "settings.telemetry": "Partilhar estatísticas de utilização anónimas",
  "settings.telemetryHint":
    "Ajuda-nos a ver que funcionalidades são usadas, com que rapidez funcionam e que erros ocorrem.",
  "settings.telemetryDetail":
    "O que é enviado: contagens de utilização de funcionalidades, intervalos de duração, categorias de erro, versão da aplicação, sistema operativo e versão, tipo de CPU e número de núcleos, e um identificador de instalação aleatório que pode reiniciar. O que nunca é enviado: as suas gravações, transcrições, resumos, títulos de reuniões, nomes de participantes, caminhos de ficheiros, nem nada do que escreve ou diz. Se a aplicação estiver offline, os relatórios aguardam num pequeno ficheiro neste dispositivo e são enviados mais tarde. Os relatórios são guardados durante 12 meses. Desativar isto interrompe imediatamente qualquer envio, elimina o que ainda estiver a aguardar neste dispositivo e elimina o identificador de instalação.",

  "settings.serverUrl": "URL do servidor",
  "settings.serverUrlLocked":
    "Bloqueado — configurado na compilação pela CI ({url}).",
  "settings.serverUrlEmbedded": "incorporado",
  "settings.serverUrlHint":
    "Os servidores remotos têm de usar https:// — http:// só funciona para localhost.",
  "settings.accessToken": "Token de acesso",
  "settings.tokenFromBuild":
    "Configurado na compilação pela CI e guardado no porta-chaves do sistema.",
  "settings.tokenFromEnv": "Definido a partir de DESKSEC_TOKEN no .env.",
  "settings.tokenInKeychain": "Guardado no porta-chaves do sistema.",
  "settings.tokenMissing":
    "Defina DESKSEC_TOKEN no .env (ver .env.example) para os resumos com IA.",
  "settings.summaryModel": "Modelo de resumo",
  "settings.chunkLength": "Duração do bloco",
  "settings.chunkLengthHint":
    "Segundos. Em cada bloco são produzidos segmentos definitivos de transcrição.",
  "settings.partialInterval": "Intervalo provisório",
  "settings.partialIntervalHint":
    "Segundos, 0 = desligado. O texto provisório é atualizado neste intervalo. Ambos são executados no dispositivo.",
  "settings.exportMarkdown": "Exportar as reuniões terminadas para ~/meetings",
  "settings.exportMarkdownHint":
    "Replica cada reunião concluída em markdown, para que a CLI Minutes incluída, as ferramentas MCP e o grafo de relações a possam ler.",

  /* ---------------- Outside the components ---------------- */
  "recording.appearsWhenSpoken": "A transcrição aparece à medida que se fala.",
  "settings.connectionCheckFailed": "Não foi possível verificar a ligação",
  "serverUrl.enterFull": "Introduza um URL completo, por ex. https://minutes.example.com ou http://localhost:8787.",
  "serverUrl.onlyHttp": "Só são suportados URL http:// e https://.",
  "serverUrl.httpsRequired": "Os servidores remotos têm de usar https:// — com http:// simples o seu token e a sua transcrição seriam enviados em texto simples. (http:// só é permitido para localhost.)",
  "model.tiny": "Tiny — o mais rápido, menor precisão",
  "model.base": "Base — rápido",
  "model.small": "Small — recomendado",
  "model.medium": "Medium — mais preciso, mais lento",
  "model.largeV3": "Large v3 — melhor precisão, o mais lento",
  "model.unknownSize": "tamanho desconhecido",
  "error.unknown": "Ocorreu um erro desconhecido.",
  "dialog.markdown": "Markdown",
  "dialog.word": "Documento Word",
  "dialog.pdf": "PDF",

  /* ---------------- Language & Region ---------------- */
  "settings.tab.languageRegion": "Idioma e região",
  "settings.blurb.languageRegion": "O idioma da aplicação e a forma como as datas são mostradas.",
  "settings.dateFormat": "Datas e horas",
  "settings.dateFormatHint": "São mostradas no formato regional deste dispositivo, obtido do sistema operativo. Altere-o nas definições do sistema.",

  /* ---------------- Server connection state ---------------- */
  "settings.connected": "Ligado",
  "settings.notConfigured": "Não configurado",
  "settings.unreachable": "Inacessível",

  "model.vad": "Deteção de atividade de voz",
  "model.diarization": "Identificação de intervenientes",

  /* ---------------- Errors the backend asks us to show ---------------- */
  "error.deleteWhileRecording": "Essa reunião está a ser gravada — pare-a antes de a eliminar.",
  "error.meetingNotFound": "Essa reunião não foi encontrada.",
  "error.nothingToShare": "Ainda não há nada para partilhar nesta reunião.",
  "error.shareUnsupported": "A partilha para outra aplicação não está disponível nesta plataforma — guarde o ficheiro em vez disso.",
  "error.noWindowToShare": "A janela principal não está disponível para partilhar.",
  "error.stopBeforeEngineChange": "Pare a gravação antes de mudar o motor de transcrição.",
  "error.noCaptureSource": "Ative o microfone, o áudio do sistema, ou ambos.",
  "error.exportPathNotAbsolute": "Não foi possível usar esse local de gravação.",
  "error.exportExtension": "O Minutes não consegue escrever esse tipo de ficheiro.",
  "error.stopBeforeDeletingModels": "Pare a gravação antes de eliminar modelos.",

  "error.serverTokenMissing": "O token de acesso ao servidor Minutes não está configurado. Verifique as Definições ou contacte a equipa de TI.",
  "error.serverRejectedToken": "O servidor Minutes recusou o token de acesso. Verifique as Definições ou contacte a equipa de TI.",
  "error.onlineNotConfiguredOnServer": "A transcrição online não está configurada no servidor Minutes. Contacte a equipa de TI.",
  "error.unknownBrowser": "O Minutes não consegue detetar reuniões nesse navegador.",
  "error.noPrivacyPane": "Este sistema não tem uma página de configurações para essa permissão.",

  /* ---------------- Configuração inicial ---------------- */
  "onboarding.stepOf": "Passo {current} de {total}",
  "onboarding.skipAll": "Ignorar a configuração",
  "onboarding.back": "Voltar",
  "onboarding.continue": "Continuar",
  "onboarding.skipStep": "Agora não",
  "onboarding.openSettings": "Abrir as Definições do Sistema",
  "onboarding.allowed": "Permitido",
  "onboarding.notAllowed": "Não permitido",
  "onboarding.notSetUp": "Não configurado",
  "onboarding.checking": "A verificar…",

  "onboarding.welcomeTitle": "Bem-vindo ao Minutes",
  "onboarding.welcomeBody":
    "O Minutes grava as suas reuniões e redige a ata. Antes de começar, são necessárias algumas permissões.",
  "onboarding.welcomeOptional": "Cada passo é opcional e poderá alterar tudo mais tarde nas definições.",
  "onboarding.getStarted": "Começar",

  "onboarding.microphoneTitle": "Microfone",
  "onboarding.microphoneBody":
    "O Minutes grava o áudio da reunião a partir do seu microfone. Nada é gravado antes de iniciar uma reunião.",
  "onboarding.microphoneAllow": "Permitir o microfone",
  "onboarding.microphoneDeniedHint":
    "O acesso ao microfone foi recusado e o macOS só pergunta uma vez. Pode ativá-lo em «Privacidade e segurança» → «Microfone».",
  "onboarding.microphoneWindowsHint":
    "O Windows não pede esta permissão às aplicações. Se a gravação não captar nada, verifique se o acesso ao microfone está ativado para aplicações de secretária em «Privacidade e segurança» → «Microfone».",

  "onboarding.browserTitle": "Reuniões abertas no navegador",
  "onboarding.browserBody":
    "Para lhe propor tomar notas quando entra numa reunião do Google Meet ou do Teams a partir de um link, o Minutes verifica se está aberta uma reunião no seu navegador.",
  "onboarding.browserPrivacy":
    "Apenas verifica se um separador é uma reunião — não o conteúdo das páginas, e nada sai do seu dispositivo.",
  "onboarding.browserPerApp":
    "O macOS concede esta permissão navegador a navegador, por isso estão listados separadamente. Só são mostrados os navegadores que tem instalados.",
  "onboarding.browserAllow": "Permitir",
  "onboarding.browserNone":
    "Não foi encontrado nenhum navegador compatível, pelo que não há nada a configurar aqui. As reuniões no Zoom, Teams e Slack são detetadas sem isto.",
  "onboarding.browserDeniedHint":
    "O macOS só pergunta uma vez por navegador. Para alterar, vá a «Privacidade e segurança» → «Automatização» e assinale o Minutes sob esse navegador.",


  "onboarding.detectionUnavailableTitle": "Iniciar uma reunião",
  "onboarding.detectionUnavailableBody":
    "A deteção automática de reuniões só está disponível no macOS para já. Neste sistema, inicie a gravação com «Nova reunião» quando quiser que sejam tomadas notas.",

  "onboarding.doneTitle": "Está tudo pronto",
  "onboarding.doneBody": "Eis a situação atual. Pode voltar a tudo isto nas definições.",
  "onboarding.doneSkipped": "Ignorado — pode configurar isto mais tarde nas definições.",
  "onboarding.finish": "Começar a usar o Minutes",

  "settings.rerunOnboarding": "Permissões e configuração",
  "settings.rerunOnboardingHint":
    "Percorrer novamente a configuração do microfone e da deteção no navegador.",
  "settings.rerunOnboardingAction": "Iniciar a configuração",
};
