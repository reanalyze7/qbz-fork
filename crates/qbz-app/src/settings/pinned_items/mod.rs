//! Headless pinned-items service.
//!
//! Frontend-agnostic store for the Home "Pinned" section: albums, artists and
//! playlists the user pins from card glyphs. No UI knowledge, per ADR-006 —
//! this mirrors `artist_blacklist.rs` (same pragmas, error style, in-memory
//! set seeding); the per-user lifecycle lives in the `qbz` crate wrapper.
//!
//! Provides O(1) pinned checks via an in-memory `HashSet` of `(kind, id)`
//! keys backed by SQLite persistence. Rows carry a display snapshot
//! (title/subtitle/artwork) taken at pin time so the section renders without
//! re-fetching.

mod ops;
mod service;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use service::PinnedItemsService;

/// Database file name for the pinned-items store, joined onto the per-user
/// data dir by the lifecycle layer.
pub const DB_FILE_NAME: &str = "pinned_items.db";

/// A pinned entry with its display snapshot.
///
/// Ids are Strings on purpose: Qobuz album ids are alphanumeric, and artist /
/// playlist ids (numeric upstream) arrive as strings in card rows — the
/// `(kind, id)` composite TEXT key covers all three without an INTEGER axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedItem {
    /// "album" | "artist" | "playlist".
    pub kind: String,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artwork_url: String,
    /// Unix seconds; the ordering key (newest first).
    pub pinned_at: i64,
}
