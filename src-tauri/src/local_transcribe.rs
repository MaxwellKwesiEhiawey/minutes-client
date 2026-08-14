//! On-device transcription + diarization adapter.
//!
//! This is the heart of Minutes's on-device transcription path: instead of
//! streaming audio to a cloud speech-to-text vendor, every chunk is
//! transcribed locally by whisper.cpp (via `minutes-core`), and speakers are
//! attributed locally by pyannote-rs. No audio ever leaves the device for
//! transcription. Only the finished transcript is later sent to the Minutes
//! server for AI summarization.
//!
//! The public surface mirrors what `recorder.rs` needs:
//! - [`build_config`] turns Minutes settings into a `minutes-core` `Config`.
//! - [`model_present`] / [`ensure_models`] handle the first-run model download.
//! - [`transcribe_samples`] transcribes one audio chunk into speaker-attributed
//!   [`SpeakerLine`]s.

use crate::locking::MutexExt;
use minutes_core::config::Config;
use minutes_core::diarize::{self, DiarizationResult};
use minutes_core::transcribe;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter};

/// Cached result of a full integrity check so we don't re-hash a ~500 MB model
/// on every Settings open or record-button tap (that blocked the UI thread and
/// showed the macOS wait cursor).
#[derive(Clone)]
struct IntegrityCache {
    path: PathBuf,
    modified: SystemTime,
    len: u64,
    ok: bool,
}

static INTEGRITY_CACHE: Mutex<Option<IntegrityCache>> = Mutex::new(None);

/// Drop any cached integrity result (e.g. after a model download completes).
pub fn invalidate_model_cache() {
    *INTEGRITY_CACHE.lock_safe() = None;
}

/// Drop the in-memory whisper context (e.g. after the user changes models).
pub fn invalidate_whisper_runtime_cache() {
    transcribe::invalidate_whisper_context_cache();
}

/// Warm the whisper model cache on a background thread so the first Record
/// press does not block on model load.
pub fn preload_whisper(cfg: &Config) {
    if let Err(e) = transcribe::preload_whisper_model(cfg) {
        tracing::warn!("whisper preload failed: {e}");
    }
}

/// Event emitted to the frontend during a first-run model download so the
/// Settings UI can show real progress instead of an indefinite spinner.
pub const EV_MODEL_PROGRESS: &str = "model-download-progress";

/// If no bytes arrive for this long, treat the download as stalled and error out
/// (rather than hanging forever on a dead connection).
const STALL_TIMEOUT: Duration = Duration::from_secs(60);

/// One transcribed, optionally speaker-attributed line from a chunk.
#[derive(Debug, Clone)]
pub struct SpeakerLine {
    pub text: String,
    /// Raw diarization label (e.g. `SPEAKER_0`) when diarization ran, else None.
    pub speaker_label: Option<String>,
    /// Absolute start offset within the meeting, in milliseconds.
    pub start_ms: Option<i64>,
    /// Absolute end offset within the meeting, in milliseconds.
    pub end_ms: Option<i64>,
}

/// Build a `minutes-core` `Config` for on-device transcription from the small
/// slice of Minutes settings that matter here. We start from the library
/// defaults (which already point `model_path` at `~/.minutes/models`) and then
/// override the engine, model, language, and VAD/diarization toggles.
pub fn build_config(model: &str, language: &str, diarization_enabled: bool) -> Config {
    let mut cfg = Config::default();

    cfg.transcription.engine = "whisper".into();
    let model = model.trim();
    if !model.is_empty() {
        cfg.transcription.model = model.to_string();
    }

    let lang = language.trim();
    cfg.transcription.language = if lang.is_empty() || lang.eq_ignore_ascii_case("auto") {
        None // let whisper auto-detect the spoken language
    } else {
        Some(lang.to_string())
    };

    // Use whisper.cpp's bundled Silero VAD (ggml file) rather than the ort-silero
    // ONNX path, so first-run only needs the single VAD .bin we download below.
    cfg.transcription.vad_engine = "whisper-silero".into();
    // `denoise` is not compiled into Minutes; make sure the pipeline never tries it.
    cfg.transcription.noise_reduction = false;

    // "auto" uses pyannote-rs when the diarization models are present, and simply
    // skips otherwise (never breaking a recording). Explicit "none" disables it.
    cfg.diarization.engine = if diarization_enabled {
        "auto".into()
    } else {
        "none".into()
    };

    cfg
}

fn models_dir(cfg: &Config) -> PathBuf {
    cfg.transcription.model_path.clone()
}

/// Path to the whisper model weights this config resolves to.
pub fn model_file(cfg: &Config) -> PathBuf {
    models_dir(cfg).join(format!("ggml-{}.bin", cfg.transcription.model))
}

/// Canonical SHA256 digests for the default `ggml-{name}.bin` artifacts from
/// `huggingface.co/ggerganov/whisper.cpp/resolve/main/` (verified 2026-07-01).
/// Size-only checks are not enough: a corrupted download can match the expected
/// byte count yet fail whisper with `unknown tensor '' in model file`.
const WHISPER_MODEL_SHA256: &[(&str, &str)] = &[
    (
        "tiny",
        "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
    ),
    (
        "base",
        "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
    ),
    (
        "small",
        "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    ),
    // `medium` / `large-v3` intentionally omitted until their digests are
    // verified from the canonical repo; they fall back to size + load probing.
];

/// SHA256 of the Silero VAD weights (`ggml-silero-v6.2.0.bin`) from
/// `huggingface.co/ggml-org/whisper-vad` (verified 2026-07-05).
const VAD_MODEL_SHA256: &str = "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987";

/// Verify a freshly downloaded file against an expected SHA256. On mismatch the
/// bad file is removed so a later run re-downloads it. Returns a user-facing
/// error message when the digest does not match.
fn verify_sha256(path: &std::path::Path, expected: &str) -> Result<(), String> {
    let actual = sha256_hex(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        let _ = std::fs::remove_file(path);
        Err(format!(
            "checksum mismatch (expected {expected}, got {actual}); removed the bad file"
        ))
    }
}

fn expected_model_sha256(model: &str) -> Option<&'static str> {
    WHISPER_MODEL_SHA256
        .iter()
        .find(|(name, _)| *name == model)
        .map(|(_, digest)| *digest)
}

fn sha256_hex(path: &std::path::Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn model_size_ok(model: &str, len: u64) -> bool {
    if let Some(min_bytes) = transcribe::expected_whisper_model_size_bytes(model) {
        len >= min_bytes
    } else {
        len > 0
    }
}

/// Fast presence check for UI status probes — file exists and meets the minimum
/// size. Does not hash the weights (a full SHA256 of `small` takes seconds and
/// froze the macOS UI when Settings or Record was tapped).
pub fn model_likely_present(cfg: &Config) -> bool {
    let path = model_file(cfg);
    let Ok(meta) = std::fs::metadata(&path) else {
        return false;
    };
    model_size_ok(cfg.transcription.model.as_str(), meta.len())
}

/// Whether the on-disk whisper weights pass size + (when known) SHA256 checks.
fn model_integrity_ok_uncached(cfg: &Config, path: &std::path::Path, len: u64) -> bool {
    let model = cfg.transcription.model.as_str();
    if !model_size_ok(model, len) {
        return false;
    }

    if let Some(expected) = expected_model_sha256(model) {
        return sha256_hex(path)
            .ok()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(expected));
    }
    true
}

fn model_integrity_ok(cfg: &Config) -> bool {
    let path = model_file(cfg);
    let Ok(meta) = std::fs::metadata(&path) else {
        invalidate_model_cache();
        return false;
    };
    let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = meta.len();

    if let Some(cached) = INTEGRITY_CACHE.lock_safe().as_ref() {
        if cached.path == path && cached.modified == modified && cached.len == len {
            return cached.ok;
        }
    }

    let ok = model_integrity_ok_uncached(cfg, &path, len);
    *INTEGRITY_CACHE.lock_safe() = Some(IntegrityCache {
        path,
        modified,
        len,
        ok,
    });
    ok
}

/// Whether the whisper model weights are present and verified intact.
pub fn model_present(cfg: &Config) -> bool {
    model_integrity_ok(cfg)
}

/// After a download, confirm whisper can actually parse the weights. Catches
/// corruption that size/SHA tables have not caught yet (custom quant variants).
async fn probe_whisper_load(cfg: &Config) -> Result<(), String> {
    let path = model_file(cfg);
    let model_name = cfg.transcription.model.clone();
    let cfg = cfg.clone();
    tauri::async_runtime::spawn_blocking(move || transcribe::probe_whisper_model(&cfg))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| {
            let _ = std::fs::remove_file(&path);
            format!(
                "downloaded whisper model '{model_name}' failed integrity check ({e}); removed the bad file — please download again"
            )
        })
}

/// Stream a URL to a destination path, writing atomically via a `.part` file.
///
/// `on_progress(downloaded, total)` is called as bytes arrive so the caller can
/// report progress. A per-read stall timeout guarantees the download fails with
/// a clear error instead of hanging forever on a dead/stalled connection.
async fn download_file(
    url: &str,
    dest: &std::path::Path,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // No total request timeout (a 465 MB model on a slow link can take a while);
    // instead we enforce a connect timeout + a per-read stall timeout below.
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let mut resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("download failed for {url}: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("download failed for {url}: HTTP {}", resp.status()));
    }

    let total = resp.content_length();
    let part = dest.with_extension("part");
    let mut file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    on_progress(0, total);

    loop {
        // Bound each read so a stalled connection can't hang indefinitely.
        let next = tokio::time::timeout(STALL_TIMEOUT, resp.chunk()).await;
        let chunk = match next {
            Ok(Ok(Some(chunk))) => chunk,
            Ok(Ok(None)) => break, // finished
            Ok(Err(e)) => {
                let _ = std::fs::remove_file(&part);
                return Err(format!("download error for {url}: {e}"));
            }
            Err(_) => {
                let _ = std::fs::remove_file(&part);
                return Err(format!(
                    "download stalled (no data for {}s). Check your connection and try again.",
                    STALL_TIMEOUT.as_secs()
                ));
            }
        };
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(&part, dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// Emit a throttled progress event for the Settings UI.
fn emit_progress(
    app: &AppHandle,
    stage: &str,
    label: &str,
    downloaded: u64,
    total: Option<u64>,
    done: bool,
) {
    let _ = app.emit(
        EV_MODEL_PROGRESS,
        json!({
            "stage": stage,
            "label": label,
            "downloaded": downloaded,
            "total": total,
            "done": done,
        }),
    );
}

/// Download one file with throttled progress events (every ~2 MB) and a
/// completion event.
async fn download_with_progress(
    app: &AppHandle,
    stage: &str,
    label: &str,
    url: &str,
    dest: &std::path::Path,
) -> Result<(), String> {
    let mut last_emit: u64 = 0;
    let app_cb = app.clone();
    let stage_s = stage.to_string();
    let label_s = label.to_string();
    let result = download_file(url, dest, move |downloaded, total| {
        // Throttle: emit on start and every ~2 MB so we don't flood the bridge.
        if downloaded == 0 || downloaded - last_emit >= 2 * 1024 * 1024 {
            last_emit = downloaded;
            emit_progress(&app_cb, &stage_s, &label_s, downloaded, total, false);
        }
    })
    .await;
    // Always emit a terminal event (done or reset) so the UI can move on.
    emit_progress(app, stage, label, 0, None, result.is_ok());
    result
}

/// Ensure the whisper model, the Silero VAD model, and (when diarization is
/// enabled) the pyannote-rs models are present, downloading any that are
/// missing. Safe to call repeatedly; already-present files are skipped.
///
/// This is Minutes's equivalent of `minutes setup --model small [--diarization]`,
/// surfaced as a first-run step so the user never has to touch a terminal.
/// Progress is streamed to the frontend via [`EV_MODEL_PROGRESS`].
pub async fn ensure_models(
    app: &AppHandle,
    cfg: &Config,
    diarization_enabled: bool,
) -> Result<(), String> {
    // 1. Whisper weights (e.g. ggml-small.bin) from the official whisper.cpp repo.
    if !model_present(cfg) {
        let url = format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            cfg.transcription.model
        );
        let label = format!("Whisper model ({})", cfg.transcription.model);
        download_with_progress(app, "whisper", &label, &url, &model_file(cfg)).await?;
        if !model_integrity_ok(cfg) {
            let _ = std::fs::remove_file(model_file(cfg));
            return Err(format!(
                "downloaded whisper model '{}' failed checksum verification; please retry on a stable connection",
                cfg.transcription.model
            ));
        }
        probe_whisper_load(cfg).await?;
        invalidate_model_cache();
    }

    // 2. Silero VAD (prevents whisper hallucination loops on quiet/non-English audio).
    let vad_dest = models_dir(cfg).join("ggml-silero-v6.2.0.bin");
    if !vad_dest.exists() {
        let vad_url =
            "https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v6.2.0.bin";
        // A VAD failure is non-fatal: transcription still works without it.
        match download_with_progress(app, "vad", "Voice-activity model", vad_url, &vad_dest).await {
            Ok(()) => {
                if let Err(e) = verify_sha256(&vad_dest, VAD_MODEL_SHA256) {
                    tracing::warn!("VAD model {e}; continuing without VAD");
                }
            }
            Err(e) => {
                tracing::warn!("VAD model download failed ({e}); continuing without VAD")
            }
        }
    }

    // 3. Diarization (speaker) models — segmentation + speaker embedding ONNX.
    if diarization_enabled {
        let emb = diarize::embedding_model_for_config(cfg);
        let dia_dir = cfg.diarization.model_path.clone();
        let items: [(&str, &str, &str); 2] = [
            (
                "diarization-seg",
                diarize::SEGMENTATION_MODEL,
                diarize::SEGMENTATION_MODEL_URL,
            ),
            ("diarization-emb", emb.filename, emb.url),
        ];
        for (stage, filename, url) in items {
            let dest = dia_dir.join(filename);
            if !dest.exists() {
                let label = format!("Speaker model ({filename})");
                if let Err(e) = download_with_progress(app, stage, &label, url, &dest).await {
                    // Non-fatal: diarization "auto" simply skips when models are absent.
                    tracing::warn!("diarization model '{filename}' download failed ({e}); speaker labels disabled until it succeeds");
                }
            }
        }
    }

    Ok(())
}

/// Transcribe one audio chunk into speaker-attributed lines.
///
/// `samples` is mono `f32` PCM at `sample_rate`. `base_offset_secs` is where this
/// chunk begins within the overall meeting, so returned timings are absolute.
///
/// Runs the (blocking, CPU/GPU-bound) whisper + pyannote work on a blocking
/// thread so the async runtime is never stalled.
pub async fn transcribe_samples(
    cfg: &Config,
    samples: &[f32],
    sample_rate: u32,
    diarization_enabled: bool,
    base_offset_secs: f64,
) -> Result<Vec<SpeakerLine>, String> {
    let samples_16k =
        crate::audio::resample(samples, sample_rate, crate::audio::TARGET_SAMPLE_RATE);

    let cfg = cfg.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        run_blocking(&cfg, &samples_16k, diarization_enabled, base_offset_secs)
    })
    .await
    .map_err(|e| e.to_string())?;

    result
}

/// Write the 16 kHz mono chunk to a private, self-cleaning temp file for
/// diarization. Returns the [`tempfile::NamedTempFile`] handle: keep it alive
/// while reading the path, then drop it to delete the file.
fn write_temp_wav_16k(samples: &[f32]) -> Result<tempfile::NamedTempFile, String> {
    use std::io::Write as _;
    let wav = crate::audio::encode_wav_16k(samples, crate::audio::TARGET_SAMPLE_RATE)
        .map_err(|e| e.to_string())?;
    let mut file = tempfile::Builder::new()
        .prefix("desksec-chunk-")
        .suffix(".wav")
        .tempfile()
        .map_err(|e| format!("failed to create temp audio file: {e}"))?;
    file.write_all(&wav)
        .map_err(|e| format!("failed to write temp audio file: {e}"))?;
    file.flush().map_err(|e| e.to_string())?;
    Ok(file)
}

fn run_blocking(
    cfg: &Config,
    samples_16k: &[f32],
    diarization_enabled: bool,
    base_offset_secs: f64,
) -> Result<Vec<SpeakerLine>, String> {
    let transcript =
        transcribe::transcribe_pcm_16k_mono(samples_16k, cfg).map_err(|e| e.to_string())?;

    // Diarize from a temp WAV only when speaker labels are requested. The
    // NamedTempFile is created with restrictive (0600) permissions and is removed
    // when `wav` drops — even on panic or early return — so a plaintext audio
    // fragment is never left behind in shared temp storage.
    let diar: Option<DiarizationResult> = if diarization_enabled {
        let wav = write_temp_wav_16k(samples_16k)?;
        let result = diarize::diarize(wav.path(), cfg);
        drop(wav); // delete the chunk as soon as diarization is done
        result
    } else {
        None
    };

    let base_ms = (base_offset_secs * 1000.0) as i64;

    // Preferred path: per-segment timings from whisper, each attributed to the
    // speaker active at its midpoint.
    if !transcript.segments.is_empty() {
        let mut lines = Vec::with_capacity(transcript.segments.len());
        for seg in &transcript.segments {
            let text = seg.text.trim().to_string();
            if text.is_empty() {
                continue;
            }
            let speaker_label = diar
                .as_ref()
                .and_then(|d| speaker_at(d, (seg.start + seg.end) / 2.0));
            lines.push(SpeakerLine {
                text,
                speaker_label,
                start_ms: Some(base_ms + (seg.start * 1000.0) as i64),
                end_ms: Some(base_ms + (seg.end * 1000.0) as i64),
            });
        }
        if !lines.is_empty() {
            return Ok(lines);
        }
    }

    // Fallback: aggregated text with no per-segment timing (chunked whisper path).
    let text = transcript.text.trim().to_string();
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let speaker_label = diar
        .as_ref()
        .and_then(|d| d.segments.first().map(|s| s.speaker.clone()));
    Ok(vec![SpeakerLine {
        text,
        speaker_label,
        start_ms: Some(base_ms),
        end_ms: None,
    }])
}

/// Find the diarization speaker whose segment covers `t` seconds (chunk-relative),
/// falling back to the nearest segment by start time.
fn speaker_at(diar: &DiarizationResult, t: f64) -> Option<String> {
    if let Some(seg) = diar.segments.iter().find(|s| t >= s.start && t <= s.end) {
        return Some(seg.speaker.clone());
    }
    diar.segments
        .iter()
        .min_by(|a, b| {
            (a.start - t)
                .abs()
                .partial_cmp(&(b.start - t).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|s| s.speaker.clone())
}

/// One on-disk model artifact the user can remove from Settings.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledModelEntry {
    /// Stable id passed to [`delete_installed_model`]: a whisper name (`small`),
    /// `vad`, or `diarization`.
    pub id: String,
    pub kind: String,
    pub label: String,
    pub size_bytes: u64,
    /// True when deleting this entry would affect the currently selected whisper
    /// model or an enabled diarization path.
    pub in_use: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledModelsInfo {
    pub models: Vec<InstalledModelEntry>,
    pub models_dir: String,
    pub total_bytes: u64,
}

fn whisper_label(model: &str) -> String {
    match model {
        "tiny" => "Whisper tiny".into(),
        "base" => "Whisper base".into(),
        "small" => "Whisper small".into(),
        "medium" => "Whisper medium".into(),
        "large-v3" => "Whisper large v3".into(),
        other => format!("Whisper {other}"),
    }
}

fn nonzero_file_size(path: &std::path::Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()
        .map(|m| m.len())
        .filter(|len| *len > 0)
}

fn remove_model_file(path: &std::path::Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| format!("failed to delete {}: {e}", path.display()))?;
    }
    let part = path.with_extension("part");
    if part.exists() {
        std::fs::remove_file(&part).ok();
    }
    Ok(())
}

/// List whisper, VAD, and diarization artifacts present on disk.
pub fn list_installed_models(
    cfg: &Config,
    active_whisper: &str,
    diarization_enabled: bool,
) -> InstalledModelsInfo {
    let dir = models_dir(cfg);
    let dia_dir = cfg.diarization.model_path.clone();
    let mut models = Vec::new();
    let mut total_bytes = 0u64;

    for name in crate::settings::VALID_WHISPER_MODELS {
        let path = dir.join(format!("ggml-{name}.bin"));
        if let Some(size) = nonzero_file_size(&path) {
            total_bytes += size;
            models.push(InstalledModelEntry {
                id: name.to_string(),
                kind: "whisper".into(),
                label: whisper_label(name),
                size_bytes: size,
                in_use: name == active_whisper,
            });
        }
    }

    let vad_path = dir.join("ggml-silero-v6.2.0.bin");
    if let Some(size) = nonzero_file_size(&vad_path) {
        total_bytes += size;
        models.push(InstalledModelEntry {
            id: "vad".into(),
            kind: "vad".into(),
            label: "Voice activity (Silero VAD)".into(),
            size_bytes: size,
            in_use: false,
        });
    }

    let emb = diarize::embedding_model_for_config(cfg);
    let seg_path = dia_dir.join(diarize::SEGMENTATION_MODEL);
    let emb_path = dia_dir.join(emb.filename);
    let seg_size = nonzero_file_size(&seg_path).unwrap_or(0);
    let emb_size = nonzero_file_size(&emb_path).unwrap_or(0);
    if seg_size > 0 || emb_size > 0 {
        let size = seg_size + emb_size;
        total_bytes += size;
        models.push(InstalledModelEntry {
            id: "diarization".into(),
            kind: "diarization".into(),
            label: "Speaker identification (diarization)".into(),
            size_bytes: size,
            in_use: diarization_enabled,
        });
    }

    InstalledModelsInfo {
        models,
        models_dir: dir.display().to_string(),
        total_bytes,
    }
}

/// Delete one installed model group. Refuses while a recording is active.
pub fn delete_installed_model(
    cfg: &Config,
    model_id: &str,
    active_whisper: &str,
    recording_active: bool,
) -> Result<(), String> {
    if recording_active {
        return Err("Stop recording before deleting models.".into());
    }

    match model_id {
        "vad" => {
            remove_model_file(&models_dir(cfg).join("ggml-silero-v6.2.0.bin"))?;
        }
        "diarization" => {
            let dia_dir = cfg.diarization.model_path.clone();
            let emb = diarize::embedding_model_for_config(cfg);
            remove_model_file(&dia_dir.join(diarize::SEGMENTATION_MODEL))?;
            remove_model_file(&dia_dir.join(emb.filename))?;
        }
        whisper if crate::settings::VALID_WHISPER_MODELS.contains(&whisper) => {
            let path = models_dir(cfg).join(format!("ggml-{whisper}.bin"));
            if !path.exists() {
                return Err(format!("model '{whisper}' is not installed"));
            }
            remove_model_file(&path)?;
            if whisper == active_whisper {
                invalidate_whisper_runtime_cache();
            }
        }
        _ => return Err(format!("unknown model: {model_id}")),
    }

    invalidate_model_cache();
    Ok(())
}

#[cfg(test)]
mod model_storage_tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(dir: &TempDir) -> Config {
        let mut cfg = Config::default();
        cfg.transcription.model_path = dir.path().join("whisper");
        cfg.diarization.model_path = dir.path().join("diarize");
        std::fs::create_dir_all(&cfg.transcription.model_path).unwrap();
        std::fs::create_dir_all(&cfg.diarization.model_path).unwrap();
        cfg
    }

    #[test]
    fn list_installed_models_reports_whisper_and_vad() {
        let dir = TempDir::new().unwrap();
        let cfg = test_config(&dir);
        std::fs::write(
            cfg.transcription.model_path.join("ggml-small.bin"),
            vec![0u8; 1024],
        )
        .unwrap();
        std::fs::write(
            cfg.transcription.model_path.join("ggml-silero-v6.2.0.bin"),
            vec![0u8; 512],
        )
        .unwrap();

        let info = list_installed_models(&cfg, "base", false);
        assert_eq!(info.models.len(), 2);
        assert_eq!(info.total_bytes, 1024 + 512);
        assert!(info.models.iter().all(|m| !m.in_use));
    }

    #[test]
    fn delete_installed_model_removes_whisper_weights() {
        let dir = TempDir::new().unwrap();
        let cfg = test_config(&dir);
        let path = cfg.transcription.model_path.join("ggml-small.bin");
        std::fs::write(&path, b"weights").unwrap();

        delete_installed_model(&cfg, "small", "base", false).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn delete_installed_model_blocks_while_recording() {
        let dir = TempDir::new().unwrap();
        let cfg = test_config(&dir);
        let path = cfg.transcription.model_path.join("ggml-small.bin");
        std::fs::write(&path, b"weights").unwrap();

        let err = delete_installed_model(&cfg, "small", "small", true)
            .expect_err("should refuse while recording");
        assert!(err.contains("Stop recording"));
        assert!(path.exists());
    }
}
