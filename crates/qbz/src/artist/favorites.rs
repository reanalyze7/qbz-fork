use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artist::cache::{FULL_RELEASE_SECTIONS, LOADED_PAGES};
use crate::{
    AlbumCardItem, AppWindow, ArtistReleaseSection, ArtistState, JumpNavTab, LabelEntry,
    SearchPlaylistItem, SimilarEntry, StoryItem, TrackItem,
};

/// Flip the favorite heart on every discography card matching `album_id`:
/// the visible release sections, the last-release highlight card, and the
/// FULL section cache the in-page search rebuilds from (without the cache
/// pass, clearing a search filter would restore a stale heart). Called by
/// `main::set_album_row_favorite` whenever an album favorite toggles.
pub fn set_release_card_favorite(window: &AppWindow, album_id: &str, favorite: bool) {
    let flip = |model: &ModelRc<AlbumCardItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == album_id && item.is_favorite != favorite {
                    item.is_favorite = favorite;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    let state = window.global::<ArtistState>();
    let sections = state.get_release_sections();
    for s in 0..sections.row_count() {
        if let Some(section) = sections.row_data(s) {
            flip(&section.albums);
        }
    }
    // "In library" grid (the catalog/library header toggle).
    flip(&state.get_library_albums());
    let mut last = state.get_last_release();
    if last.id == album_id && last.is_favorite != favorite {
        last.is_favorite = favorite;
        state.set_last_release(last);
    }
    // The FULL cache shares the visible sections' ModelRc while no filter is
    // active (the `!= favorite` guard makes the second pass a no-op then);
    // under a filter it is a separate copy that must be flipped too.
    FULL_RELEASE_SECTIONS.with(|cell| {
        for section in cell.borrow().iter() {
            flip(&section.albums);
        }
    });
}

/// Pin twin of [`set_release_card_favorite`]: flip the `is-pinned` badge on
/// every artist-page release card matching `album_id` (sections +
/// last-release + the in-page-search FULL cache).
pub fn set_release_card_pinned(window: &AppWindow, album_id: &str, pinned: bool) {
    let flip = |model: &ModelRc<AlbumCardItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == album_id && item.is_pinned != pinned {
                    item.is_pinned = pinned;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    let state = window.global::<ArtistState>();
    let sections = state.get_release_sections();
    for s in 0..sections.row_count() {
        if let Some(section) = sections.row_data(s) {
            flip(&section.albums);
        }
    }
    // "In library" grid (the catalog/library header toggle).
    flip(&state.get_library_albums());
    let mut last = state.get_last_release();
    if last.id == album_id && last.is_pinned != pinned {
        last.is_pinned = pinned;
        state.set_last_release(last);
    }
    // The FULL cache shares the visible sections' ModelRc while no filter is
    // active (the `!= pinned` guard makes the second pass a no-op then);
    // under a filter it is a separate copy that must be flipped too.
    FULL_RELEASE_SECTIONS.with(|cell| {
        for section in cell.borrow().iter() {
            flip(&section.albums);
        }
    });
}

/// Clear artist state before loading a new artist.
pub fn reset_artist(window: &AppWindow) {
    let state = window.global::<ArtistState>();
    state.set_top_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_appears_on(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_has_last_release(false);
    state.set_last_release(AlbumCardItem::default());
    state.set_stories(ModelRc::new(VecModel::from(Vec::<StoryItem>::new())));
    state.set_stories_loading(true);
    state.set_release_sections(ModelRc::new(VecModel::from(Vec::<ArtistReleaseSection>::new())));
    LOADED_PAGES.with(|cell| cell.borrow_mut().clear());
    state.set_labels(ModelRc::new(VecModel::from(Vec::<LabelEntry>::new())));
    state.set_similar_artists(ModelRc::new(VecModel::from(Vec::<SimilarEntry>::new())));
    state.set_jump_tabs(ModelRc::new(VecModel::from(Vec::<JumpNavTab>::new())));
    state.set_artwork(slint::Image::default());
    state.set_header_atmosphere(slint::Image::default());
    state.set_name("".into());
    state.set_bio("".into());
    state.set_bio_source("".into());
    state.set_top_tracks_multi_select(false);
    state.set_top_tracks_selected_count(0);
    state.set_is_blacklisted(false);
    state.set_playlists(ModelRc::new(VecModel::from(Vec::<SearchPlaylistItem>::new())));
    // Catalog/library toggle — reset so an artist WITHOUT library items (apply
    // only seeds these when the index has the artist) never shows the previous
    // artist's count/subset.
    state.set_artist_tab("catalog".into());
    state.set_library_count(0);
    state.set_library_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_library_albums(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    state.set_loading(true);
}
