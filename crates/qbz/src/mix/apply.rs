//! `apply_mix` / `reset_mix`: push mix state into `MixState`, and
//! `artwork_jobs` for the cover-load pass.

use qbz_models::Track;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, MixState, TrackItem};

use super::item::{to_item, total_duration};
use super::state::{mix_meta, CURRENT_MIX};

pub fn apply_mix(window: &AppWindow, kind: &str, tracks: Vec<Track>) {
    let (title, subtitle) = mix_meta(kind);
    let items: Vec<TrackItem> = tracks.iter().map(to_item).collect();
    let count = tracks.len() as i32;
    let duration = total_duration(&tracks);
    if let Ok(mut cur) = CURRENT_MIX.lock() {
        *cur = tracks;
    }
    let state = window.global::<MixState>();
    state.set_kind(kind.into());
    state.set_title(title.into());
    state.set_subtitle(subtitle.into());
    state.set_tracks(ModelRc::new(VecModel::from(items)));
    state.set_track_count(count);
    state.set_total_duration(duration.into());
    state.set_loading(false);
}

pub fn reset_mix(window: &AppWindow, kind: &str) {
    let (title, subtitle) = mix_meta(kind);
    let state = window.global::<MixState>();
    state.set_kind(kind.into());
    state.set_title(title.into());
    state.set_subtitle(subtitle.into());
    state.set_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_track_count(0);
    state.set_total_duration("".into());
    state.set_loading(true);
}

pub fn artwork_jobs(tracks: &[Track]) -> Vec<ArtworkJob> {
    tracks
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            t.album
                .as_ref()
                .and_then(|a| a.image.best().cloned())
                .filter(|u| !u.is_empty())
                .map(|url| ArtworkJob {
                    url,
                    target: ArtworkTarget::MixTrack { index: i },
                })
        })
        .collect()
}
