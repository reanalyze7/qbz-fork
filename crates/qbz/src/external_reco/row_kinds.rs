//! Row-kind enums shared across the apply/artist-rails submodules, plus the
//! P7 title-adjacent track-id lookup.
use slint::ComponentHandle;

use slint::Model;

use crate::{AppWindow, ExternalRecoState};

#[derive(Clone, Copy)]
pub(super) enum ArtistRow {
    RecArtistsCommon,
    RecArtistsRecent,
    TopArtists,
}
#[derive(Clone, Copy)]
pub(super) enum AlbumRow {
    RecAlbums,
    FreshReleases,
    DeepCuts,
    TopAlbums,
}
#[derive(Clone, Copy)]
pub(super) enum TrackRow {
    WeeklyExploration,
    WeeklyJams,
}

/// Read the backing Qobuz track ids of one external-reco Weekly TRACK row
/// (Weekly Exploration / Weekly Jams) for the P7 title-adjacent buttons.
/// Returns the whole backing list (not just the 24 visible), in row order.
pub fn list_track_ids(window: &AppWindow, section: &str) -> Vec<u64> {
    let s = window.global::<ExternalRecoState>();
    let model = match section {
        "weekly-exploration" => s.get_weekly_exploration(),
        "weekly-jams" => s.get_weekly_jams(),
        _ => return Vec::new(),
    };
    model
        .iter()
        .filter_map(|it| it.id.as_str().parse::<u64>().ok())
        .collect()
}
