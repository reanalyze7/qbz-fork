//! Library > Favorites controller — fetches the user's saved
//! tracks / albums / artists via `QbzCore::get_favorites` and pushes
//! them into `FavoritesState`. Mirrors Tauri's FavoritesView.svelte
//! data flow: each tab is fetched lazily the first time it is opened.
//!
//! `get_favorites` returns a raw JSON value shaped
//! `{ <type>: { items: [...], total: N } }`; this module parses the
//! relevant branch into typed qbz-models items and maps them to the
//! Slint row/card structs.

mod albums_artwork;
mod apply;
mod artwork_apply;
mod artwork_jobs;
mod counts;
mod derive;
mod fetch;
mod mapping;
mod mutate_rows;
mod random;
mod selected_artist;
mod selection;

use std::cell::RefCell;
use std::collections::HashMap;

use slint::ComponentHandle;

use crate::AppWindow;

pub use albums_artwork::{
    album_artwork_job_done, albums_gen_current, albums_view_mode_changed, albums_window_changed,
    begin_albums_artwork,
};
pub use apply::apply_favorites;
pub use artwork_apply::{set_album_artwork, set_artist_image, set_playlist_cover, set_track_artwork};
pub use artwork_jobs::artwork_jobs;
pub use counts::{apply_counts, load_counts, FavCounts};
pub use derive::{derive_albums, derive_artists, derive_labels, derive_playlists, derive_tracks};
pub use fetch::{favorite_album_ids, load_favorites, FavData};
pub use mapping::TrackCard;
pub use mutate_rows::{mark_album_removing, mark_track_removing, remove_album_row, remove_playlist_row, remove_track_row};
pub use random::{
    play_tracks, random_visible_album, random_visible_artist, random_visible_label,
    random_visible_playlist, shuffled_tracks,
};
pub use selected_artist::{apply_selected_artist, selected_artist_artwork_jobs};
pub use selection::{clear_selection, recount_selected, select_all, selected_ids, selected_tracks, set_multi_select};

/// Page size — matches Tauri's FAVORITES_PAGE_SIZE. We fetch one
/// page on tab open (favorites lists are typically small; full
/// pagination can come later).
pub const PAGE_SIZE: u32 = 500;

/// Hard ceiling on favorites pulled across all pages (mirrors Tauri's
/// FAVORITES_PAGE_SIZE * FAVORITES_MAX_PAGES ceiling).
pub(crate) const MAX_ITEMS: usize = 10_000;

thread_local! {
    /// Track id -> genre name, for the favorites Tracks genre filter
    /// (TrackItem carries no genre). Set on the UI thread by apply.
    pub(crate) static FAV_TRACK_GENRE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Which favorites tab to load.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FavTab {
    Tracks,
    Albums,
    Artists,
    Playlists,
    Labels,
}

impl FavTab {
    pub fn from_route(route: &str) -> Option<Self> {
        Self::from_tab_id(route.strip_prefix("favorites-")?)
    }

    pub fn from_tab_id(id: &str) -> Option<Self> {
        match id {
            "tracks" => Some(Self::Tracks),
            "albums" => Some(Self::Albums),
            "artists" => Some(Self::Artists),
            "playlists" => Some(Self::Playlists),
            "labels" => Some(Self::Labels),
            _ => None,
        }
    }

    /// The Qobuz favType string + the JSON branch key (for the
    /// get_favorites-backed tabs).
    fn key(self) -> &'static str {
        match self {
            Self::Tracks => "tracks",
            Self::Albums => "albums",
            Self::Artists => "artists",
            Self::Playlists => "playlists",
            Self::Labels => "labels",
        }
    }
}

pub fn reset_loading(window: &AppWindow) {
    let state = window.global::<crate::FavoritesState>();
    state.set_loading(true);
    state.set_load_error("".into());
}
