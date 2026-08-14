//! Small concurrency helpers.

use std::sync::{Mutex, MutexGuard};

/// Locking that survives poisoning.
///
/// A poisoned `Mutex` means another thread panicked while holding the lock.
/// With plain `.lock().unwrap()` that panic then cascades: every subsequent
/// lock attempt panics too, so a single failure anywhere permanently bricks the
/// app until it is restarted (an availability / DoS hazard). For this app's
/// guarded state — a SQLite handle, the settings struct, the recording session,
/// an integrity cache — the data behind the lock is still structurally valid, so
/// we recover the guard via [`std::sync::PoisonError::into_inner`] instead of
/// propagating the panic.
pub trait MutexExt<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}
