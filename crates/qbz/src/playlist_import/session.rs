//! Rust-side session mirror + import generation counter.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use qbz_playlist_import::{ImportPlaylist, ProviderKey};

/// Rust-side mirror of the Svelte component state that never reaches the
/// UI (sidebar.rs module-state pattern). Reset wholesale on every open.
#[derive(Default)]
pub(super) struct Session {
    pub preview: Option<ImportPlaylist>,
    /// Trimmed URL the preview was fetched for (Svelte `previewUrl`).
    pub preview_url: String,
    /// Provider locked at fetch time; survives URL edits until the reset
    /// paths clear it (Svelte `lockedProvider`).
    pub locked_provider: Option<ProviderKey>,
    /// Trimmed URL of the last completed import (Svelte `lastImportedUrl`).
    pub last_imported_url: String,
    /// 5%-milestone tracker for the matching log lines (-1 = none yet).
    pub last_logged_percent: i32,
    /// Mirror of the modal's rename field, kept fresh by `name-edited`
    /// and read at execute time (Svelte `customName`).
    pub custom_name: String,
}

pub(super) static SESSION: LazyLock<Mutex<Session>> = LazyLock::new(|| Mutex::new(Session::default()));

/// Import generation (§1.8): bumped on every open() and execute(). Sink
/// events and task completions carry the generation they were spawned
/// with; a mismatch means the modal was reset for a fresh run, so the
/// stale run may only fire toast + sidebar refresh, never modal writes.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn current_generation() -> u64 {
    GENERATION.load(Ordering::SeqCst)
}

pub(super) fn bump_generation() -> u64 {
    GENERATION.fetch_add(1, Ordering::SeqCst) + 1
}
