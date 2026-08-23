//! 4th-tab "Recommendations" arms (external-reco engine).

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::ExtRecoRecArtistCommon { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_rec_artists_common();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoRecArtistRecent { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_rec_artists_recent();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoTopArtist { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_top_artists();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoRecAlbum { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_rec_albums().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoFreshAlbum { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_fresh_releases().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoDeepAlbum { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_deep_cut_albums().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoTopAlbum { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_top_albums().albums;
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoWeeklyExploration { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_weekly_exploration();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::ExtRecoWeeklyJams { index } => {
            let model = window.global::<crate::ExternalRecoState>().get_weekly_jams();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        _ => return false,
    }
    true
}
