//! Apply a freshly fetched favorites tab into `FavoritesState`.

mod albums;
mod artists;
mod labels;
mod playlists;
mod tracks;

use slint::ComponentHandle;

use crate::favorites::fetch::FavData;
use crate::{AppWindow, FavoritesState};

/// Apply one loaded favorites tab. Runs on the Slint event loop. For the
/// Albums arm, the caller must have called `begin_albums_artwork` first (see
/// that fn's doc comment) so the windowed-artwork generation guard is
/// current before `derive_albums` dispatches covers.
pub fn apply_favorites(window: &AppWindow, data: FavData) {
    let state = window.global::<FavoritesState>();
    match data {
        FavData::Tracks { items, play, total } => tracks::apply_tracks(window, items, play, total),
        FavData::Albums { items, total } => albums::apply_albums(window, items, total),
        FavData::Artists { items, total } => artists::apply_artists(window, items, total),
        FavData::Playlists { favorites, following } => {
            playlists::apply_playlists(window, favorites, following)
        }
        FavData::Labels { items, total } => labels::apply_labels(window, items, total),
    }
    state.set_loading(false);
}
