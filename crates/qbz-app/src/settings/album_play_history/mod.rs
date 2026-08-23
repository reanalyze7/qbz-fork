//! Local album play history — source of truth for the "Most Played Albums"
//! rail + its View-all page.
//!
//! Mirrors [`crate::play_history`] (the per-artist store): an
//! `album_play_events` table grows one row per track-start whose album is
//! known, plus a side `album_meta` table (id -> title/artist/artwork/quality)
//! refreshed on each play. "Most played" = `COUNT(*) GROUP BY album_id
//! ORDER BY plays DESC`. Counting is PER TRACK-START, like play_history, so an
//! album listened all the way through adds one per track.
//!
//! The Qobuz API exposes no most-played endpoint (verified against the
//! inferred OpenAPI), so the ranking is derived locally from our own plays.
//!
//! SQLite is opened lazily; every read/write swallows errors into a
//! `log::warn!`. A fresh user (no DB yet) yields an empty list, so the rail
//! self-hides — same default the other #566 rails land on.

mod api;
mod db;
mod model;
mod queries;
#[cfg(test)]
mod tests;

pub use api::{all_albums, record_album_play, top_albums};
pub use model::{AlbumPlayMeta, AlbumPlayRow};
