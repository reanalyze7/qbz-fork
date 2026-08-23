use slint::{ComponentHandle, ModelRc, VecModel};

use crate::favorites::derive::derive_playlists;
use crate::search::{self, PlaylistRow};
use crate::{AppWindow, FavoritesState, SearchPlaylistItem};

pub(crate) fn apply_playlists(window: &AppWindow, favorites: Vec<PlaylistRow>, following: Vec<PlaylistRow>) {
    let state = window.global::<FavoritesState>();
    let fav_items: Vec<SearchPlaylistItem> =
        favorites.into_iter().map(search::playlist_item).collect();
    let following_items: Vec<SearchPlaylistItem> =
        following.into_iter().map(search::playlist_item).collect();
    // Tab badge = Library (favorited) count; Following badge separate.
    state.set_playlists_total(fav_items.len() as i32);
    state.set_playlists_following_count(following_items.len() as i32);
    state.set_playlists_favorites(ModelRc::new(VecModel::from(fav_items)));
    state.set_playlists_following(ModelRc::new(VecModel::from(following_items)));
    state.set_playlists_search("".into());
    // Seed `playlists-visible` for the current sub-tab (shares the
    // source model until a search forks it, so collage artwork stays live).
    derive_playlists(window);
}
