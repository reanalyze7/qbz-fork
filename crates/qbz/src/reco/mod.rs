//! Per-user `RecoStore` runtime wrapper (mirrors `fav_cache.rs`).
//!
//! Holds the ported, headless `qbz_app` [`RecoStore`] behind a process-global
//! `Mutex<Option<…>>`; all access goes through typed helpers so the source
//! gating and the `spawn_blocking` discipline live in ONE place. Every helper
//! degrades to a no-op when no session is active (the store is `None`), so
//! callers never branch on "is reco enabled" — reco simply contributes
//! nothing until a session opens it.
//!
//! Lifecycle mirrors `fav_cache`: [`init_for_user`] on every session
//! activation (login, restore, offline entry), [`teardown`] on logout. The DB
//! file is shared with Tauri (`<base_dir>/reco/events.db`), so a user's
//! existing recommendation history carries across frontends.

mod favorite;
mod lifecycle;
mod play;
mod playlist;
mod surfaces;
#[cfg(test)]
mod tests;
mod train;

pub use favorite::{log_favorite_album, log_favorite_artist, log_favorite_track};
pub use lifecycle::{init_for_user, teardown};
pub use play::{is_qobuz_source, log_play_gated};
pub use playlist::log_playlist_add;
pub use surfaces::{
    backfill_album_genres, forgotten_favorite_album_ids, home_seeds, known_artist_ids,
    recent_track_ids, scored_favorite_album_ids,
};
pub use train::train_async;

use std::sync::Mutex;

use qbz_app::settings::reco_store::RecoStore;

/// Per-user reco event store. `None` until a session (online or offline) is
/// activated; every helper is a no-op in that window.
static RECO: Mutex<Option<RecoStore>> = Mutex::new(None);
