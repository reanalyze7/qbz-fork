use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::items::{album_item, artist_item, playlist_item, track_item};
use crate::search::rows::{MostPopularRow, SearchData};
use crate::{AlbumCardItem, AppWindow, SearchPlaylistItem, SearchState, SlimItem, TrackItem};

/// Apply search results to the `SearchState` global. Runs on the Slint
/// event loop.
pub fn apply_search(window: &AppWindow, data: SearchData) {
    let state = window.global::<SearchState>();
    state.set_query(data.query.into());

    let albums: Vec<AlbumCardItem> = data.albums.into_iter().map(album_item).collect();
    let tracks: Vec<TrackItem> = data.tracks.into_iter().map(track_item).collect();
    let artists: Vec<SlimItem> = data.artists.into_iter().map(artist_item).collect();
    let playlists: Vec<SearchPlaylistItem> =
        data.playlists.into_iter().map(playlist_item).collect();
    // Carousel variant of the artists list — drops the first entry when
    // it equals the most-popular hero, so the All tab does not duplicate
    // the Top result alongside the carousel.
    let mp_id = if let MostPopularRow::Artist(ref mp) = data.most_popular {
        Some(mp.id.clone())
    } else {
        None
    };
    let artists_carousel: Vec<SlimItem> = match (mp_id, artists.first()) {
        (Some(id), Some(first)) if first.id == id.as_str() => artists[1..].to_vec(),
        _ => artists.clone(),
    };

    state.set_albums(ModelRc::new(VecModel::from(albums)));
    state.set_tracks(ModelRc::new(VecModel::from(tracks)));
    state.set_artists(ModelRc::new(VecModel::from(artists)));
    state.set_artists_carousel(ModelRc::new(VecModel::from(artists_carousel)));
    state.set_playlists(ModelRc::new(VecModel::from(playlists)));

    state.set_albums_total(data.albums_total as i32);
    state.set_tracks_total(data.tracks_total as i32);
    state.set_artists_total(data.artists_total as i32);
    state.set_playlists_total(data.playlists_total as i32);

    // Default the hero quality label off; only the track branch sets it.
    state.set_most_popular_quality_label("".into());
    match data.most_popular {
        MostPopularRow::Album(row) => {
            state.set_most_popular_kind("album".into());
            state.set_most_popular_album(album_item(row));
        }
        MostPopularRow::Artist(row) => {
            state.set_most_popular_kind("artist".into());
            state.set_most_popular_artist(artist_item(row));
        }
        MostPopularRow::Track(row) => {
            state.set_most_popular_kind("track".into());
            state.set_most_popular_quality_label(row.quality_label.clone().into());
            state.set_most_popular_track(track_item(row));
        }
        MostPopularRow::None => {
            state.set_most_popular_kind("".into());
        }
    }

    recompute_hi_res_filtered(window);
}

/// Re-derive `SearchState.filtered-albums` / `filtered-tracks` — the
/// Hi-Res-only ("hires" quality tier) subset of the current `albums` /
/// `tracks` lists. Called after every mutation of those two lists (initial
/// load, load-more append, searchType re-query, reset) so the filtered pair
/// is always current, regardless of whether the toggle is on. The view
/// (SearchResultsView.slint) picks between the raw and filtered pair at
/// bind time via `SearchState.hi-res-only`.
///
/// Client-side by necessity: Qobuz's `search_albums`/`search_tracks`
/// endpoints (qbz-qobuz::client) take query/limit/offset/searchType only —
/// no quality parameter — so there is no server-side filter to request.
pub fn recompute_hi_res_filtered(window: &AppWindow) {
    let state = window.global::<SearchState>();
    let albums: Vec<AlbumCardItem> = state
        .get_albums()
        .iter()
        .filter(|a| a.quality_tier.as_str() == "hires")
        .collect();
    let tracks: Vec<TrackItem> = state
        .get_tracks()
        .iter()
        .filter(|t| t.quality_tier.as_str() == "hires")
        .collect();
    state.set_filtered_albums(ModelRc::new(VecModel::from(albums)));
    state.set_filtered_tracks(ModelRc::new(VecModel::from(tracks)));
}

/// Clear search state and show the loading state (used when starting a new
/// search so the previous results do not flash).
pub fn reset_search(window: &AppWindow) {
    let state = window.global::<SearchState>();
    state.set_albums(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    state.set_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_artists(ModelRc::new(VecModel::from(Vec::<SlimItem>::new())));
    state.set_playlists(ModelRc::new(VecModel::from(Vec::<SearchPlaylistItem>::new())));
    state.set_albums_total(0);
    state.set_tracks_total(0);
    state.set_artists_total(0);
    state.set_playlists_total(0);
    state.set_most_popular_kind("".into());
    state.set_most_popular_quality_label("".into());
    state.set_filter_index(0);
    // A fresh search clears the Hi-Res toggle too — same reset-on-new-query
    // behavior as the searchType filter above, so a filter set on a prior
    // query never silently hides results on the next one.
    state.set_hi_res_only(false);
    state.set_loading(true);
    recompute_hi_res_filtered(window);
}
