//! Pure Slint-state pushing for the location-scene artist grid.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::{ArtistCard, LocationData};
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, LocationViewState, SlimItem};

fn to_item(card: ArtistCard) -> SlimItem {
    // SlimItem mapping for the app-wide ArtistGridCard: the genres ride the
    // subtitle (second row under the name), follow/pin seed from the
    // disk-backed caches so the chips are right from first paint.
    SlimItem {
        following: card
            .qobuz_id
            .parse::<u64>()
            .map(crate::fav_cache::is_artist_favorite)
            .unwrap_or(false),
        is_pinned: crate::pinned::is_pinned("artist", &card.qobuz_id),
        id: card.qobuz_id.into(),
        title: card.name.into(),
        subtitle: card.genres_line.into(),
        artwork_url: card.image_url.into(),
        ..Default::default()
    }
}

pub fn apply_scene(window: &AppWindow, data: LocationData) {
    let items: Vec<SlimItem> = data.artists.into_iter().map(to_item).collect();
    let state = window.global::<LocationViewState>();
    state.set_scene_label(data.scene_label.into());
    state.set_genre_summary(data.genre_summary.into());
    state.set_artists(ModelRc::new(VecModel::from(items)));
    state.set_total(data.total as i32);
    state.set_loading(false);
}

pub fn append_scene(window: &AppWindow, artists: Vec<ArtistCard>, total: usize) {
    let state = window.global::<LocationViewState>();
    let model = state.get_artists();
    let mut combined: Vec<SlimItem> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();
    combined.extend(artists.into_iter().map(to_item));
    state.set_artists(ModelRc::new(VecModel::from(combined)));
    state.set_total(total as i32);
    state.set_load_more_loading(false);
}

pub fn reset_scene(window: &AppWindow) {
    let state = window.global::<LocationViewState>();
    state.set_scene_label("".into());
    state.set_genre_summary("".into());
    state.set_artists(ModelRc::new(VecModel::from(Vec::<SlimItem>::new())));
    state.set_total(0);
    state.set_loading(true);
    state.set_load_more_loading(false);
}

/// Artwork jobs for the scene artist grid (the candidates' Qobuz
/// thumbnails).
pub fn artwork_jobs(data: &LocationData) -> Vec<ArtworkJob> {
    data.artists
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.image_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.image_url.clone(),
            target: ArtworkTarget::LocationArtist { index: i },
        })
        .collect()
}
