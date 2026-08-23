//! Un-favorite in place: fade (set `removing`) then remove.

use slint::{ComponentHandle, Model, VecModel};

use crate::favorites::derive::derive_playlists;
use crate::{AlbumCardItem, AppWindow, FavoritesState, SearchPlaylistItem, TrackItem};

/// Flag the matching track row(s) as removing so they fade out.
pub fn mark_track_removing(window: &AppWindow, id: &str) {
    let state = window.global::<FavoritesState>();
    for model in [state.get_tracks_visible(), state.get_tracks()] {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == id && !item.removing {
                    item.removing = true;
                    model.set_row_data(i, item);
                }
            }
        }
    }
}

/// Remove the track row from both the rendered + full models (after fade).
pub fn remove_track_row(window: &AppWindow, id: &str) {
    let state = window.global::<FavoritesState>();
    for model in [state.get_tracks_visible(), state.get_tracks()] {
        if let Some(vm) = model.as_any().downcast_ref::<VecModel<TrackItem>>() {
            for i in 0..vm.row_count() {
                if vm.row_data(i).map(|t| t.id == id).unwrap_or(false) {
                    vm.remove(i);
                    break;
                }
            }
        }
    }
}

/// Flag the matching album card(s) as removing so they fade out.
pub fn mark_album_removing(window: &AppWindow, id: &str) {
    let state = window.global::<FavoritesState>();
    for model in [state.get_albums_visible(), state.get_albums()] {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == id && !item.removing {
                    item.removing = true;
                    model.set_row_data(i, item);
                }
            }
        }
    }
}

/// Remove the album card from both the rendered + full models (after fade).
pub fn remove_album_row(window: &AppWindow, id: &str) {
    let state = window.global::<FavoritesState>();
    for model in [state.get_albums_visible(), state.get_albums()] {
        if let Some(vm) = model.as_any().downcast_ref::<VecModel<AlbumCardItem>>() {
            for i in 0..vm.row_count() {
                if vm.row_data(i).map(|a| a.id == id).unwrap_or(false) {
                    vm.remove(i);
                    break;
                }
            }
        }
    }
}

/// Remove a playlist from the Library (favorites) source after a local
/// un-favorite, then re-derive the rendered model + update the tab badge.
/// Following is untouched (a followed playlist stays followed).
pub fn remove_playlist_row(window: &AppWindow, id: &str) {
    let state = window.global::<FavoritesState>();
    let model = state.get_playlists_favorites();
    if let Some(vm) = model.as_any().downcast_ref::<VecModel<SearchPlaylistItem>>() {
        for i in 0..vm.row_count() {
            if vm.row_data(i).map(|p| p.id == id).unwrap_or(false) {
                vm.remove(i);
                break;
            }
        }
    }
    state.set_playlists_total(model.row_count() as i32);
    derive_playlists(window);
}
