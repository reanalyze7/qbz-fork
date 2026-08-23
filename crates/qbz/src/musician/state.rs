//! Slint-state-application + artwork-job-building half of the
//! MusicianPageView controller. No network access here.

use qbz_integrations::musicbrainz::MusicianConfidence;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, MusicianAppearanceItem, MusicianState};

use super::load::{AppearanceData, MusicianData};

/// Apply the freshly loaded musician page to MusicianState.
pub fn apply_musician(window: &AppWindow, data: MusicianData) {
    let items: Vec<MusicianAppearanceItem> = data
        .appearances
        .into_iter()
        .map(|a| MusicianAppearanceItem {
            album_id: a.album_id.into(),
            album_title: a.album_title.into(),
            artist_name: a.artist_name.into(),
            year: a.year.into(),
            role_on_album: a.role_on_album.into(),
            artwork_url: a.artwork_url.into(),
            artwork: slint::Image::default(),
        })
        .collect();
    let state = window.global::<MusicianState>();
    state.set_name(data.name.into());
    state.set_role(data.role.into());
    state.set_confidence(confidence_label(data.confidence).into());
    state.set_appearances(ModelRc::new(VecModel::from(items)));
    state.set_total(data.total as i32);
    state.set_loading(false);
}

/// Append a freshly fetched page of appearances onto the existing
/// model. Called by the MusicianActions::load-more handler.
pub fn append_appearances(
    window: &AppWindow,
    appearances: Vec<AppearanceData>,
    total: usize,
) {
    let state = window.global::<MusicianState>();
    let model = state.get_appearances();
    let mut combined: Vec<MusicianAppearanceItem> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();
    for a in appearances {
        combined.push(MusicianAppearanceItem {
            album_id: a.album_id.into(),
            album_title: a.album_title.into(),
            artist_name: a.artist_name.into(),
            year: a.year.into(),
            role_on_album: a.role_on_album.into(),
            artwork_url: a.artwork_url.into(),
            artwork: slint::Image::default(),
        });
    }
    state.set_appearances(ModelRc::new(VecModel::from(combined)));
    state.set_total(total as i32);
    state.set_load_more_loading(false);
}

pub fn reset_musician(window: &AppWindow) {
    let state = window.global::<MusicianState>();
    state.set_name("".into());
    state.set_role("".into());
    state.set_confidence("".into());
    state.set_appearances(ModelRc::new(VecModel::from(
        Vec::<MusicianAppearanceItem>::new(),
    )));
    state.set_total(0);
    state.set_loading(true);
    state.set_load_more_loading(false);
}

/// Artwork download jobs for the appearance grid — same pipeline
/// the Discover album cards use, so covers fill in progressively.
pub fn artwork_jobs(data: &MusicianData) -> Vec<ArtworkJob> {
    data.appearances
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.artwork_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.artwork_url.clone(),
            target: ArtworkTarget::MusicianAppearance { index: i },
        })
        .collect()
}

fn confidence_label(c: MusicianConfidence) -> &'static str {
    match c {
        MusicianConfidence::Confirmed => "confirmed",
        MusicianConfidence::Contextual => "contextual",
        MusicianConfidence::Weak => "weak",
        MusicianConfidence::None => "none",
    }
}
