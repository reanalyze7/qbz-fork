use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artist::apply_sections::map_release_sections;
use crate::artist::cache::{FULL_APPEARS_ON, FULL_RELEASE_SECTIONS, FULL_TOP_TRACKS, LOADED_PAGES};
use crate::artist::data::ArtistData;
use crate::artist::jump_tabs::build_jump_tabs;
use crate::artist::track_map::{card_to_item, playlist_to_item, track_data_to_item};
use crate::{
    AppWindow, ArtistState, LabelEntry, SearchPlaylistItem,
    SimilarEntry, TrackItem,
};

/// Apply artist data to the `ArtistState` global. Runs on the Slint event loop.
pub fn apply_artist(window: &AppWindow, data: ArtistData) {
    // Capture counts before we move the data so the JUMP TO tab
    // anchor-y estimates can use them.
    let top_tracks_count = data.top_tracks.len();
    let section_counts: Vec<(String, usize)> = data
        .release_sections
        .iter()
        .map(|s| (s.title.clone(), s.cards.len()))
        .collect();

    let appears_on_count = data.appears_on.len();
    let has_last_release = data.last_release.is_some();

    let top_tracks: Vec<TrackItem> = data
        .top_tracks
        .into_iter()
        .map(track_data_to_item)
        .collect();
    let appears_on: Vec<TrackItem> = data
        .appears_on
        .into_iter()
        .map(track_data_to_item)
        .collect();
    let last_release_item = data
        .last_release
        .filter(|c| !crate::artist_blacklist::card_blacklisted(&c.id, &c.artist_id))
        .map(card_to_item)
        .unwrap_or_default();
    let release_sections = map_release_sections(data.release_sections);

    // Reset the per-bucket page counters to 1 (the embedded bucket).
    LOADED_PAGES.with(|cell| {
        let mut m = cell.borrow_mut();
        m.clear();
        for s in &release_sections {
            m.insert(s.release_type.to_string(), 1);
        }
    });

    let jump_tabs = build_jump_tabs(
        top_tracks_count,
        has_last_release,
        &section_counts,
        appears_on_count,
    );

    let labels: Vec<LabelEntry> = data
        .labels
        .into_iter()
        .map(|label| LabelEntry {
            id: label.id.into(),
            name: label.name.into(),
        })
        .collect();
    let similar_artists: Vec<SimilarEntry> = data
        .similar_artists
        .into_iter()
        .map(|sa| SimilarEntry {
            id: sa.id.into(),
            name: sa.name.into(),
        })
        .collect();
    let playlists: Vec<SearchPlaylistItem> =
        data.playlists.iter().map(playlist_to_item).collect();

    // Cache the full models on the UI thread so the in-page search
    // can rebuild filtered views without re-fetching the artist.
    FULL_TOP_TRACKS.with(|cell| {
        *cell.borrow_mut() = top_tracks.clone();
    });
    FULL_APPEARS_ON.with(|cell| {
        *cell.borrow_mut() = appears_on.clone();
    });
    FULL_RELEASE_SECTIONS.with(|cell| {
        *cell.borrow_mut() = release_sections.clone();
    });

    let has_custom_image = crate::custom_artwork::artist_image(&data.name).is_some();
    let artwork_url = data.artwork_url.clone();

    let state = window.global::<ArtistState>();
    state.set_name(data.name.into());
    state.set_artwork_url(artwork_url.into());
    state.set_has_custom_image(has_custom_image);
    state.set_bio(data.bio.into());
    state.set_bio_short(data.bio_short.into());
    state.set_bio_truncated(data.bio_truncated);
    state.set_bio_source(data.bio_source.into());
    state.set_top_tracks(ModelRc::new(VecModel::from(top_tracks)));
    state.set_has_last_release(has_last_release);
    state.set_last_release(last_release_item);
    state.set_appears_on(ModelRc::new(VecModel::from(appears_on)));
    state.set_release_sections(ModelRc::new(VecModel::from(release_sections)));
    state.set_labels(ModelRc::new(VecModel::from(labels)));
    state.set_similar_artists(ModelRc::new(VecModel::from(similar_artists)));
    state.set_playlists(ModelRc::new(VecModel::from(playlists)));
    state.set_jump_tabs(ModelRc::new(VecModel::from(jump_tabs)));
}
