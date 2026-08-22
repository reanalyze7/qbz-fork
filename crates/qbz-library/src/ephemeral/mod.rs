//! In-memory ephemeral library for ad-hoc folder playback.
//!
//! The user can point QBZ at a folder that lives outside their library
//! (a downloaded album they haven't decided to keep, an external drive,
//! etc.), browse it, and play tracks from it without anything landing
//! in `local_tracks`. The ephemeral session lives only in memory: a
//! `HashMap<i64, LocalTrack>` keyed by *synthetic ids in the high
//! range* (>= `EPHEMERAL_ID_FLOOR` = 2^48). Synthetic ids in this range
//! are how the rest of the playback pipeline distinguishes ephemeral
//! tracks from DB-resolvable ones — local_tracks autoincrement IDs are
//! orders of magnitude smaller, so any track_id arriving at
//! `v2_library_play_track` at or above the floor is unambiguously
//! ephemeral and gets routed here instead of the DB.
//!
//! The high-positive design (instead of the obvious "use negatives")
//! exists because the queue/playback-context commands serialize ids as
//! `u64` end-to-end (V2QueueTrack, v2_set_playback_context) and reject
//! negative numbers at the serde boundary. Positive ids above the DB
//! range and below 2^53 (JS Number safe limit) are valid u64 *and*
//! survive the JSON round-trip without precision loss.
//!
//! Only one folder is held at a time; opening a new folder replaces the
//! previous session. The state vanishes on app exit by virtue of being
//! in-memory — nothing persists, no migration, no cleanup logic needed.
//!
//! This module is frontend-agnostic (ADR-006): it has zero Tauri/Slint
//! dependency and is consumed by both frontends. The Tauri build re-exports
//! it via `src-tauri/src/ephemeral_library/mod.rs`; the Slint build wraps it
//! in a process-global singleton in `crates/qbz-slint/src/ephemeral.rs`.
//!
//! Split across this directory by responsibility: this file holds the
//! public types and shared state shell; `scan_cue` and `scan_audio` are
//! the two scan sub-passes run by `open_folder` (in `open_folder.rs`);
//! `query` holds the small read/clear accessors.

mod open_folder;
mod query;
mod scan_audio;
mod scan_cue;

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{LibraryError, LocalTrack};
use serde::Serialize;

/// Floor for synthetic ephemeral track ids. Any id at or above this
/// value is an ephemeral track; below it is a DB row id. Set high
/// enough to be impossible to collide with autoincrement DB ids in any
/// realistic library size, low enough to fit in JS Number's safe
/// integer range (2^53 - 1) so the JSON round-trip stays lossless.
pub const EPHEMERAL_ID_FLOOR: i64 = 1 << 48;

#[derive(Debug, Serialize, Clone)]
pub struct EphemeralFolderResult {
    pub folder_path: String,
    pub tracks: Vec<LocalTrack>,
    pub skipped_files: usize,
}

#[derive(Debug)]
pub enum EphemeralError {
    Lock,
    Library(String),
    Io(String),
}

impl std::fmt::Display for EphemeralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lock => write!(f, "ephemeral library state lock poisoned"),
            Self::Library(msg) => write!(f, "{}", msg),
            Self::Io(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<LibraryError> for EphemeralError {
    fn from(e: LibraryError) -> Self {
        EphemeralError::Library(e.to_string())
    }
}

struct EphemeralLibraryInner {
    tracks: HashMap<i64, LocalTrack>,
    next_id: i64,
    current_folder_path: Option<String>,
}

impl EphemeralLibraryInner {
    fn new() -> Self {
        Self {
            tracks: HashMap::new(),
            next_id: EPHEMERAL_ID_FLOOR,
            current_folder_path: None,
        }
    }

    fn reset(&mut self) {
        self.tracks.clear();
        self.next_id = EPHEMERAL_ID_FLOOR;
        self.current_folder_path = None;
    }
}

pub struct EphemeralLibraryState {
    inner: Mutex<EphemeralLibraryInner>,
}

impl EphemeralLibraryState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(EphemeralLibraryInner::new()),
        }
    }
}

impl Default for EphemeralLibraryState {
    fn default() -> Self {
        Self::new()
    }
}
