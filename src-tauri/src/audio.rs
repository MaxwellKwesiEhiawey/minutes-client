//! Audio capture.
//!
//! Capture is modelled as a set of independent [`CaptureSource`]s — typically a
//! microphone, optionally plus system-audio loopback — that are mixed into one
//! mono stream. Two properties matter and drive the whole shape of this module:
//!
//! 1. **Every source normalizes to [`TARGET_SAMPLE_RATE`] before it leaves its
//!    capture callback.** The rest of the app therefore never has to know the
//!    device's native rate, which is what makes it safe to swap devices in the
//!    middle of a recording: a 48 kHz headset can be replaced by a 44.1 kHz
//!    built-in mic and the transcription pipeline downstream never notices.
//!
//! 2. **Each source is supervised and rebuilt on loss.** cpal reports device
//!    loss/rerouting through the stream error callback but does *not* reopen the
//!    stream, so without a supervisor a Bluetooth headset disconnecting leaves a
//!    live-looking recording that captures nothing for the rest of the meeting.
//!    A microphone is additionally watchdogged for silence, because on Windows
//!    an endpoint can stop delivering frames without any error being raised.
//!
//! The watchdog applies to microphones **only**. A WASAPI loopback capture on an
//! idle output device delivers zero frames — not silence, literally no callbacks
//! at all — so "no data" is a normal state for a system-audio source and must
//! never be treated as device loss.

use crate::models::{AudioDeviceKind, AudioDevicesResponse, AudioInputDevice};
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

/// Rate every capture source is resampled to before leaving this module. Chosen
/// for on-device Whisper (whisper.cpp expects 16 kHz) and also what the online
/// Deepgram stream is fed.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Stored device id prefix for Windows WASAPI loopback (capture from a speaker/output device).
pub const WASAPI_LOOPBACK_PREFIX: &str = "wasapi-loopback::";

/// How long a microphone may deliver no samples at all before we assume the
/// endpoint is gone and reopen against whatever device is current. A capture
/// stream on a live device delivers frames continuously even in a silent room,
/// so a gap this long is device loss, not quiet.
const MIC_STALL_TIMEOUT: Duration = Duration::from_secs(3);

/// Grace period before the *first* stall verdict on a freshly opened stream.
///
/// A Bluetooth headset must renegotiate from A2DP (stereo) to Hands-Free profile
/// before its microphone produces anything, and that takes several seconds.
/// Judging it at [`MIC_STALL_TIMEOUT`] tears the stream down and restarts the
/// very negotiation it is waiting on, so the mic can never come alive — observed
/// in the field as 20 reopen cycles over 62 seconds. Once a stream has delivered
/// its first samples the strict timeout applies, because by then silence really
/// does mean the device went away.
const FIRST_SAMPLES_GRACE: Duration = Duration::from_secs(8);

/// How long a device that opened but delivered nothing is passed over, and the
/// ceiling as that grows on repeat offences.
///
/// Escalating rather than fixed: a device that is present-but-dead — a Bluetooth
/// headset whose mic endpoint exists while the radio is busy carrying stereo
/// audio — would otherwise cost the user several seconds of microphone every time
/// the bench expired, forever.
const STALL_BENCH_BASE: Duration = Duration::from_secs(60);
const STALL_BENCH_MAX: Duration = Duration::from_secs(30 * 60);

/// Delay before the first reopen attempt, doubled on each consecutive failure.
const REOPEN_BACKOFF: Duration = Duration::from_millis(300);
const REOPEN_BACKOFF_MAX: Duration = Duration::from_secs(5);

/// Attempts at the *initial* open before giving up and failing the recording.
/// More than one because pressing record moments after a device was unplugged
/// catches Windows mid-transition, when activating an endpoint can fail for a
/// beat and then succeed.
const INITIAL_OPEN_ATTEMPTS: u32 = 3;

/// How often a running source re-checks whether a better device is now available
/// — a headset being plugged in, the OS default moving, a pinned device coming
/// back. Device *loss* does not depend on this: cpal's error callback reports
/// that within milliseconds. This interval only bounds how long we keep using a
/// device after a preferable one appears.
const DEVICE_RECHECK_INTERVAL: Duration = Duration::from_millis(750);

/// Consecutive re-checks that must agree before we act. Device enumeration is
/// briefly inconsistent while Windows brings an endpoint up or down, and
/// switching on a transient reading would cut audio for no reason.
const SWITCH_CONFIRMATIONS: u32 = 2;

/// How long to stop wanting a device after trying it and landing somewhere else.
/// Without this, a device that is preferred but cannot be opened (in use by
/// another app, driver wedged) would be retried every [`DEVICE_RECHECK_INTERVAL`]
/// forever, tearing down a working stream each time.
const SWITCH_COOLDOWN: Duration = Duration::from_secs(30);

/// How often the mixer drains its inputs and emits a mixed batch.
const MIXER_TICK: Duration = Duration::from_millis(20);

/// A source that has delivered nothing for this long is treated as idle and
/// stops holding back the mix. This is what keeps a paused or silent
/// system-audio loopback — which delivers no callbacks at all — from stalling
/// the microphone.
const SOURCE_IDLE_AFTER: Duration = Duration::from_millis(500);

/// Per-source mixer backlog cap, 1 s. Two capture devices run off independent
/// clocks and drift apart, so the faster one is trimmed here instead of being
/// allowed to grow latency without bound. Deliberately larger than
/// [`SOURCE_IDLE_AFTER`] so nothing is dropped while a source is going idle.
const MIX_QUEUE_CAP: usize = TARGET_SAMPLE_RATE as usize;

const LOOPBACK_PATTERNS: &[&str] = &[
    "monitor of",
    ".monitor",
    "monitor sink",
    "stereo mix",
    "what u hear",
    "wave out mix",
    "loopback",
    "blackhole",
    "soundflower",
    "vb-audio cable",
    "virtual cable",
    "cable output",
    "speakers (loopback",
    "speaker (loopback",
    "output (loopback",
    "wasapi.loopback",
];

const MICROPHONE_PATTERNS: &[&str] = &[
    "microphone",
    "headset",
    "headphone",
    "airpods",
    "built-in input",
    "usb audio",
    "webcam",
    " mic ",
];

/// cpal 0.18 removed `Device::name()`; the name now lives on `DeviceDescription`.
fn device_name(d: &cpal::Device) -> Option<String> {
    d.description().ok().map(|desc| desc.name().to_string())
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    }
}

pub fn is_wasapi_loopback_id(name: &str) -> bool {
    name.starts_with(WASAPI_LOOPBACK_PREFIX)
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn wasapi_loopback_id(device_name: &str) -> String {
    format!("{WASAPI_LOOPBACK_PREFIX}{device_name}")
}

fn wasapi_loopback_device_name(stored: &str) -> &str {
    stored
        .strip_prefix(WASAPI_LOOPBACK_PREFIX)
        .unwrap_or(stored)
}

fn display_device_name(name: &str) -> &str {
    wasapi_loopback_device_name(name)
}

/// Heuristic classification of an input device name.
pub fn classify_device(name: &str) -> AudioDeviceKind {
    if is_wasapi_loopback_id(name) {
        return AudioDeviceKind::Loopback;
    }
    let lower = display_device_name(name).to_lowercase();
    if LOOPBACK_PATTERNS.iter().any(|p| lower.contains(p)) {
        return AudioDeviceKind::Loopback;
    }
    if MICROPHONE_PATTERNS.iter().any(|p| lower.contains(p))
        || lower.ends_with(" mic")
        || lower.starts_with("mic ")
    {
        return AudioDeviceKind::Microphone;
    }
    AudioDeviceKind::Unknown
}

fn device_label(kind: AudioDeviceKind, name: &str) -> String {
    let display = display_device_name(name);
    match kind {
        AudioDeviceKind::Loopback => format!("[System audio] {display}"),
        AudioDeviceKind::Microphone => format!("[Microphone] {display}"),
        AudioDeviceKind::Unknown => display.to_string(),
    }
}

fn kind_rank(kind: AudioDeviceKind) -> u8 {
    match kind {
        AudioDeviceKind::Loopback => 0,
        AudioDeviceKind::Microphone => 1,
        AudioDeviceKind::Unknown => 2,
    }
}

/// List input devices with loopback/microphone labels for the Settings UI.
pub fn list_input_devices() -> AudioDevicesResponse {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    if let Ok(inputs) = host.input_devices() {
        for d in inputs {
            if let Some(name) = device_name(&d) {
                if devices
                    .iter()
                    .any(|existing: &AudioInputDevice| existing.name == name)
                {
                    continue;
                }
                let kind = classify_device(&name);
                devices.push(AudioInputDevice {
                    label: device_label(kind, &name),
                    kind,
                    name,
                });
            }
        }
    }
    #[cfg(target_os = "windows")]
    append_wasapi_loopback_devices(&host, &mut devices);
    devices.sort_by(|a, b| {
        kind_rank(a.kind)
            .cmp(&kind_rank(b.kind))
            .then_with(|| a.name.cmp(&b.name))
    });
    let has_loopback = devices.iter().any(|d| d.kind == AudioDeviceKind::Loopback);
    AudioDevicesResponse {
        platform: platform_name().to_string(),
        has_loopback,
        devices,
    }
}

/// On Windows, cpal exposes system-audio loopback via output (speaker) devices, not inputs.
#[cfg(target_os = "windows")]
fn append_wasapi_loopback_devices(host: &cpal::Host, devices: &mut Vec<AudioInputDevice>) {
    if let Ok(outputs) = host.output_devices() {
        for d in outputs {
            if let Some(raw_name) = device_name(&d) {
                let name = wasapi_loopback_id(&raw_name);
                if devices.iter().any(|existing| existing.name == name) {
                    continue;
                }
                devices.push(AudioInputDevice {
                    label: device_label(AudioDeviceKind::Loopback, &raw_name),
                    kind: AudioDeviceKind::Loopback,
                    name,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Capture sources
// ---------------------------------------------------------------------------

/// What a capture source is for. Drives device resolution (input endpoints vs
/// output endpoints in loopback mode) and whether the silence watchdog applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRole {
    /// The user's voice.
    Microphone,
    /// Everything the machine is playing — the far side of a meeting call.
    SystemAudio,
}

impl SourceRole {
    fn thread_label(self) -> &'static str {
        match self {
            SourceRole::Microphone => "mic",
            SourceRole::SystemAudio => "sysaudio",
        }
    }

    /// Human-readable, for messages that reach the UI.
    pub fn describe(self) -> &'static str {
        match self {
            SourceRole::Microphone => "microphone",
            SourceRole::SystemAudio => "system audio",
        }
    }
}

/// One thing to capture from.
#[derive(Debug, Clone)]
pub struct CaptureSource {
    pub role: SourceRole,
    /// Stored device name, or `None` for "whatever the OS default is right now".
    /// A named device that has disappeared falls back to the current default so
    /// an unplugged headset does not end the recording.
    pub device: Option<String>,
}

impl CaptureSource {
    pub fn microphone(device: Option<String>) -> Self {
        CaptureSource {
            role: SourceRole::Microphone,
            device,
        }
    }

    pub fn system_audio(device: Option<String>) -> Self {
        CaptureSource {
            role: SourceRole::SystemAudio,
            device,
        }
    }
}

/// Why a source changed device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchReason {
    /// The device we were using went away.
    DeviceLost,
    /// A preferable device became available — a headset was connected, the OS
    /// default moved, or the user's configured device came back.
    Preferred,
    /// The device opened fine but produced no audio at all, so it was passed over.
    /// The common cause is a Bluetooth headset whose microphone endpoint exists
    /// while the radio is busy carrying stereo playback.
    NoAudio,
}

/// Something the capture layer wants the user to know about mid-recording.
#[derive(Debug, Clone)]
pub enum CaptureNotice {
    /// A source is now capturing from a different `device`.
    SwitchedDevice {
        role: SourceRole,
        device: String,
        reason: SwitchReason,
    },
    /// A source could not be reopened. Capture continues with the remaining
    /// sources; this one keeps retrying in the background.
    SourceUnavailable { role: SourceRole, reason: String },
}

impl CaptureNotice {
    /// The message shown to the user.
    pub fn message(&self) -> String {
        match self {
            CaptureNotice::SwitchedDevice {
                role,
                device,
                reason: SwitchReason::DeviceLost,
            } => format!("{} disconnected — switched to {device}", role.describe()),
            CaptureNotice::SwitchedDevice {
                role,
                device,
                reason: SwitchReason::Preferred,
            } => format!("{} switched to {device}", role.describe()),
            CaptureNotice::SwitchedDevice {
                role,
                device,
                reason: SwitchReason::NoAudio,
            } => format!(
                "{} was not delivering audio — switched to {device}",
                role.describe()
            ),
            CaptureNotice::SourceUnavailable { role, reason } => {
                format!("{} unavailable: {reason}", role.describe())
            }
        }
    }
}

/// Callback used to report [`CaptureNotice`]s from the capture threads.
pub type NoticeFn = Arc<dyn Fn(CaptureNotice) + Send + Sync>;

/// Owns every thread involved in a running capture.
pub struct CaptureSession {
    running: Arc<AtomicBool>,
    supervisors: Vec<JoinHandle<()>>,
    mixer: Option<JoinHandle<()>>,
    bridge: Option<JoinHandle<()>>,
}

impl CaptureSession {
    /// Stop capture and wait for every thread to finish.
    pub fn stop(self) {
        // `Drop` does the work — this exists so call sites read intentionally.
        drop(self);
    }
}

/// Cleanup lives in `Drop` rather than only in [`CaptureSession::stop`] so an
/// early return between starting capture and storing the session (creating the
/// meeting row can fail, for one) cannot leak the capture threads.
impl Drop for CaptureSession {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        for join in self.supervisors.drain(..) {
            let _ = join.join();
        }
        if let Some(j) = self.mixer.take() {
            let _ = j.join();
        }
        if let Some(j) = self.bridge.take() {
            let _ = j.join();
        }
    }
}

/// Start capturing from `sources`, mixed down to one mono 16 kHz stream.
///
/// Returns once every source has been opened successfully at least once. A
/// microphone that cannot be opened is fatal — the user needs to know before the
/// meeting starts. A system-audio source that cannot be opened is reported
/// through `notify` and retried in the background, since a recording with only
/// the microphone is still worth having.
pub fn start_capture(
    sources: Vec<CaptureSource>,
    sample_tx: UnboundedSender<Vec<f32>>,
    notify: NoticeFn,
) -> Result<CaptureSession> {
    if sources.is_empty() {
        return Err(anyhow!("no capture sources configured"));
    }

    let running = Arc::new(AtomicBool::new(true));
    let mut supervisors = Vec::new();
    let mut receivers = Vec::new();

    for source in sources {
        let role = source.role;
        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        // Tracks whether this source currently has a live stream, so the mixer
        // can stop pacing on it the instant it goes down for a device change.
        let streaming = Arc::new(AtomicBool::new(false));
        match spawn_source(
            source,
            tx,
            running.clone(),
            notify.clone(),
            streaming.clone(),
        ) {
            Ok(join) => {
                supervisors.push(join);
                receivers.push((rx, streaming));
            }
            Err(e) => {
                if role == SourceRole::Microphone {
                    // Fatal: tear down anything already started.
                    join_all(&running, supervisors, None, None);
                    return Err(e);
                }
                tracing::warn!("{} capture unavailable at start: {e}", role.describe());
                notify(CaptureNotice::SourceUnavailable {
                    role,
                    reason: e.to_string(),
                });
            }
        }
    }

    if receivers.is_empty() {
        running.store(false, Ordering::SeqCst);
        return Err(anyhow!("no audio capture source could be opened"));
    }

    // A single source needs no mixing — hand its channel straight to the bridge.
    let mut mixer = None;
    let bridge_rx = if receivers.len() == 1 {
        // No mixing, so no pacing to decouple — the `streaming` flag is unused.
        receivers.pop().expect("one receiver").0
    } else {
        let (mixed_tx, mixed_rx) = mpsc::channel::<Vec<f32>>();
        match spawn_mixer(receivers, mixed_tx, running.clone()) {
            Ok(m) => mixer = Some(m),
            Err(e) => {
                join_all(&running, supervisors, None, None);
                return Err(e);
            }
        }
        mixed_rx
    };

    let bridge = match bridge_to_tokio(bridge_rx, sample_tx, running.clone()) {
        Ok(b) => b,
        Err(e) => {
            join_all(&running, supervisors, mixer, None);
            return Err(e);
        }
    };

    Ok(CaptureSession {
        running,
        supervisors,
        mixer,
        bridge: Some(bridge),
    })
}

/// Stop the session flag and join whatever was already spawned.
fn join_all(
    running: &AtomicBool,
    supervisors: Vec<JoinHandle<()>>,
    mixer: Option<JoinHandle<()>>,
    bridge: Option<JoinHandle<()>>,
) {
    running.store(false, Ordering::SeqCst);
    for join in supervisors {
        let _ = join.join();
    }
    if let Some(j) = mixer {
        let _ = j.join();
    }
    if let Some(j) = bridge {
        let _ = j.join();
    }
}

fn bridge_to_tokio(
    std_rx: mpsc::Receiver<Vec<f32>>,
    out_tx: UnboundedSender<Vec<f32>>,
    running: Arc<AtomicBool>,
) -> Result<JoinHandle<()>> {
    // Propagate instead of panicking: spawning can fail on a transient OS limit
    // (thread/FD exhaustion), and that should surface as a recoverable "couldn't
    // start recording" error, not crash the whole app.
    thread::Builder::new()
        .name("audio-bridge".into())
        .spawn(move || {
            while running.load(Ordering::SeqCst) {
                match std_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(batch) => {
                        if out_tx.send(batch).is_err() {
                            break;
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .map_err(|e| anyhow!("failed to spawn audio bridge thread: {e}"))
}

struct MixInput {
    rx: mpsc::Receiver<Vec<f32>>,
    queue: VecDeque<f32>,
    last_data: Option<Instant>,
    /// False while this source's supervisor is between streams. Without it, a
    /// source counts as active for [`SOURCE_IDLE_AFTER`] after its last sample,
    /// so a microphone being swapped would hold system audio back for half a
    /// second — and system audio is required to keep flowing through a
    /// microphone change.
    streaming: Arc<AtomicBool>,
}

impl MixInput {
    /// Whether this source is currently producing audio. Idle sources are
    /// excluded from the pacing decision below.
    fn is_active(&self) -> bool {
        self.streaming.load(Ordering::Relaxed)
            && self
                .last_data
                .is_some_and(|t| t.elapsed() < SOURCE_IDLE_AFTER)
    }
}

/// Sum every source sample-for-sample.
///
/// All sources are already at [`TARGET_SAMPLE_RATE`], so this is a straight add
/// with a clamp rather than an average: averaging would halve the microphone
/// whenever system audio is idle, and an idle system-audio loopback is the
/// common case.
///
/// Pacing is set by the *slowest currently active* source, so a source with a
/// momentary gap does not get zeros punched into it. Idle sources are ignored
/// entirely — otherwise a silent loopback would stop the mix dead — and they
/// simply contribute silence to the sum.
fn spawn_mixer(
    receivers: Vec<(mpsc::Receiver<Vec<f32>>, Arc<AtomicBool>)>,
    out: mpsc::Sender<Vec<f32>>,
    running: Arc<AtomicBool>,
) -> Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("audio-mixer".into())
        .spawn(move || {
            let mut inputs: Vec<MixInput> = receivers
                .into_iter()
                .map(|(rx, streaming)| MixInput {
                    rx,
                    queue: VecDeque::new(),
                    last_data: None,
                    streaming,
                })
                .collect();

            while running.load(Ordering::SeqCst) {
                thread::sleep(MIXER_TICK);

                for input in inputs.iter_mut() {
                    let mut received = false;
                    while let Ok(batch) = input.rx.try_recv() {
                        input.queue.extend(batch);
                        received = true;
                    }
                    if received {
                        input.last_data = Some(Instant::now());
                    }
                    if input.queue.len() > MIX_QUEUE_CAP {
                        let excess = input.queue.len() - MIX_QUEUE_CAP;
                        input.queue.drain(0..excess);
                    }
                }

                let n = inputs
                    .iter()
                    .filter(|i| i.is_active())
                    .map(|i| i.queue.len())
                    .min()
                    .unwrap_or(0);
                if n == 0 {
                    continue;
                }

                let mut mixed = Vec::with_capacity(n);
                for i in 0..n {
                    let sum: f32 = inputs
                        .iter()
                        .map(|input| input.queue.get(i).copied().unwrap_or(0.0))
                        .sum();
                    mixed.push(sum.clamp(-1.0, 1.0));
                }
                for input in inputs.iter_mut() {
                    let take = n.min(input.queue.len());
                    input.queue.drain(0..take);
                }

                if out.send(mixed).is_err() {
                    break;
                }
            }
        })
        .map_err(|e| anyhow!("failed to spawn audio mixer thread: {e}"))
}

/// Spawn a source's supervisor thread and wait for its first successful open.
fn spawn_source(
    source: CaptureSource,
    out: mpsc::Sender<Vec<f32>>,
    running: Arc<AtomicBool>,
    notify: NoticeFn,
    streaming: Arc<AtomicBool>,
) -> Result<JoinHandle<()>> {
    let (ready_tx, ready_rx) = mpsc::channel::<Result<String, String>>();
    let label = source.role.thread_label();
    let join = thread::Builder::new()
        .name(format!("audio-{label}"))
        .spawn(move || supervise_source(source, out, running, notify, streaming, ready_tx))
        .map_err(|e| anyhow!("failed to spawn audio capture thread: {e}"))?;

    match ready_rx.recv() {
        Ok(Ok(_device)) => Ok(join),
        Ok(Err(e)) => {
            // supervise_source returns immediately after reporting a fatal open
            // failure, so joining cannot block.
            let _ = join.join();
            Err(anyhow!(e))
        }
        Err(_) => {
            let _ = join.join();
            Err(anyhow!(
                "audio capture thread exited before reporting readiness"
            ))
        }
    }
}

/// Keep one source's stream alive for as long as the session runs, reopening it
/// against the currently available device whenever it is lost.
fn supervise_source(
    source: CaptureSource,
    out: mpsc::Sender<Vec<f32>>,
    running: Arc<AtomicBool>,
    notify: NoticeFn,
    streaming: Arc<AtomicBool>,
    ready_tx: mpsc::Sender<Result<String, String>>,
) {
    let role = source.role;
    let mut ready_tx = Some(ready_tx);
    let mut current_device: Option<String> = None;
    let mut backoff = REOPEN_BACKOFF;
    let mut initial_attempts = 0u32;
    let mut policy = SwitchPolicy::default();
    // Why the previous stream ended, so the user-facing notice can tell "your
    // headset died" apart from "your headset is now available".
    let mut last_end: Option<StreamEnd> = None;

    while running.load(Ordering::SeqCst) {
        // Set by the cpal error callback to ask for a rebuild.
        let restart = Arc::new(AtomicBool::new(false));
        // Monotonic sample counter, read by the silence watchdog.
        let delivered = Arc::new(AtomicU64::new(0));

        let benched = policy.benched_names(Instant::now());
        let opened = open_stream(
            &source,
            out.clone(),
            restart.clone(),
            delivered.clone(),
            &benched,
        );
        let (stream, device) = match opened {
            Ok(v) => v,
            Err(e) => {
                if ready_tx.is_some() {
                    initial_attempts += 1;
                    if initial_attempts < INITIAL_OPEN_ATTEMPTS {
                        tracing::warn!(
                            "{} capture open attempt {initial_attempts} failed: {e}; retrying",
                            role.describe()
                        );
                        if !sleep_while_running(&running, REOPEN_BACKOFF) {
                            return;
                        }
                        continue;
                    }
                    // Out of attempts — let the caller decide whether this is fatal.
                    tracing::error!("{} capture failed to open: {e}", role.describe());
                    if let Some(tx) = ready_tx.take() {
                        let _ = tx.send(Err(e));
                    }
                    return;
                }
                tracing::warn!("{} capture could not be reopened: {e}", role.describe());
                notify(CaptureNotice::SourceUnavailable {
                    role,
                    reason: e.clone(),
                });
                if !sleep_while_running(&running, backoff) {
                    return;
                }
                backoff = (backoff * 2).min(REOPEN_BACKOFF_MAX);
                continue;
            }
        };
        // Deliberately no `backoff` reset here. A device that is present but dead
        // opens successfully every time, so resetting on open alone made the
        // stall guard never grow — the field symptom was a reopen every 3.4 s for
        // a minute. It is reset below, once audio has actually flowed.
        //
        // Tell the policy where we landed, so a preferred-but-unopenable device
        // is passed over instead of being retried on every re-check.
        policy.record_outcome(&device, Instant::now());
        streaming.store(true, Ordering::SeqCst);

        if let Some(tx) = ready_tx.take() {
            tracing::info!("{} capture opened on \"{device}\"", role.describe());
            let _ = tx.send(Ok(device.clone()));
        } else if current_device.as_deref() == Some(device.as_str()) {
            tracing::info!("{} capture reopened on \"{device}\"", role.describe());
        } else {
            tracing::info!(
                "{} capture switched from {:?} to \"{device}\"",
                role.describe(),
                current_device.as_deref().unwrap_or("<none>")
            );
            notify(CaptureNotice::SwitchedDevice {
                role,
                device: display_device_name(&device).to_string(),
                reason: match last_end {
                    // We chose to move because something better appeared.
                    Some(StreamEnd::Superseded) => SwitchReason::Preferred,
                    // The previous device opened but produced nothing at all.
                    Some(StreamEnd::Stalled {
                        ever_delivered: false,
                    }) => SwitchReason::NoAudio,
                    // The old device errored or went silent under us.
                    _ => SwitchReason::DeviceLost,
                },
            });
        }
        current_device = Some(device.clone());

        let ended = hold_stream(
            &running,
            &restart,
            &delivered,
            &source,
            &device,
            &mut policy,
        );
        drop(stream);
        streaming.store(false, Ordering::SeqCst);
        last_end = Some(ended);

        match ended {
            // Opened fine, delivered nothing. Pass this device over so the reopen
            // lands somewhere that actually works — this is the case that left a
            // recording with no microphone at all for a whole meeting.
            StreamEnd::Stalled {
                ever_delivered: false,
            } => {
                policy.bench(&device, STALL_BENCH_BASE, Instant::now());
                if !sleep_while_running(&running, backoff) {
                    return;
                }
                backoff = (backoff * 2).min(REOPEN_BACKOFF_MAX);
            }
            // Anything else: reopen promptly. If this device actually produced
            // audio it has proved itself, so clear any bench history it carried.
            _ => {
                backoff = REOPEN_BACKOFF;
                if delivered.load(Ordering::Relaxed) > 0 {
                    policy.forgive(&device);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamEnd {
    /// The session is shutting down.
    Stopped,
    /// cpal reported an error that requires a rebuild.
    Errored,
    /// A microphone stopped delivering samples. `ever_delivered` distinguishes a
    /// device that worked and then died — reopen it — from one that never
    /// produced a single sample, which should be passed over instead.
    Stalled { ever_delivered: bool },
    /// A preferable device became available, so this stream is being replaced.
    /// Not a failure — no backoff applies.
    Superseded,
}

/// Decides when a running stream should move to a different device.
///
/// Split out from the capture threads and driven by an injected clock so the
/// debounce and cooldown can be tested directly: a device watcher that
/// oscillates between two devices would be considerably worse than the bug it
/// exists to fix, so this logic is worth testing on its own.
/// A device we should not choose right now.
#[derive(Debug, Clone)]
struct Benched {
    until: Instant,
    /// How many times this device has been benched, so repeat offences bench for
    /// longer.
    strikes: u32,
}

#[derive(Debug, Default)]
struct SwitchPolicy {
    /// The device we have been observing as preferable, and for how many
    /// consecutive checks.
    pending: Option<(String, u32)>,
    /// The device the most recent switch decision asked for, until the caller
    /// reports where it actually landed.
    requested: Option<String>,
    /// Devices to pass over for now: ones we asked for and could not land on, and
    /// ones that opened but delivered no audio.
    benched: HashMap<String, Benched>,
}

impl SwitchPolicy {
    /// Devices that should be excluded from candidate selection right now.
    ///
    /// This has to feed into resolution, not just the switch decision — otherwise
    /// a reopen picks the dead device straight back up as its first candidate.
    fn benched_names(&self, now: Instant) -> Vec<String> {
        self.benched
            .iter()
            .filter(|(_, b)| b.until > now)
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn is_benched(&self, device: &str, now: Instant) -> bool {
        self.benched.get(device).is_some_and(|b| b.until > now)
    }

    /// Bench a device, for longer each time it offends.
    fn bench(&mut self, device: &str, base: Duration, now: Instant) {
        let strikes = self.benched.get(device).map_or(0, |b| b.strikes) + 1;
        let hold = base
            .saturating_mul(1u32 << (strikes - 1).min(8))
            .min(STALL_BENCH_MAX);
        tracing::info!(
            "benching \"{device}\" for {}s (offence {strikes})",
            hold.as_secs()
        );
        self.benched.insert(
            device.to_string(),
            Benched {
                until: now + hold,
                strikes,
            },
        );
    }

    /// Clear a device's bench and strike history. Called only once a stream on it
    /// has actually delivered audio, which is the only real proof it works.
    fn forgive(&mut self, device: &str) {
        self.benched.remove(device);
    }

    /// Record one observation. Returns `true` when the caller should switch.
    fn observe(&mut self, current: &str, desired: Option<&str>, now: Instant) -> bool {
        // Note: expired entries are deliberately *not* pruned here. `is_benched`
        // and `benched_names` already compare against `until`, so an expired
        // entry is inert — but keeping it preserves the strike count, which is
        // what makes the bench lengthen on repeat offences. Pruning it reset the
        // escalation and reintroduced the retry-forever behaviour. The map holds
        // at most one entry per device, so it cannot grow without bound.
        let Some(desired) = desired else {
            self.pending = None;
            return false;
        };
        if desired == current {
            self.pending = None;
            return false;
        }
        if self.is_benched(desired, now) {
            self.pending = None;
            return false;
        }

        match &mut self.pending {
            Some((name, seen)) if name == desired => {
                *seen += 1;
                if *seen >= SWITCH_CONFIRMATIONS {
                    self.pending = None;
                    self.requested = Some(desired.to_string());
                    return true;
                }
            }
            _ => self.pending = Some((desired.to_string(), 1)),
        }
        false
    }

    /// Report where the stream actually landed after a switch. Landing anywhere
    /// other than the device we asked for means that device is not usable right
    /// now, so stop wanting it until the cooldown expires. A no-op when no
    /// switch was requested.
    fn record_outcome(&mut self, landed_on: &str, now: Instant) {
        let Some(wanted) = self.requested.take() else {
            return;
        };
        if wanted == landed_on {
            // Deliberately does *not* clear the bench. A present-but-dead device
            // opens successfully every time, so clearing on a successful open
            // would reset its strike count and the escalation below would never
            // build — it would be retried roughly every minute forever, costing
            // the user `FIRST_SAMPLES_GRACE` of microphone each time. Only
            // actually delivering audio earns forgiveness; see `forgive`.
        } else {
            tracing::info!(
                "preferred device \"{wanted}\" could not be opened (landed on \"{landed_on}\")"
            );
            self.bench(&wanted, SWITCH_COOLDOWN, now);
        }
    }
}

/// Block while the stream is healthy. Returns why it stopped being healthy.
///
/// Three things can end a stream: cpal reporting an error, a microphone going
/// silent, or a preferable device appearing. The last is what makes the
/// recording follow a headset being plugged in or the OS default moving.
fn hold_stream(
    running: &AtomicBool,
    restart: &AtomicBool,
    delivered: &AtomicU64,
    source: &CaptureSource,
    current_device: &str,
    policy: &mut SwitchPolicy,
) -> StreamEnd {
    let mut last_count = delivered.load(Ordering::Relaxed);
    let mut last_progress = Instant::now();
    let mut last_recheck = Instant::now();
    // Until the first samples arrive we are patient, because a Bluetooth device
    // may still be renegotiating its profile. After that, silence means loss.
    let mut ever_delivered = last_count > 0;

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
        if restart.load(Ordering::SeqCst) {
            return StreamEnd::Errored;
        }

        if last_recheck.elapsed() >= DEVICE_RECHECK_INTERVAL {
            last_recheck = Instant::now();
            let now = Instant::now();
            let desired = preferred_device_name(source, &policy.benched_names(now));
            if policy.observe(current_device, desired.as_deref(), now) {
                tracing::info!(
                    "{} capture: \"{}\" is now preferred over \"{current_device}\"; switching",
                    source.role.describe(),
                    desired.as_deref().unwrap_or("<none>")
                );
                return StreamEnd::Superseded;
            }
        }

        // Only a microphone can be judged dead by silence: an idle system-audio
        // loopback delivers no callbacks at all, which is normal.
        if source.role != SourceRole::Microphone {
            continue;
        }
        let count = delivered.load(Ordering::Relaxed);
        if count != last_count {
            last_count = count;
            last_progress = Instant::now();
            ever_delivered = true;
        } else {
            let limit = if ever_delivered {
                MIC_STALL_TIMEOUT
            } else {
                FIRST_SAMPLES_GRACE
            };
            if last_progress.elapsed() >= limit {
                if ever_delivered {
                    tracing::warn!(
                        "microphone \"{current_device}\" stopped delivering audio after {}s; reopening",
                        limit.as_secs()
                    );
                } else {
                    tracing::warn!(
                        "microphone \"{current_device}\" delivered no audio at all in {}s; \
                         passing it over",
                        limit.as_secs()
                    );
                }
                return StreamEnd::Stalled { ever_delivered };
            }
        }
    }
    StreamEnd::Stopped
}

/// The device this source would open if it opened right now — i.e. the best
/// candidate. Comparing this against the device actually in use is what detects
/// a plugged-in headset, a moved OS default, or a configured device returning.
fn preferred_device_name(source: &CaptureSource, excluded: &[String]) -> Option<String> {
    let host = cpal::default_host();
    resolve_candidates(&host, source, excluded)
        .into_iter()
        .next()
        .map(|c| c.name)
}

/// Sleep, waking early if the session is stopped. Returns `false` if it was.
fn sleep_while_running(running: &AtomicBool, total: Duration) -> bool {
    let step = Duration::from_millis(50);
    let mut slept = Duration::ZERO;
    while slept < total {
        if !running.load(Ordering::SeqCst) {
            return false;
        }
        thread::sleep(step);
        slept += step;
    }
    running.load(Ordering::SeqCst)
}

enum DeviceCaptureKind {
    Input,
    WasapiLoopback,
}

struct Candidate {
    device: cpal::Device,
    capture: DeviceCaptureKind,
    /// The stored device name, i.e. loopback candidates keep their
    /// `wasapi-loopback::` prefix so it round-trips with settings.
    name: String,
}

/// Open a source's stream, trying each candidate device in preference order.
fn open_stream(
    source: &CaptureSource,
    out: mpsc::Sender<Vec<f32>>,
    restart: Arc<AtomicBool>,
    delivered: Arc<AtomicU64>,
    excluded: &[String],
) -> Result<(cpal::Stream, String), String> {
    let host = cpal::default_host();
    let candidates = resolve_candidates(&host, source, excluded);

    if let Some(configured) = source.device.as_deref() {
        if !configured.is_empty()
            && configured != "default"
            && !candidates.iter().any(|c| c.name == configured)
        {
            tracing::warn!(
                "configured {} device \"{}\" is not available; falling back to the default",
                source.role.describe(),
                display_device_name(configured)
            );
        }
    }

    if candidates.is_empty() {
        // The role prefix is added by `CaptureNotice::message`, so these read as
        // the *reason* rather than repeating "system audio unavailable".
        return Err(match (&source.device, source.role) {
            (Some(name), _) => format!(
                "\"{}\" is not connected and no fallback device is available",
                display_device_name(name)
            ),
            (None, SourceRole::SystemAudio) => missing_system_audio_hint().to_string(),
            (None, SourceRole::Microphone) => "no microphone is available".to_string(),
        });
    }

    let mut last_err = None;
    for candidate in candidates {
        match build_stream(&candidate, out.clone(), restart.clone(), delivered.clone()) {
            Ok(stream) => return Ok((stream, candidate.name)),
            Err(e) => {
                tracing::debug!("capture candidate \"{}\" rejected: {e}", candidate.name);
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| "no usable audio device".to_string()))
}

/// What to tell the user when there is no system-audio source at all.
///
/// Worth spelling out because it is the one configuration that cannot work out
/// of the box: Windows has WASAPI loopback and Linux exposes monitor sources, but
/// macOS has no user-space loopback without a virtual audio driver. Since system
/// audio capture is on by default, a Mac without one would otherwise just report
/// "unavailable" every recording with no hint about what to do.
fn missing_system_audio_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS needs a virtual audio driver to capture system audio. Install one (e.g. BlackHole), then choose it in Settings."
    } else if cfg!(target_os = "linux") {
        "no monitor source found. With PipeWire or PulseAudio, enable a \"Monitor of …\" source, then choose it in Settings."
    } else {
        "no playback device was found to capture from."
    }
}

/// Devices to try for a source, best first.
///
/// The OS default endpoint is deliberately resolved to a **concrete** device
/// before cpal's own default-device handle is considered. cpal activates its
/// default handle through `ActivateAudioInterfaceAsync`, which has been observed
/// to fail with `RPC_E_CHANGED_MODE` ("Cannot change thread mode after it is
/// set", os error -2147417850) inside the Tauri process; a named device goes
/// through `IMMDevice::Activate` instead and does not hit that path. cpal's
/// default handle is kept as a fallback because it is the only one that
/// auto-reroutes at the WASAPI level, but this module supervises reopening
/// itself, so nothing depends on that.
/// `excluded` names are filtered out — devices that opened but delivered nothing,
/// or that we asked for and could not reach. If filtering would leave nothing at
/// all the exclusions are ignored, because some device beats no device.
fn resolve_candidates(
    host: &cpal::Host,
    source: &CaptureSource,
    excluded: &[String],
) -> Vec<Candidate> {
    let all = resolve_all_candidates(host, source);
    if excluded.is_empty() {
        return all;
    }
    let keep = |c: &Candidate| !excluded.iter().any(|e| e == &c.name);
    if !all.iter().any(keep) {
        tracing::warn!(
            "every {} candidate is benched; ignoring the bench rather than losing the source",
            source.role.describe()
        );
        return all;
    }
    all.into_iter().filter(|c| keep(c)).collect()
}

fn resolve_all_candidates(host: &cpal::Host, source: &CaptureSource) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = Vec::new();

    // 1. The device the user actually chose, if it is still connected.
    //    Deliberately silent: this runs on every device re-check while
    //    recording, so `open_stream` does the logging instead.
    if let Some(stored) = source.device.as_deref() {
        if !stored.is_empty() && stored != "default" {
            if let Some(c) = named_candidate(host, stored) {
                out.push(c);
            }
        }
    }

    match source.role {
        SourceRole::Microphone => push_microphone_candidates(host, &mut out),
        SourceRole::SystemAudio => push_system_audio_candidates(host, &mut out),
    }

    out
}

fn push_microphone_candidates(host: &cpal::Host, out: &mut Vec<Candidate>) {
    // Resolved first so cpal has initialised COM on this thread before
    // `communications_capture_name` uses it (it returns `None` rather than
    // failing if COM is not ready, so the ordering is belt-and-braces).
    let default_name = host.default_input_device().and_then(|d| device_name(&d));

    // Windows' Default *Communication* Device, which is what a headset gets and
    // what every call app uses, ahead of the plain default. Only microphones get
    // this preference — see `push_system_audio_candidates` for why loopback does
    // not.
    if let Some(name) = communications_capture_name() {
        if default_name.as_deref() != Some(name.as_str()) {
            tracing::debug!("preferring communications capture endpoint \"{name}\"");
        }
        push_unique(out, named_candidate(host, &name));
    }

    // The current default endpoint, resolved to a concrete device.
    if let Some(name) = default_name.as_deref() {
        push_unique(out, named_candidate(host, name));
    }
    // cpal's default handle. Deliberately not deduplicated against the entry
    // above: it is the same endpoint reached through a different activation
    // path, which is the point of having it.
    if let Some(device) = host.default_input_device() {
        out.push(Candidate {
            device,
            capture: DeviceCaptureKind::Input,
            name: default_name.unwrap_or_else(|| "default input".to_string()),
        });
    }
    // Any other real microphone, so a recording survives an unplugged device
    // even when the OS has not nominated a replacement yet.
    if let Ok(devices) = host.input_devices() {
        for d in devices {
            let Some(name) = device_name(&d) else {
                continue;
            };
            // Never fall back onto a monitor/loopback source for a microphone —
            // that would silently record the machine's output as "your voice".
            if classify_device(&name) == AudioDeviceKind::Loopback {
                continue;
            }
            push_unique(
                out,
                Some(Candidate {
                    device: d,
                    capture: DeviceCaptureKind::Input,
                    name,
                }),
            );
        }
    }
}

/// Friendly name of the OS "communications" capture endpoint, or `None` where the
/// platform has no such concept (everywhere except Windows) or on any failure.
///
/// Failing softly is deliberate: the caller falls back to cpal's console default,
/// which is exactly today's behaviour, so this can never make device selection
/// worse than it already was.
#[cfg(target_os = "windows")]
fn communications_capture_name() -> Option<String> {
    communications_endpoint_name(windows::Win32::Media::Audio::eCapture)
}

#[cfg(not(target_os = "windows"))]
fn communications_capture_name() -> Option<String> {
    None
}

/// Make sure COM is initialised on this thread before we use Core Audio, by
/// making a cheap cpal call that does it.
///
/// Deliberately indirect. cpal owns COM lifetime for the process — it chooses the
/// apartment model, tolerates `RPC_E_CHANGED_MODE`, and holds a thread-local
/// guard that calls `CoUninitialize` on thread exit. Calling `CoInitializeEx`
/// ourselves alongside that is precisely how this codebase produced a
/// `RPC_E_CHANGED_MODE` failure before, so keeping exactly one owner is worth the
/// inelegance. Without this the endpoint lookup below silently returns `None`,
/// which a test caught.
#[cfg(target_os = "windows")]
fn ensure_com_initialized() {
    let _ = cpal::default_host()
        .default_input_device()
        .and_then(|d| d.description().ok());
}

#[cfg(target_os = "windows")]
fn communications_endpoint_name(flow: windows::Win32::Media::Audio::EDataFlow) -> Option<String> {
    ensure_com_initialized();

    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Devices::Properties::DEVPKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{eCommunications, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, STGM_READ};
    use windows::Win32::System::Variant::VT_LPWSTR;

    // SAFETY: every call is checked; the PROPVARIANT is only read as a wide
    // string after its discriminant is confirmed to be VT_LPWSTR, and the string
    // is copied out before the variant is dropped.
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).ok()?;
        let device = enumerator
            .GetDefaultAudioEndpoint(flow, eCommunications)
            .ok()?;
        let store = device.OpenPropertyStore(STGM_READ).ok()?;
        let value = store
            .GetValue(&DEVPKEY_Device_FriendlyName as *const _ as *const _)
            .ok()?;

        let variant = &value.Anonymous.Anonymous;
        if variant.vt != VT_LPWSTR {
            return None;
        }
        let ptr = *(&variant.Anonymous as *const _ as *const *const u16);
        if ptr.is_null() {
            return None;
        }
        // Bound the scan in case the string is not null-terminated.
        const MAX_CHARS: usize = 32_768;
        let mut len = 0usize;
        while len < MAX_CHARS && *ptr.add(len) != 0 {
            len += 1;
        }
        if len == 0 || len >= MAX_CHARS {
            return None;
        }
        let wide = std::slice::from_raw_parts(ptr, len);
        Some(OsString::from_wide(wide).to_string_lossy().into_owned())
    }
}

/// On Windows an output device used as an input transparently captures loopback,
/// so system audio comes from the render endpoints.
///
/// Note this deliberately does **not** prefer the communications render endpoint
/// the way the microphone path does. The failure modes are asymmetric: picking
/// the wrong *capture* endpoint records the wrong microphone, which is audible,
/// whereas picking the wrong *render* endpoint records nothing at all — a
/// loopback capture on an endpoint with no active stream produces no callbacks —
/// and silence from loopback is indistinguishable from "nothing is playing". The
/// console default is where general audio goes, so it is the safer guess. See
/// SYSTEM_OVERVIEW.md for the known limitation when the two defaults differ.
#[cfg(target_os = "windows")]
fn push_system_audio_candidates(host: &cpal::Host, out: &mut Vec<Candidate>) {
    let default_name = host.default_output_device().and_then(|d| device_name(&d));

    if let Some(name) = default_name.as_deref() {
        push_unique(out, named_candidate(host, &wasapi_loopback_id(name)));
    }
    if let Some(device) = host.default_output_device() {
        out.push(Candidate {
            device,
            capture: DeviceCaptureKind::WasapiLoopback,
            name: default_name
                .as_deref()
                .map(wasapi_loopback_id)
                .unwrap_or_else(|| wasapi_loopback_id("default output")),
        });
    }
    if let Ok(devices) = host.output_devices() {
        for d in devices {
            let Some(raw) = device_name(&d) else { continue };
            push_unique(
                out,
                Some(Candidate {
                    device: d,
                    capture: DeviceCaptureKind::WasapiLoopback,
                    name: wasapi_loopback_id(&raw),
                }),
            );
        }
    }
}

/// Elsewhere, system audio has to come from a monitor / virtual-loopback *input*
/// source (PipeWire and PulseAudio expose "Monitor of …"; macOS needs a driver
/// such as BlackHole). An output endpoint cannot be captured directly.
#[cfg(not(target_os = "windows"))]
fn push_system_audio_candidates(host: &cpal::Host, out: &mut Vec<Candidate>) {
    if let Ok(devices) = host.input_devices() {
        for d in devices {
            let Some(name) = device_name(&d) else {
                continue;
            };
            if classify_device(&name) != AudioDeviceKind::Loopback {
                continue;
            }
            push_unique(
                out,
                Some(Candidate {
                    device: d,
                    capture: DeviceCaptureKind::Input,
                    name,
                }),
            );
        }
    }
}

fn push_unique(out: &mut Vec<Candidate>, candidate: Option<Candidate>) {
    if let Some(c) = candidate {
        if !out.iter().any(|existing| existing.name == c.name) {
            out.push(c);
        }
    }
}

/// Look up a stored device name among the host's devices.
fn named_candidate(host: &cpal::Host, stored: &str) -> Option<Candidate> {
    if is_wasapi_loopback_id(stored) {
        let raw = wasapi_loopback_device_name(stored);
        let devices = host.output_devices().ok()?;
        for d in devices {
            if device_name(&d).as_deref() == Some(raw) {
                return Some(Candidate {
                    device: d,
                    capture: DeviceCaptureKind::WasapiLoopback,
                    name: stored.to_string(),
                });
            }
        }
        return None;
    }
    let devices = host.input_devices().ok()?;
    for d in devices {
        if device_name(&d).as_deref() == Some(stored) {
            return Some(Candidate {
                device: d,
                capture: DeviceCaptureKind::Input,
                name: stored.to_string(),
            });
        }
    }
    None
}

fn build_stream(
    candidate: &Candidate,
    out: mpsc::Sender<Vec<f32>>,
    restart: Arc<AtomicBool>,
    delivered: Arc<AtomicU64>,
) -> Result<cpal::Stream, String> {
    let device = &candidate.device;
    let config = match candidate.capture {
        DeviceCaptureKind::Input => device.default_input_config(),
        // A WASAPI output device used as an input transparently captures
        // loopback, and its config comes from the render side.
        DeviceCaptureKind::WasapiLoopback => device.default_output_config(),
    }
    .map_err(|e| format!("failed to read audio config: {e}"))?;

    let src_rate = config.sample_rate();
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();
    let label = candidate.name.clone();

    let err_fn = move |err: cpal::Error| {
        // RealtimeDenied only means the audio thread did not get real-time
        // priority — the stream is still delivering, so don't rebuild for it.
        // Everything else (DeviceChanged when WASAPI reroutes the default
        // endpoint, DeviceNotAvailable / StreamInvalidated when it disappears)
        // leaves cpal's run loop waiting on a handle that will never fire again,
        // so the stream has to be rebuilt.
        if err.kind() == cpal::ErrorKind::RealtimeDenied {
            tracing::debug!("audio stream on \"{label}\": {err}");
            return;
        }
        tracing::warn!(
            "audio stream error on \"{label}\" (kind={:?}): {err}",
            err.kind()
        );
        restart.store(true, Ordering::SeqCst);
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            config.into(),
            move |data: &[f32], _| feed(data, channels, src_rate, |s| s, &out, &delivered),
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            config.into(),
            move |data: &[i16], _| {
                feed(
                    data,
                    channels,
                    src_rate,
                    |s| s as f32 / i16::MAX as f32,
                    &out,
                    &delivered,
                )
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            config.into(),
            move |data: &[u16], _| {
                feed(
                    data,
                    channels,
                    src_rate,
                    |s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0,
                    &out,
                    &delivered,
                )
            },
            err_fn,
            None,
        ),
        other => return Err(format!("unsupported sample format: {other:?}")),
    }
    .map_err(|e| format!("failed to build input stream: {e}"))?;

    stream
        .play()
        .map_err(|e| format!("failed to start input stream: {e}"))?;
    Ok(stream)
}

/// Downmix to mono, resample to [`TARGET_SAMPLE_RATE`], and forward the batch.
///
/// Resampling here rather than downstream is what lets a device be swapped
/// mid-recording: the consumer only ever sees 16 kHz mono.
fn feed<T: Copy>(
    data: &[T],
    channels: usize,
    src_rate: u32,
    convert: impl Fn(T) -> f32,
    tx: &mpsc::Sender<Vec<f32>>,
    delivered: &AtomicU64,
) {
    let mono = to_mono(data, channels, convert);
    if mono.is_empty() {
        return;
    }
    // Count pre-resample so the watchdog measures the device, not the maths.
    delivered.fetch_add(mono.len() as u64, Ordering::Relaxed);
    let batch = resample(&mono, src_rate, TARGET_SAMPLE_RATE);
    if batch.is_empty() {
        return;
    }
    let _ = tx.send(batch);
}

/// Collapse interleaved multi-channel frames into mono.
fn to_mono<T: Copy>(data: &[T], channels: usize, convert: impl Fn(T) -> f32) -> Vec<f32> {
    if channels <= 1 {
        return data.iter().map(|&s| convert(s)).collect();
    }
    let mut mono = Vec::with_capacity(data.len() / channels + 1);
    for frame in data.chunks(channels) {
        let mut sum = 0.0f32;
        for &s in frame {
            sum += convert(s);
        }
        mono.push(sum / frame.len() as f32);
    }
    mono
}

/// Naive linear resampler from `src_rate` to `dst_rate` (mono).
pub fn resample(samples: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if src_rate == dst_rate || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = dst_rate as f64 / src_rate as f64;
    let out_len = ((samples.len() as f64) * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos.floor() as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

/// Encode mono `f32` samples to an in-memory 16 kHz 16-bit WAV file.
pub fn encode_wav_16k(samples: &[f32], src_rate: u32) -> Result<Vec<u8>> {
    let resampled = resample(samples, src_rate, TARGET_SAMPLE_RATE);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for &s in &resampled {
            let clamped = s.clamp(-1.0, 1.0);
            writer.write_sample((clamped * i16::MAX as f32) as i16)?;
        }
        writer.finalize()?;
    }
    Ok(cursor.into_inner())
}

/// Root-mean-square loudness of a buffer, in [0, 1].
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Rough loudness gate so we don't spend API calls transcribing silence.
pub fn is_mostly_silent(samples: &[f32]) -> bool {
    rms(samples) < 0.004
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_loopback_names() {
        assert_eq!(
            classify_device("Monitor of Built-in Audio Analog Stereo"),
            AudioDeviceKind::Loopback
        );
        assert_eq!(
            classify_device("Stereo Mix (Realtek Audio)"),
            AudioDeviceKind::Loopback
        );
    }

    #[test]
    fn classifies_microphone_names() {
        assert_eq!(
            classify_device("MacBook Pro Microphone"),
            AudioDeviceKind::Microphone
        );
    }

    #[test]
    fn classifies_wasapi_loopback_ids() {
        let id = wasapi_loopback_id("Speakers (Realtek Audio)");
        assert!(is_wasapi_loopback_id(&id));
        assert_eq!(classify_device(&id), AudioDeviceKind::Loopback);
    }

    #[test]
    fn to_mono_averages_channels() {
        // Two interleaved stereo frames: (1.0, 0.0) and (0.0, 1.0).
        let data = [1.0f32, 0.0, 0.0, 1.0];
        assert_eq!(to_mono(&data, 2, |s| s), vec![0.5, 0.5]);
    }

    #[test]
    fn resample_halves_length_when_halving_rate() {
        let samples: Vec<f32> = (0..100).map(|i| i as f32).collect();
        assert_eq!(resample(&samples, 32_000, 16_000).len(), 50);
    }

    /// Drain a capture session for `secs`, reporting the sample count, the peak
    /// batch loudness (so a run with audio playing is distinguishable from a
    /// silent one), and any notices raised.
    #[cfg(test)]
    fn drain_for(sources: Vec<CaptureSource>, secs: u64) -> (usize, f32, Vec<String>) {
        use std::sync::Mutex;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<f32>>();
        let notices = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = notices.clone();
        let session = start_capture(
            sources,
            tx,
            Arc::new(move |n: CaptureNotice| sink.lock().unwrap().push(n.message())),
        )
        .expect("capture should start");

        let mut samples = 0usize;
        let mut peak = 0.0f32;
        let deadline = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < deadline {
            while let Ok(batch) = rx.try_recv() {
                samples += batch.len();
                peak = peak.max(rms(&batch));
            }
            thread::sleep(Duration::from_millis(50));
        }
        session.stop();
        let collected = notices.lock().unwrap().clone();
        (samples, peak, collected)
    }

    /// Diagnostic: characterise what a microphone is actually delivering, to tell
    /// "no callbacks at all" apart from "callbacks carrying digital silence".
    /// Those need different detection and only one of them is currently caught.
    #[test]
    #[ignore = "diagnostic; requires a real audio input device"]
    fn characterise_each_microphone() {
        let host = cpal::default_host();
        let mut names: Vec<String> = Vec::new();
        if let Ok(devices) = host.input_devices() {
            for d in devices {
                if let Some(n) = device_name(&d) {
                    if classify_device(&n) != AudioDeviceKind::Loopback {
                        names.push(n);
                    }
                }
            }
        }
        println!("inputs: {names:?}\n");

        for name in names {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<f32>>();
            let notify: NoticeFn = Arc::new(|_| {});
            let session = match start_capture(
                vec![CaptureSource::microphone(Some(name.clone()))],
                tx,
                notify,
            ) {
                Ok(s) => s,
                Err(e) => {
                    println!("{name:<60} could not open: {e}");
                    continue;
                }
            };

            let mut total = 0usize;
            let mut zeros = 0usize;
            let mut peak = 0.0f32;
            let mut sum_sq = 0.0f64;
            let deadline = Instant::now() + Duration::from_secs(4);
            while Instant::now() < deadline {
                while let Ok(batch) = rx.try_recv() {
                    for s in &batch {
                        total += 1;
                        if *s == 0.0 {
                            zeros += 1;
                        }
                        peak = peak.max(s.abs());
                        sum_sq += (*s as f64) * (*s as f64);
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
            session.stop();

            let rms = if total > 0 {
                (sum_sq / total as f64).sqrt()
            } else {
                0.0
            };
            let zero_pct = if total > 0 {
                zeros as f64 * 100.0 / total as f64
            } else {
                0.0
            };
            println!(
                "{name:<60} samples={total:<8} exact-zero={zero_pct:5.1}%  peak={peak:.6}  rms={rms:.6}"
            );
        }
    }

    /// Needs a real capture device, so it is `#[ignore]`d in normal runs.
    /// `cargo test --lib -- --ignored --nocapture`
    #[test]
    #[ignore = "requires a real audio input device"]
    fn captures_from_the_default_microphone_at_the_target_rate() {
        let (samples, peak, notices) = drain_for(vec![CaptureSource::microphone(None)], 3);
        println!("microphone: {samples} samples, peak RMS {peak:.4}, notices: {notices:?}");
        // 3 s at 16 kHz is ~48000; allow generous slack for open latency.
        assert!(
            samples > TARGET_SAMPLE_RATE as usize,
            "expected over a second of 16 kHz audio, got {samples} samples"
        );
    }

    /// A named device that does not exist must fall back to the current default
    /// rather than failing — this is the path an unplugged headset takes.
    #[test]
    #[ignore = "requires a real audio input device"]
    fn falls_back_to_the_default_when_the_named_device_is_gone() {
        let (samples, peak, notices) = drain_for(
            vec![CaptureSource::microphone(Some(
                "No Such Device (definitely not connected)".into(),
            ))],
            3,
        );
        println!("fallback: {samples} samples, peak RMS {peak:.4}, notices: {notices:?}");
        assert!(
            samples > TARGET_SAMPLE_RATE as usize,
            "expected capture to fall back to the default device, got {samples} samples"
        );
    }

    /// System audio on its own. Requires something to actually be playing:
    /// a WASAPI loopback capture on an idle output device delivers nothing at
    /// all, so with silence this correctly reports zero samples.
    #[test]
    #[ignore = "requires an output device with audio actively playing"]
    fn system_audio_only_captures_what_is_playing() {
        let (samples, peak, notices) = drain_for(vec![CaptureSource::system_audio(None)], 3);
        println!("system audio: {samples} samples, peak RMS {peak:.4}, notices: {notices:?}");
        assert!(
            samples > TARGET_SAMPLE_RATE as usize,
            "expected over a second of loopback audio — is anything playing? got {samples} samples"
        );
        assert!(
            peak > 0.001,
            "loopback audio was silent (peak RMS {peak:.4})"
        );
    }

    /// The regression this guards: an idle system-audio loopback delivers no
    /// callbacks at all, and must not stall the microphone it is mixed with.
    #[test]
    #[ignore = "requires real audio input and output devices"]
    fn idle_system_audio_does_not_stall_the_microphone() {
        let (samples, peak, notices) = drain_for(
            vec![
                CaptureSource::microphone(None),
                CaptureSource::system_audio(None),
            ],
            3,
        );
        println!("mic + system audio: {samples} samples, peak RMS {peak:.4}, notices: {notices:?}");
        assert!(
            samples > TARGET_SAMPLE_RATE as usize,
            "microphone audio should keep flowing while system audio is idle, got {samples} samples"
        );
    }

    // ---- switch policy ----
    //
    // A device watcher that oscillates between two devices would be worse than
    // the bug it fixes, so the debounce and the cooldown are tested directly
    // rather than only through the capture threads.

    #[test]
    fn switch_policy_needs_repeated_agreement_before_switching() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();
        // One sighting is not enough — enumeration is briefly inconsistent while
        // Windows brings an endpoint up.
        assert!(!p.observe("built-in", Some("headset"), t));
        assert!(p.observe("built-in", Some("headset"), t));
    }

    #[test]
    fn switch_policy_ignores_a_flapping_observation() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();
        assert!(!p.observe("built-in", Some("headset"), t));
        // A different device appears next tick, resetting the count.
        assert!(!p.observe("built-in", Some("dock"), t));
        assert!(!p.observe("built-in", Some("headset"), t));
        assert!(p.observe("built-in", Some("headset"), t));
    }

    #[test]
    fn switch_policy_stays_put_when_already_on_the_preferred_device() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();
        for _ in 0..5 {
            assert!(!p.observe("headset", Some("headset"), t));
        }
        // And when there is no candidate at all.
        for _ in 0..5 {
            assert!(!p.observe("headset", None, t));
        }
    }

    #[test]
    fn switch_policy_cools_down_a_device_it_could_not_open() {
        let mut p = SwitchPolicy::default();
        let t0 = Instant::now();

        assert!(!p.observe("built-in", Some("headset"), t0));
        assert!(p.observe("built-in", Some("headset"), t0));
        // The switch was attempted but landed back on the built-in mic — the
        // headset is preferred yet unopenable, e.g. held by another app.
        p.record_outcome("built-in", t0);

        // Without a cooldown this is where it would thrash: tear down a working
        // stream every re-check, forever.
        for _ in 0..10 {
            assert!(
                !p.observe("built-in", Some("headset"), t0),
                "must not retry a device that just failed"
            );
        }

        // Once the cooldown expires it becomes eligible again.
        let later = t0 + SWITCH_COOLDOWN + Duration::from_millis(1);
        assert!(!p.observe("built-in", Some("headset"), later));
        assert!(p.observe("built-in", Some("headset"), later));
    }

    #[test]
    fn switch_policy_clears_cooldown_when_the_switch_succeeds() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();
        assert!(!p.observe("built-in", Some("headset"), t));
        assert!(p.observe("built-in", Some("headset"), t));
        p.record_outcome("headset", t); // landed where we wanted
        assert!(!p.is_benched("headset", t));
    }

    /// Landing on the requested device is not proof it works — a dead endpoint
    /// opens successfully every time. Only delivering audio clears the record, so
    /// escalation can actually build instead of resetting every cycle.
    #[test]
    fn a_successful_open_alone_does_not_clear_the_bench() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();
        p.bench("headset", STALL_BENCH_BASE, t);
        let after = t + STALL_BENCH_BASE + Duration::from_millis(1);

        // Switch back to it once the bench lapses, and land on it.
        assert!(!p.observe("built-in", Some("headset"), after));
        assert!(p.observe("built-in", Some("headset"), after));
        p.record_outcome("headset", after);

        // It still carries a strike, so benching again holds for longer.
        p.bench("headset", STALL_BENCH_BASE, after);
        assert!(
            p.is_benched("headset", after + STALL_BENCH_BASE + Duration::from_secs(1)),
            "strike history must survive a merely-successful open"
        );

        // Delivering audio is what clears it.
        p.forgive("headset");
        assert!(!p.is_benched("headset", after));
    }

    #[test]
    fn switch_policy_outcome_without_a_request_is_a_no_op() {
        let mut p = SwitchPolicy::default();
        p.record_outcome("built-in", Instant::now());
        assert!(p.benched.is_empty());
        assert!(p.requested.is_none());
    }

    /// The field failure: a Bluetooth headset mic endpoint that opens fine and
    /// delivers nothing. It must be excluded from selection, not reopened.
    #[test]
    fn benched_device_is_excluded_from_selection() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();
        p.bench("headset", STALL_BENCH_BASE, t);

        assert!(p.is_benched("headset", t));
        assert_eq!(p.benched_names(t), vec!["headset".to_string()]);
        // And it is not chosen as a switch target while benched.
        assert!(!p.observe("built-in", Some("headset"), t));
        assert!(!p.observe("built-in", Some("headset"), t));
    }

    #[test]
    fn bench_expires_and_lengthens_on_repeat_offences() {
        let mut p = SwitchPolicy::default();
        let t = Instant::now();

        p.bench("headset", STALL_BENCH_BASE, t);
        let just_after_first = t + STALL_BENCH_BASE + Duration::from_millis(1);
        assert!(
            !p.is_benched("headset", just_after_first),
            "first bench expires"
        );

        // Second offence must hold for longer than the first, so a dead device
        // stops costing the user audio every time the bench lapses.
        p.bench("headset", STALL_BENCH_BASE, just_after_first);
        assert!(
            p.is_benched(
                "headset",
                just_after_first + STALL_BENCH_BASE + Duration::from_secs(1)
            ),
            "second bench should outlast the first"
        );
    }

    #[test]
    fn bench_is_capped() {
        let mut p = SwitchPolicy::default();
        let mut t = Instant::now();
        for _ in 0..20 {
            p.bench("headset", STALL_BENCH_BASE, t);
            t += Duration::from_secs(1);
        }
        // Never bench beyond the cap, so a device can always eventually recover.
        assert!(!p.is_benched("headset", t + STALL_BENCH_MAX + Duration::from_secs(1)));
    }

    #[test]
    fn candidate_exclusion_is_ignored_rather_than_losing_the_source() {
        // Excluding every candidate must fall back to the unfiltered list: some
        // device beats no device.
        let host = cpal::default_host();
        let source = CaptureSource::microphone(None);
        let all = resolve_all_candidates(&host, &source);
        if all.is_empty() {
            return; // no audio hardware in this environment
        }
        let everything: Vec<String> = all.iter().map(|c| c.name.clone()).collect();
        let still = resolve_candidates(&host, &source, &everything);
        assert_eq!(
            still.len(),
            all.len(),
            "excluding everything should ignore the exclusions"
        );
    }

    /// Steady state must produce no switches at all. This is the regression that
    /// would hurt most: a re-check loop that keeps deciding a different device is
    /// better would tear down the stream every 750 ms for the whole meeting.
    #[test]
    #[ignore = "requires a real audio input device"]
    fn steady_state_does_not_switch_devices() {
        let (samples, peak, notices) = drain_for(vec![CaptureSource::microphone(None)], 6);
        println!("steady state: {samples} samples, peak RMS {peak:.4}, notices: {notices:?}");
        assert!(
            notices.is_empty(),
            "capture should not switch devices when nothing changed, got: {notices:?}"
        );
        // 6 s of re-checks (8 of them) must not have interrupted capture either.
        assert!(
            samples > 5 * TARGET_SAMPLE_RATE as usize,
            "expected roughly 6s of continuous audio, got {samples} samples"
        );
    }

    /// Same, with both sources running, since each supervises independently.
    #[test]
    #[ignore = "requires real audio input and output devices"]
    fn steady_state_does_not_switch_with_both_sources() {
        let (samples, peak, notices) = drain_for(
            vec![
                CaptureSource::microphone(None),
                CaptureSource::system_audio(None),
            ],
            6,
        );
        println!(
            "steady state (both): {samples} samples, peak RMS {peak:.4}, notices: {notices:?}"
        );
        assert!(
            notices.is_empty(),
            "neither source should switch when nothing changed, got: {notices:?}"
        );
    }

    /// Confirms the Core Audio call actually resolves, rather than failing softly
    /// and leaving the communications preference as dead code.
    #[test]
    #[ignore = "requires a real audio input device"]
    fn communications_capture_endpoint_resolves() {
        let comms = communications_capture_name();
        let console = cpal::default_host()
            .default_input_device()
            .and_then(|d| device_name(&d));
        println!("communications capture endpoint: {comms:?}");
        println!("console capture endpoint:        {console:?}");
        #[cfg(target_os = "windows")]
        assert!(
            comms.is_some(),
            "the communications endpoint lookup returned None — the preference would be dead code"
        );
        if comms != console {
            println!("NOTE: the two defaults differ; this is the case the preference exists for");
        }
    }

    #[test]
    #[ignore = "requires a real audio input device"]
    fn preferred_device_is_stable_across_repeated_resolution() {
        // The re-check compares this against the device in use, so if it is not
        // deterministic the capture layer would switch on its own noise.
        let source = CaptureSource::microphone(None);
        let first = preferred_device_name(&source, &[]);
        println!("preferred microphone: {first:?}");
        assert!(first.is_some(), "expected a preferred microphone");
        for _ in 0..8 {
            assert_eq!(preferred_device_name(&source, &[]), first);
        }
    }

    /// Benching the preferred microphone must yield a *different* device, which is
    /// what lets a dead Bluetooth mic fall through to the built-in one.
    #[test]
    #[ignore = "requires at least two real audio input devices"]
    fn benching_the_preferred_microphone_yields_another_one() {
        let source = CaptureSource::microphone(None);
        let Some(first) = preferred_device_name(&source, &[]) else {
            return;
        };
        let next = preferred_device_name(&source, std::slice::from_ref(&first));
        println!("preferred: {first:?}  after benching it: {next:?}");
        match next {
            Some(other) => assert_ne!(other, first, "benched device must not be reselected"),
            None => println!("only one input device on this machine; nothing to fall back to"),
        }
    }

    #[test]
    fn notice_messages_name_the_source_and_the_reason() {
        let n = CaptureNotice::SwitchedDevice {
            role: SourceRole::Microphone,
            device: "Built-in Mic".into(),
            reason: SwitchReason::DeviceLost,
        };
        assert_eq!(
            n.message(),
            "microphone disconnected — switched to Built-in Mic"
        );
        let n = CaptureNotice::SwitchedDevice {
            role: SourceRole::Microphone,
            device: "Jabra Evolve2".into(),
            reason: SwitchReason::Preferred,
        };
        assert_eq!(n.message(), "microphone switched to Jabra Evolve2");
        let n = CaptureNotice::SourceUnavailable {
            role: SourceRole::SystemAudio,
            reason: "no output device".into(),
        };
        assert_eq!(n.message(), "system audio unavailable: no output device");
    }
}
