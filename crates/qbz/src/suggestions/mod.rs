//! Immersive Suggestions panel controller (split-only, split-panel == 2).
//!
//! Ports Tauri's `SuggestionsPanel.svelte` 1:1, with all assembly logic in
//! Rust (ADR-006): live artist queries only (NEVER `reco_store` — that powers
//! the home page; see data-panels.md §6). Two data products:
//!
//!   * RECOMMENDED TRACKS — `artist.tracks_appears_on`, falling back to
//!     `get_artist_tracks(limit 30)` when sparse (<5), deduped by exact
//!     lowercase title, the current track filtered out, shuffled, take 10.
//!   * CARDS — the first 2 curated `artist.playlists` (each a book-collage of
//!     up to 3 distinct album covers, fetched via `get_playlist`) + ONE seed
//!     "Song Radio" card (diamond-collage of up to 4 rec-track covers).
//!
//! The shuffle is a deterministic splitmix64 seeded off the artist+track ids
//! (matches qbz-radio's RNG family; avoids pulling `rand` just for this).

mod apply;
mod artwork;
mod covers;
mod load;
mod shuffle;
mod types;

pub use apply::{apply_suggestions, reset_suggestions};
pub use artwork::suggestions_artwork_jobs;
pub use load::load_suggestions;
pub use types::empty_payload;

/// Recommended-track target count (Tauri `slice(0, 10)`).
const REC_LIMIT: usize = 10;
/// Sparse threshold below which the artist-tracks fallback runs (Tauri `< 5`).
const SPARSE_THRESHOLD: usize = 5;
/// Artist-tracks fallback page size (Tauri `limit: 30`).
const FALLBACK_LIMIT: u32 = 30;
/// Max curated playlist cards (Tauri `slice(0, 2)`).
const MAX_PLAYLIST_CARDS: usize = 2;
/// Book-collage cover count per playlist card (Tauri 3).
const BOOK_COVERS: usize = 3;
/// Diamond-collage cover count for the radio card (Tauri max 4).
const RADIO_COVERS: usize = 4;
