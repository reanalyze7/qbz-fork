//! Track search filtering + the full-state reset before opening a new album.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AlbumState, AppWindow, DiscoverSection, TrackItem};

use super::{FULL_TRACKS, PLAY_TRACKS};

/// Filter the visible track list by `query` (case-insensitive match on
/// title or artist), against the unfiltered list kept in `FULL_TRACKS`.
/// Runs on the Slint event loop.
pub fn filter_tracks(window: &AppWindow, query: &str) {
    let needle = query.trim().to_lowercase();
    let filtered: Vec<TrackItem> = FULL_TRACKS.with(|cell| {
        cell.borrow()
            .iter()
            .filter(|track| {
                needle.is_empty()
                    || track.title.as_str().to_lowercase().contains(&needle)
                    || track.artist.as_str().to_lowercase().contains(&needle)
            })
            .cloned()
            .collect()
    });
    window
        .global::<AlbumState>()
        .set_tracks(ModelRc::new(VecModel::from(filtered)));
}

/// Clear album state and show an empty track list (used when opening a new
/// album so the previous one does not flash).
pub fn reset_album(window: &AppWindow) {
    FULL_TRACKS.with(|cell| cell.borrow_mut().clear());
    PLAY_TRACKS.with(|cell| cell.borrow_mut().clear());
    let state = window.global::<AlbumState>();
    state.set_multi_select(false);
    state.set_selected_count(0);
    state.set_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_artwork(slint::Image::default());
    state.set_header_atmosphere(slint::Image::default());
    // Clear the booklet gate so the previous album's value doesn't linger.
    state.set_has_booklet(false);
    crate::booklet::clear_current_url();
    // Clear the polish carousels + external links so the previous album's
    // values don't flash before the new loads land.
    state.set_more_from_artist(DiscoverSection::default());
    state.set_show_more_from_artist(false);
    state.set_suggestions_section(DiscoverSection::default());
    state.set_show_suggestions(false);
    state.set_lastfm_suggestions_section(DiscoverSection::default());
    state.set_show_lastfm_suggestions(false);
    state.set_show_external_links(false);
    state.set_lastfm_url("".into());
    state.set_discogs_url("".into());
    state.set_musicbrainz_url("".into());
    state.set_album_fully_cached(false);
    state.set_is_favorite(false);
    state.set_pinned(false);
    state.set_favorite_loading(false);
    // Default to a Qobuz album; the local-album loader opts in.
    state.set_is_local(false);
    state.set_loading(true);
}
