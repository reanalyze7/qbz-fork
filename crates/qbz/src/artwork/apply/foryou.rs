//! For You tab arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::ForYouReleaseWatch { index } => {
            let model = window.global::<crate::ForYouState>().get_release_watch().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouRecentAlbum { index } => {
            let model = window.global::<crate::ForYouState>().get_recent_albums().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouRecentTrack { index } => {
            let model = window.global::<crate::ForYouState>().get_recent_tracks();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouTopArtist { index } => {
            let model = window.global::<crate::ForYouState>().get_top_artists();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouToFollow { index } => {
            let model = window.global::<crate::ForYouState>().get_artists_to_follow();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouMoreFromLibrary { index } => {
            let model = window.global::<crate::ForYouState>().get_more_from_library().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouRediscover { index } => {
            let model = window.global::<crate::ForYouState>().get_rediscover().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouFavoriteAlbum { index } => {
            let model = window.global::<crate::ForYouState>().get_favorite_albums().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouMostPlayedAlbum { index } => {
            let model = window.global::<crate::ForYouState>().get_most_played_albums().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ForYouSpotlightArtist => {
            window.global::<crate::ForYouState>().set_spotlight_image(image.clone());
        }
        ArtworkTarget::ForYouSpotlightAlbum { index } => {
            let model = window.global::<crate::ForYouState>().get_spotlight_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        _ => return false,
    }
    true
}
