//! Applying the landing payload to `LabelState`, and the JUMP-TO tabs /
//! artwork-job derivation that go with it.

use slint::{ComponentHandle, ModelRc, VecModel};

use super::to_slint::{artist_to_item, label_to_item, playlist_to_item, section, top_track_to_item};
use super::{LabelPagePayload, PLAY_TOP_TRACKS};
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, LabelState, SearchPlaylistItem, SlimItem, TrackItem};

/// Apply the landing payload to `LabelState`. Runs on the Slint event loop.
pub fn apply_label_page(window: &AppWindow, payload: LabelPagePayload) {
    PLAY_TOP_TRACKS.with(|c| *c.borrow_mut() = payload.play_tracks.clone());

    let top_tracks: Vec<TrackItem> = payload.top_tracks.iter().map(top_track_to_item).collect();
    let playlists: Vec<SearchPlaylistItem> =
        payload.playlists.iter().map(playlist_to_item).collect();
    let artists: Vec<SlimItem> = payload.artists.iter().map(artist_to_item).collect();
    let more_labels: Vec<SlimItem> = payload.more_labels.iter().map(label_to_item).collect();
    let releases = section(&qbz_i18n::t("Releases"), &payload.releases);
    let critics = section(&qbz_i18n::t("Critics' Picks"), &payload.critics);
    let jump_tabs = super::jump_tabs::build_label_jump_tabs(&payload);

    let state = window.global::<LabelState>();
    state.set_id(payload.id.into());
    state.set_name(payload.name.into());
    state.set_image_url(payload.image_url.into());
    state.set_description(payload.description.into());
    state.set_description_short(payload.description_short.into());
    state.set_description_truncated(payload.description_truncated);
    state.set_is_following(payload.is_following);
    state.set_top_tracks(ModelRc::new(VecModel::from(top_tracks)));
    state.set_releases_section(releases);
    state.set_critics_section(critics);
    state.set_playlists(ModelRc::new(VecModel::from(playlists)));
    state.set_artists(ModelRc::new(VecModel::from(artists)));
    state.set_more_labels(ModelRc::new(VecModel::from(more_labels)));
    state.set_jump_tabs(ModelRc::new(VecModel::from(jump_tabs)));
    state.set_page_loaded(true);
    state.set_loading(false);
}

/// Seed the catalog/library toggle state for the open label (the user's
/// favorite tracks + albums on it, from the session `library_by_label`
/// index). Runs on the Slint event loop, right after `apply_label_page`.
pub fn apply_label_library(window: &AppWindow, library: &crate::library_by_label::LabelLibrary) {
    let state = window.global::<LabelState>();
    state.set_library_count(library.count() as i32);
    state.set_library_tracks(crate::library_by_artist::track_items(&library.tracks));
    state.set_library_albums(crate::library_by_artist::album_items(&library.albums));
    // Every visit starts on the catalog tab (the toggle is session-owned UI
    // state, like the artist page's).
    state.set_label_tab("catalog".into());
}

/// Artwork jobs for the landing sections (top-track thumbs + carousels).
pub fn page_artwork_jobs(payload: &LabelPagePayload) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    let push = |jobs: &mut Vec<ArtworkJob>, url: &str, target: ArtworkTarget| {
        if !url.is_empty() {
            jobs.push(ArtworkJob {
                url: url.to_string(),
                target,
            });
        }
    };
    for (i, t) in payload.top_tracks.iter().enumerate() {
        push(&mut jobs, &t.artwork_url, ArtworkTarget::LabelTopTrack { index: i });
    }
    for (i, a) in payload.releases.iter().enumerate() {
        push(&mut jobs, &a.artwork_url, ArtworkTarget::LabelReleaseAlbum { index: i });
    }
    for (i, a) in payload.critics.iter().enumerate() {
        push(&mut jobs, &a.artwork_url, ArtworkTarget::LabelCriticsAlbum { index: i });
    }
    for (i, p) in payload.playlists.iter().enumerate() {
        push(&mut jobs, &p.image_url, ArtworkTarget::LabelPlaylistCover { index: i });
    }
    for (i, a) in payload.artists.iter().enumerate() {
        push(&mut jobs, &a.image_url, ArtworkTarget::LabelArtist { index: i });
    }
    for (i, l) in payload.more_labels.iter().enumerate() {
        push(&mut jobs, &l.image_url, ArtworkTarget::LabelMoreLabel { index: i });
    }
    jobs
}
