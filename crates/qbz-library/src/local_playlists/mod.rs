//! First-class LOCAL playlists (offline-mode port, decisions D7/D8).
//!
//! Unlike the `playlist_*` sidecar tables (which enhance a *Qobuz* playlist
//! keyed by its server id), these are standalone entities living entirely in
//! the per-user `library.db`. Ids are TEXT `local:<uuid>` (Mixtape precedent)
//! so they are unrepresentable in any Qobuz-bound call that takes a `u64`
//! playlist id — the type guard demanded by D7.
//!
//! All functions take `&Connection` (the qbz-mixtape repo idiom): no Tauri
//! state, no async runtime — testable with in-memory SQLite. The Slint
//! command layer reaches them through `LibraryDatabase::with_connection`.
//!
//! Split into: `model` (types), `schema` (DDL/migrations), `playlist_ops` +
//! `playlist_query` (playlist-header CRUD), `track_ops` + `track_reorder`
//! (membership CRUD). This file re-exports every public item so
//! `qbz_library::local_playlists::X` paths are unchanged.

mod model;
mod playlist_ops;
mod playlist_query;
mod schema;
mod track_ops;
mod track_reorder;

#[cfg(test)]
mod tests;

pub use model::{
    is_local_playlist_id, LocalPlaylist, LocalPlaylistTrack, LocalPlaylistTrackInput,
    LocalPlaylistTrackSource, LOCAL_PLAYLIST_PREFIX,
};
pub use playlist_ops::{
    clear_folder, create, delete, move_to_folder, rename, set_custom_artwork, set_description,
    set_favorite, set_hidden, set_offline_only,
};
pub use playlist_query::{get, list};
pub use schema::init_schema;
pub use track_ops::{add_tracks, get_tracks};
pub use track_reorder::{remove_track, reorder};
