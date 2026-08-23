use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artist::cache::{FULL_APPEARS_ON, FULL_RELEASE_SECTIONS, FULL_TOP_TRACKS};
use crate::{AlbumCardItem, AppWindow, ArtistReleaseSection, ArtistState, TrackItem};

/// Filter the visible Popular Tracks (title or artist substring) and
/// release-section albums (title substring) against `query`. An empty
/// query restores the full unfiltered view. Runs on the Slint event
/// loop; called by ArtistActions::on_search from main.rs.
pub fn filter_artist(window: &AppWindow, query: &str) {
    let needle = query.trim().to_lowercase();
    let filtered_tracks: Vec<TrackItem> = FULL_TOP_TRACKS.with(|cell| {
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
    let filtered_sections: Vec<ArtistReleaseSection> = FULL_RELEASE_SECTIONS.with(|cell| {
        cell.borrow()
            .iter()
            .filter_map(|section| {
                let kept: Vec<AlbumCardItem> = section
                    .albums
                    .iter()
                    .filter(|album| {
                        needle.is_empty()
                            || album.title.as_str().to_lowercase().contains(&needle)
                    })
                    .collect();
                if kept.is_empty() {
                    return None;
                }
                Some(ArtistReleaseSection {
                    release_type: section.release_type.clone(),
                    title: section.title.clone(),
                    albums: ModelRc::new(VecModel::from(kept)),
                    // No load-more while a search filter is active (it would
                    // append unfiltered items); restore on empty query.
                    has_more: if needle.is_empty() { section.has_more } else { false },
                    sort_by: section.sort_by.clone(),
                })
            })
            .collect()
    });

    let filtered_appears: Vec<TrackItem> = FULL_APPEARS_ON.with(|cell| {
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

    let state = window.global::<ArtistState>();
    state.set_top_tracks(ModelRc::new(VecModel::from(filtered_tracks)));
    state.set_appears_on(ModelRc::new(VecModel::from(filtered_appears)));
    state.set_release_sections(ModelRc::new(VecModel::from(filtered_sections)));
}
