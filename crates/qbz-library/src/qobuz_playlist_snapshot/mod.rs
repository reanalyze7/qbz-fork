//! Local snapshot of the user's QOBUZ playlists (offline-mode port, B7/B8).
//!
//! Spec D11 left an HONEST LIMIT: playlist names and membership live only in
//! the Qobuz API, so offline a mixed playlist falls back to a synthesized
//! "Playlist (N local)" name and shows zero Qobuz rows. This module stores a
//! point-in-time snapshot captured opportunistically from data the app
//! ALREADY fetches while online (no new API traffic):
//!
//! - NAMES: every user-playlist list load (sidebar / playlist manager)
//!   upserts id + name (+ owner, track_count) for ALL listed playlists —
//!   cheap names-only rows, no track membership.
//! - MEMBERSHIP: opening a playlist DETAIL online full-replaces its snapshot
//!   track ids (the detail fetch already returns the full track list).
//!   Membership is recorded ONLY for playlists already captured by the
//!   names producer (the user's own list) — a merely-viewed public playlist
//!   never lands in the snapshot, so the offline surfaces stay the user's.
//!
//! Rows are point-in-time: offline consumers show them as-is (no staleness
//! UI in v1); `snapped_at` is stamped for the future.
//!
//! All functions take `&Connection` (the local_playlists idiom): no Tauri
//! state, no async runtime — testable with in-memory SQLite.

mod read;
mod schema;
mod types;
mod write;

#[cfg(test)]
mod tests;

pub use read::{all_headers, all_track_ids, get_header, track_ids};
pub use schema::init_schema;
pub use types::{SnapshotHeader, SnapshotNameEntry};
pub use write::{replace_tracks, upsert_names};
