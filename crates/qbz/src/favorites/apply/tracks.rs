use qbz_models::Track;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::favorites::derive::derive_tracks;
use crate::favorites::mapping::TrackCard;
use crate::favorites::random::FAV_CURRENT;
use crate::favorites::FAV_TRACK_GENRE;
use crate::{AppWindow, FavoritesState, TrackItem};

pub(crate) fn apply_tracks(window: &AppWindow, items: Vec<TrackCard>, play: Vec<Track>, total: usize) {
    let state = window.global::<FavoritesState>();
    if let Ok(mut current) = FAV_CURRENT.lock() {
        *current = play;
    }
    FAV_TRACK_GENRE.with(|m| {
        *m.borrow_mut() = items.iter().map(|t| (t.id.clone(), t.genre.clone())).collect();
    });
    let rows: Vec<TrackItem> = items
        .into_iter()
        .map(|t| TrackItem {
            is_blacklisted: crate::artist_blacklist::stamp_row(
                "qobuz",
                &[t.artist_id.as_str(), t.composer_id.as_str()],
                Some(t.album_id.as_str()),
            ),
            id: t.id.clone().into(),
            number: "".into(),
            title: t.title.into(),
            artist: t.artist.into(),
            album: t.album.into(),
            duration: t.duration.into(),
            quality_tier: t.quality_tier.into(),
            quality_detail: t.quality_detail.into(),
            explicit: t.explicit,
            selected: false,
            artwork_url: t.artwork_url.into(),
            artwork: slint::Image::default(),
            // Everything in the Favorites > Tracks tab is, by
            // definition, a favorite.
            is_favorite: true,
            artist_id: t.artist_id.into(),
            album_id: t.album_id.into(),
            removing: false,
            cache_status: if crate::offline_cache::is_cached(&t.id) { 3 } else { 0 },
            cache_progress: 0.0,
            source: "qobuz".into(),
            unlocking: false,
            // Disc grouping is album-detail only; flat lists carry none.
            disc_header_number: 0,
            // Work grouping is album-detail only too.
            work_header: "".into(),
            work_composer_name: "".into(),
            work_composer_id: "".into(),
        })
        .collect();
    // `tracks` is the full set the artwork pipeline targets;
    // `tracks-visible` (what the list renders) shares the same
    // model until a search filter forks it, so artwork stays live.
    let model = ModelRc::new(VecModel::from(rows));
    state.set_tracks(model.clone());
    state.set_tracks_visible(model);
    state.set_tracks_total(total as i32);
    state.set_tracks_search("".into());
    // Apply the (persisted) group mode to the freshly loaded set.
    derive_tracks(window);
}
