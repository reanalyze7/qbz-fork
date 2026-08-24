//! "Add to Mixtape/Collection" controller — the Rust side of the global
//! `AddToMixtapeModal` (spec 21/50). Every app-wide "Add to Mixtape/Collection"
//! trigger builds an [`AddItem`] payload (or a batch) and calls [`open`]; this
//! module holds the pending items in a process-global, loads the picker list
//! (kind-restricted + recency-sorted + `item_exists`-resolved), and on pick /
//! create-and-add writes the items into the chosen collection via the shared
//! `qbz_mixtape::repo` (reached through `crate::library_db::with_db` +
//! `with_connection` — no Tauri command wrappers, ADR-005/006).
//!
//! Dedup is the backend's job: `add_item_with(allow_duplicate=false)` returns
//! `false` for an exact `(collection_id, source, source_item_id)` duplicate
//! (not an error). We count `added` vs `skipped` and surface the outcome via a
//! toast ("Added N to {name}" / "Already in {name}"), mirroring Tauri's
//! `toastBatchResult` + the dup flow's net result.

mod mutate;
mod open_close;
mod render;
mod rows;

pub use mutate::{add_items, create_collection, take_pending, toast_outcome, track_items_from_local, AddOutcome};
pub use open_close::{close, open};
pub use render::{apply_rows, rebuild};
pub use rows::load_rows;

use std::sync::{LazyLock, Mutex};

use qbz_models::mixtape::{AlbumSource, ItemType};

/// One pending item to add. Built by each callsite from its row/album/playlist
/// data (spec 50 §0.2). `source_item_id` is ALWAYS a string (numeric track ids
/// are stringified by the caller).
#[derive(Clone)]
pub struct AddItem {
    /// "album" | "track" | "playlist".
    pub item_type: String,
    /// "qobuz" | "local".
    pub source: String,
    pub source_item_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub artwork_url: Option<String>,
    pub year: Option<i32>,
    pub track_count: Option<i32>,
}

/// Pending items for the currently-open picker. Set by [`open`], read by the
/// add/create handlers. Cleared on close.
pub(super) static PENDING: LazyLock<Mutex<Vec<AddItem>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub(super) fn pending_snapshot() -> Vec<AddItem> {
    PENDING.lock().map(|p| p.clone()).unwrap_or_default()
}

pub(super) fn item_type_from_str(s: &str) -> ItemType {
    match s {
        "track" => ItemType::Track,
        "playlist" => ItemType::Playlist,
        _ => ItemType::Album,
    }
}

pub(super) fn source_from_str(s: &str) -> AlbumSource {
    match s {
        "local" => AlbumSource::Local,
        _ => AlbumSource::Qobuz,
    }
}
