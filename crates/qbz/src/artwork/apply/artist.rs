//! Artist-page + Musician arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::{AppWindow, ArtistState};

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::ArtistRelease { section_idx, album_idx } => {
            let sections = window.global::<ArtistState>().get_release_sections();
            let Some(section) = sections.row_data(section_idx) else {
                return true;
            };
            let Some(mut item) = section.albums.row_data(album_idx) else {
                return true;
            };
            item.artwork = image.clone();
            section.albums.set_row_data(album_idx, item);
        }
        ArtworkTarget::ArtistLastRelease => {
            let mut item = window.global::<ArtistState>().get_last_release();
            item.artwork = image.clone();
            window.global::<ArtistState>().set_last_release(item);
        }
        ArtworkTarget::ArtistReleasesAlbum { index } => {
            let model = window.global::<crate::ArtistReleasesState>().get_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ArtistPlaylistCover { index } => {
            let model = window.global::<ArtistState>().get_playlists();
            if let Some(mut item) = model.row_data(index) {
                item.cover1 = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ArtistLibraryTrack { index } => {
            let model = window.global::<ArtistState>().get_library_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ArtistLibraryAlbum { index } => {
            let model = window.global::<ArtistState>().get_library_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ArtistStory { index } => {
            let model = window.global::<ArtistState>().get_stories();
            if let Some(mut item) = model.row_data(index) {
                item.image = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ArtistTopTrack { index } => {
            let model = window.global::<ArtistState>().get_top_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::MusicianAppearance { index } => {
            let model = window.global::<crate::MusicianState>().get_appearances();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        _ => return false,
    }
    true
}
