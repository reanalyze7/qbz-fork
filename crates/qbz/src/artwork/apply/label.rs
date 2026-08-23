//! Label-page arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::LabelAlbum { index } => {
            let model = window.global::<crate::LabelState>().get_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelTopTrack { index } => {
            let model = window.global::<crate::LabelState>().get_top_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelReleaseAlbum { index } => {
            let model = window.global::<crate::LabelState>().get_releases_section().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelLibraryAlbum { index } => {
            let model = window.global::<crate::LabelState>().get_library_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelLibraryTrack { index } => {
            let model = window.global::<crate::LabelState>().get_library_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelCriticsAlbum { index } => {
            let model = window.global::<crate::LabelState>().get_critics_section().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelPlaylistCover { index } => {
            let model = window.global::<crate::LabelState>().get_playlists();
            if let Some(mut item) = model.row_data(index) {
                item.cover1 = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelArtist { index } => {
            let model = window.global::<crate::LabelState>().get_artists();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LabelMoreLabel { index } => {
            let model = window.global::<crate::LabelState>().get_more_labels();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        _ => return false,
    }
    true
}
