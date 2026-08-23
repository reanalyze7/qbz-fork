//! Album-view carousel arms + the Location-view artist tile — small enough
//! to not warrant their own named category file each.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::AlbumMoreFromArtist { index } => {
            let model = window.global::<crate::AlbumState>().get_more_from_artist().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::AlbumSuggestion { index } => {
            let model = window.global::<crate::AlbumState>().get_suggestions_section().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::AlbumLastfmSuggestion { index } => {
            let model = window
                .global::<crate::AlbumState>()
                .get_lastfm_suggestions_section()
                .albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LocationArtist { index } => {
            let model = window.global::<crate::LocationViewState>().get_artists();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        _ => return false,
    }
    true
}
