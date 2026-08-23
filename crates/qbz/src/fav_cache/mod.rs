//! Shared favorite-track cache.
//!
//! A single process-wide set of the user's favorite track IDs, so every
//! track-list surface (album, artist, search, playlist, mix, favorites,
//! queue) can stamp `is-favorite` on each row without re-fetching, and the
//! row heart can toggle optimistically.
//!
//! Disk-first seeding: [`init_for_user`] binds the per-user persistent
//! store (`favorites_cache.db`, same file + schema as Tauri) on session
//! activation and loads the IDs from disk — so hearts are correct offline.
//! The online shell entry then refreshes the set from the network and
//! writes it back via [`set_all`]. Toggles keep memory and disk in sync
//! through [`set`].

mod albums;
mod artists;
mod lifecycle;
mod tracks;

pub use albums::{is_album_favorite, set_album, set_all_albums};
pub use artists::{all_artists, is_artist_favorite, set_all_artists, set_artist};
pub use lifecycle::{init_for_user, teardown};
pub use tracks::{all, contains, is_favorite, set, set_all};

use std::collections::HashSet;
use std::sync::{LazyLock, Mutex, RwLock};

use qbz_app::settings::favorites_cache::FavoritesCacheStore;

pub(super) static FAVORITES: LazyLock<RwLock<HashSet<u64>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

/// Process-wide set of the user's favorite ALBUM ids (string catalog ids).
/// Same disk-first + network-refresh lifecycle as [`FAVORITES`], so the
/// album header heart renders the right state from first paint and stays
/// live across toggles. Mirrors Tauri's `albumFavoritesStore`.
pub(super) static FAV_ALBUMS: LazyLock<RwLock<HashSet<String>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

/// Process-wide set of the user's followed ARTIST ids (u64). Same disk-first
/// + network-refresh lifecycle as the album set (seeded from the per-user
/// store in [`init_for_user`], refreshed by the shell-entry warm and the
/// search/artist-page loaders) — the Pinned carousel's artist follow chip
/// reads it at build time, so it must be correct from first paint and offline.
pub(super) static FAV_ARTISTS: LazyLock<RwLock<HashSet<u64>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

/// Per-user persistent ID store. `None` until a session (online or offline)
/// is activated; pure in-memory behavior in that window.
pub(super) static STORE: Mutex<Option<FavoritesCacheStore>> = Mutex::new(None);
