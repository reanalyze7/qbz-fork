//! Mix / Playlist / Playlist Manager arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::MixTrack { index } => {
            let model = window.global::<crate::MixState>().get_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::PlaylistTrack { index } => {
            // Resolve into the stable FULL_ITEMS + the visible row (by
            // id) so sorting/filtering keeps the artwork.
            crate::playlist::set_track_artwork(window, index, image.clone());
        }
        ArtworkTarget::PlaylistCover => {
            window.global::<crate::PlaylistState>().set_cover(image.clone());
        }
        ArtworkTarget::PmPlaylistCover { index, slot } => {
            let model = window.global::<crate::PlaylistManagerState>().get_playlists();
            if let Some(mut item) = model.row_data(index) {
                match slot {
                    0 => item.cover1 = image.clone(),
                    1 => item.cover2 = image.clone(),
                    2 => item.cover3 = image.clone(),
                    3 => item.cover4 = image.clone(),
                    _ => return true,
                }
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::PmTreeCover { index, slot } => {
            let model = window.global::<crate::PlaylistManagerState>().get_tree();
            if let Some(mut row) = model.row_data(index) {
                match slot {
                    0 => row.playlist.cover1 = image.clone(),
                    1 => row.playlist.cover2 = image.clone(),
                    2 => row.playlist.cover3 = image.clone(),
                    3 => row.playlist.cover4 = image.clone(),
                    _ => return true,
                }
                model.set_row_data(index, row);
            }
        }
        _ => return false,
    }
    true
}
