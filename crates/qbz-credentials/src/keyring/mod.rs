//! Keyring session state and the broken-state latch.
//!
//! Linux Secret Service (the backing store for the `keyring` crate on Linux)
//! is allowed to prompt the user for a passphrase when its collection is
//! locked. That prompt blocks the calling thread indefinitely. A user whose
//! GNOME Keyring got out of sync with their login password sees the dialog
//! appear, dismisses it (because they can't remember the old password), and
//! we then re-try another keyring operation which produces the dialog again
//! — and again, and again, for every keyring touch in the login flow.
//!
//! The defense here has two parts:
//!
//! 1. Each keyring call is executed on a worker thread with a hard wall-clock
//!    timeout (`ops::run_with_keyring_timeout`). If the call hasn't returned
//!    within the timeout, we assume the user is staring at a dialog they
//!    can't satisfy and we give up — the worker thread stays blocked in the
//!    background, but our control flow moves on. (The worker eventually
//!    resolves when the user dismisses the dialog; it's a small thread leak
//!    once per broken-keyring session.)
//!
//! 2. The first failure or timeout latches a process-wide flag
//!    (`KEYRING_STATE`) that short-circuits every subsequent keyring touch.
//!    One prompt max per session; everything after that goes straight to the
//!    encrypted file. A restart is required to retry the keyring, which is
//!    also the point where the user's keyring might have been repaired.
//!
//! Errors that mean "the entry doesn't exist" (`keyring::Error::NoEntry`)
//! don't count as a failure — they're just data the caller has to handle.

mod ops;

pub(crate) use ops::{keyring_delete, keyring_get, keyring_set};

use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) const SERVICE_NAME: &str = "qbz";

const KEYRING_UNTESTED: u8 = 0;
const KEYRING_WORKING: u8 = 1;
const KEYRING_BROKEN: u8 = 2;

static KEYRING_STATE: AtomicU8 = AtomicU8::new(KEYRING_UNTESTED);

pub(super) fn keyring_is_broken() -> bool {
    KEYRING_STATE.load(Ordering::Relaxed) == KEYRING_BROKEN
}

pub(super) fn mark_keyring_broken(reason: &str) {
    let previous = KEYRING_STATE.swap(KEYRING_BROKEN, Ordering::Relaxed);
    if previous != KEYRING_BROKEN {
        log::warn!(
            "[Credentials] Disabling system keyring for the rest of this session \
             (falling back to encrypted file only): {}",
            reason
        );
    }
}

pub(super) fn mark_keyring_working() {
    // Only promote from UNTESTED to WORKING. Never climb back out of BROKEN —
    // once we've given up on the keyring for this session, stay given up.
    let _ = KEYRING_STATE.compare_exchange(
        KEYRING_UNTESTED,
        KEYRING_WORKING,
        Ordering::Relaxed,
        Ordering::Relaxed,
    );
}
