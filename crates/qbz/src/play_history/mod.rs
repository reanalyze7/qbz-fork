//! Local play history — the source of truth for the discovery
//! filter "skip artists I already know".
//!
//! Mirrors the minimum-needed shape of `src-tauri/src/reco_store` for
//! the artist-network sidebar: a `play_events` table that grows one
//! row per track-start, and a side `artist_names` table that maps
//! id -> name (updated on each play). The discovery pipeline reads
//! both at once and turns them into the (qobuz_ids, normalized_names)
//! pair that filters MB candidates and validated Qobuz matches.
//!
//! SQLite is opened lazily once, and every read/write swallows errors
//! into a `log::warn!`. A fresh user (no DB yet) yields empty sets,
//! which simply means no exclusion is applied — same default Tauri
//! lands on a first-run profile.

mod db;
mod query;
mod record;

pub use query::known_artists;
pub use record::record_play;
