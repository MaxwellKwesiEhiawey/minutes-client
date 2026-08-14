use crate::locking::MutexExt;
use crate::prompt_window::PromptState;
use crate::recorder::RecordingSession;
use crate::settings::Settings;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Global application state managed by Tauri.
pub struct AppState {
    /// SQLite connection. `Connection` is `Send` but `!Sync`, so we guard it.
    pub db: Arc<Mutex<Connection>>,
    /// Shared HTTP client for Minutes server calls.
    pub http: reqwest::Client,
    /// Directory where `settings.json` lives.
    pub config_dir: PathBuf,
    pub settings: Mutex<Settings>,
    /// The in-flight recording, if any.
    pub session: Mutex<Option<RecordingSession>>,
    /// Set while a recording start is in flight but `session` is not yet
    /// populated. `recorder::start` does real work (opens the audio device,
    /// inserts the meeting row) between its "already recording?" check and the
    /// moment it stores the session, so a plain check on `session` lets two
    /// near-simultaneous starts both pass the guard and record on top of each
    /// other. This flag closes that window: it is claimed atomically via
    /// [`AppState::try_claim_recording_slot`] before any work happens, and the
    /// RAII [`RecordingClaim`] guarantees it is released on every error path.
    pub starting: AtomicBool,
    /// Floating meeting-prompt staging + call-detect dismiss cooldowns.
    pub prompt: PromptState,
}

impl AppState {
    /// True while a recording is active *or* one is currently starting.
    pub fn is_recording(&self) -> bool {
        self.starting.load(Ordering::Acquire) || self.session.lock_safe().is_some()
    }

    pub fn recording_meeting_id(&self) -> Option<String> {
        self.session
            .lock_safe()
            .as_ref()
            .map(|s| s.meeting_id.clone())
    }

    /// Atomically claim the exclusive right to start a recording.
    ///
    /// Exactly one caller can hold the claim at a time; concurrent callers get
    /// an error immediately. The claim is released automatically when the
    /// returned guard is dropped (e.g. an early `?` return in
    /// `recorder::start`), so a failed start can never leave the app stuck in a
    /// permanent "starting" state. Call [`RecordingClaim::commit`] with the
    /// live session once the start has fully succeeded.
    pub fn try_claim_recording_slot(&self) -> Result<RecordingClaim<'_>, String> {
        // Claim the flag first; only one thread can flip false -> true.
        if self
            .starting
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err("a recording is already in progress".into());
        }
        // Holding the flag, check for an already-running session. Only the
        // flag holder ever sets `session`, so this check cannot race another
        // start.
        if self.session.lock_safe().is_some() {
            self.starting.store(false, Ordering::Release);
            return Err("a recording is already in progress".into());
        }
        Ok(RecordingClaim { state: self })
    }
}

/// RAII guard for an in-flight recording start (see
/// [`AppState::try_claim_recording_slot`]).
///
/// Dropping the guard without committing releases the "starting" flag, so
/// every error path in the start flow — including `?` returns — automatically
/// frees the slot for the next attempt.
pub struct RecordingClaim<'a> {
    state: &'a AppState,
}

impl RecordingClaim<'_> {
    /// Publish the successfully started recording session.
    ///
    /// The session is stored *before* the guard drops and clears the
    /// "starting" flag, so `is_recording()` never observes a false gap between
    /// "starting" and "recording".
    pub fn commit(self, session: RecordingSession) {
        *self.state.session.lock_safe() = Some(session);
        // `self` drops here, clearing `starting`; `session` is already set.
    }
}

impl Drop for RecordingClaim<'_> {
    fn drop(&mut self) {
        self.state.starting.store(false, Ordering::Release);
    }
}
