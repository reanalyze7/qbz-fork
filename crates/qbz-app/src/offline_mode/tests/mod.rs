mod banner;
mod basic;
mod persistence;

use super::*;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

pub(super) fn unique_test_dir(name: &str) -> std::path::PathBuf {
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let nonce = NONCE.fetch_add(1, AtomicOrdering::Relaxed);
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

/// The engine flips the PROCESS-GLOBAL `qbz_qobuz::offline_gate`; these
/// tests must not run concurrently or the gate assertions race.
static GATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(super) fn serialize() -> std::sync::MutexGuard<'static, ()> {
    GATE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn up() -> ConnectivitySnapshot {
    ConnectivitySnapshot {
        state: Connectivity::Up,
        captive_portal: false,
    }
}

pub(super) fn down() -> ConnectivitySnapshot {
    ConnectivitySnapshot {
        state: Connectivity::Down,
        captive_portal: false,
    }
}
