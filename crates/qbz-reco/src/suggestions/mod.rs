//! Playlist suggestions engine
//!
//! Uses artist vectors to suggest new tracks for a playlist.
//! Algorithm:
//! 1. Extract unique artists from playlist tracks
//! 2. Compute combined playlist vector (sum + normalize)
//! 3. Find nearest artists not already in playlist
//! 4. Search Qobuz for top tracks by those artists
//! 5. Return suggested tracks with optional reasons
//!
//! Ported 1:1 from the Tauri `artist_vectors::suggestions`. Only the API client
//! types are swapped (`crate::api::{QobuzClient, Track}` →
//! `qbz_qobuz::QobuzClient` / `qbz_models::Track`); the `Arc<tokio::Mutex/RwLock>`
//! ownership is kept (the store/cache hold `!Sync` rusqlite connections, so each
//! guard is dropped before every `.await`, exactly like `builder.rs`). Step 3
//! ranks candidates by summed relationship weight via
//! `store.get_all_related_artists`, NOT cosine similarity (epic decision D3); the
//! Step-2 `compute_playlist_vector` is kept because production still uses it as
//! the empty-vector gate (it only sums + normalizes — it never `find_nearest`s).
//!
//! **Locking discipline (critical, preserve across all submodules)**: every
//! guard (`store.lock().await`, `qobuz_client.read().await`) is scoped in a
//! block and dropped before crossing an `.await` — same discipline as
//! `builder.rs`, required for the spawned suggestions future to remain `Send`.

mod engine;
mod generate;
mod genre_filter;
mod mbids;
mod name_match;
mod playlist_vector;
mod reason;
#[cfg(test)]
mod tests;
mod track_convert;
mod track_search;
mod types;
mod validate_artist;

pub use engine::SuggestionsEngine;
pub use mbids::extract_artist_mbids;
pub use types::{SuggestedTrack, SuggestionConfig, SuggestionResult};
