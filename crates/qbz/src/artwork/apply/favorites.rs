//! Favorites-tab arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::FavoriteTrack { index } => {
            let model = window.global::<crate::FavoritesState>().get_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                let id = item.id.to_string();
                model.set_row_data(index, item);
                // Also reach the rendered (possibly sorted/grouped) model.
                crate::favorites::set_track_artwork(window, &id, image.clone());
            }
        }
        ArtworkTarget::FavoriteAlbumById { id, gen } => {
            // The job is done either way — free its in-flight slot so the
            // window dispatcher can re-request it after an eviction.
            crate::favorites::album_artwork_job_done(&id);
            // Drop the cover if a reload superseded the set it belongs to.
            if !crate::favorites::albums_gen_current(gen) {
                return true;
            }
            // Set by id onto the full set + visible + grouped sections.
            crate::favorites::set_album_artwork(window, &id, image.clone());
        }
        ArtworkTarget::FavoriteArtist { index } => {
            let model = window.global::<crate::FavoritesState>().get_artists();
            if let Some(mut item) = model.row_data(index) {
                let id = item.id.to_string();
                item.image = image.clone();
                model.set_row_data(index, item);
                // Also reach the rendered (visible + grouped/sidepanel) models.
                crate::favorites::set_artist_image(window, &id, image.clone());
            }
        }
        ArtworkTarget::FavoriteLabel { index } => {
            let model = window.global::<crate::FavoritesState>().get_labels();
            if let Some(mut item) = model.row_data(index) {
                item.image = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::FavPlaylistCover { following, index, slot } => {
            let st = window.global::<crate::FavoritesState>();
            let model = if following {
                st.get_playlists_following()
            } else {
                st.get_playlists_favorites()
            };
            if let Some(mut item) = model.row_data(index) {
                let id = item.id.to_string();
                match slot {
                    0 => item.cover1 = image.clone(),
                    1 => item.cover2 = image.clone(),
                    2 => item.cover3 = image.clone(),
                    _ => item.cover4 = image.clone(),
                }
                model.set_row_data(index, item);
                // Also reach the rendered (possibly search-filtered) model.
                crate::favorites::set_playlist_cover(window, &id, slot, image.clone());
            }
        }
        ArtworkTarget::FavoriteArtistAlbum { section, index } => {
            let sections = window
                .global::<crate::FavoritesState>()
                .get_selected_artist_sections();
            if let Some(sec) = sections.row_data(section) {
                if let Some(mut item) = sec.albums.row_data(index) {
                    item.artwork = image.clone();
                    sec.albums.set_row_data(index, item);
                }
            }
        }
        _ => return false,
    }
    true
}
