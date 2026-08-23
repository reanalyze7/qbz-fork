//! Pure pool filtering + the UI-thread projection onto
//! `PlaylistSuggestionsState`. `project` is what every other file calls
//! after mutating `SESSION`.

use std::collections::HashSet;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, PlaylistSuggestionRow, PlaylistSuggestionsState};

use super::adaptive_artists::{make_key, mmss};
use super::session::{Session, SESSION};
use super::VISIBLE_COUNT;

/// Compute the indices into `session.pool` that survive filtering, in pool
/// order: not dismissed, not excluded, not a duplicate of an existing playlist
/// track, and de-duplicated within the pool by `title|artist`.
pub(super) fn filtered_indices(session: &Session) -> Vec<usize> {
    let dismissed = crate::playlist_suggestions_dismiss::dismissed_for_playlist(session.playlist_id);
    let mut seen_keys: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for (idx, item) in session.pool.iter().enumerate() {
        if dismissed.contains(&item.track_id) || session.exclude_ids.contains(&item.track_id) {
            continue;
        }
        let key = make_key(&item.title, &item.artist_name);
        if session.existing_keys.contains(&key) {
            continue;
        }
        if !seen_keys.insert(key) {
            continue;
        }
        out.push(idx);
    }
    out
}

fn to_row(track: &qbz_reco::SuggestedTrack) -> PlaylistSuggestionRow {
    PlaylistSuggestionRow {
        track_id: track.track_id.to_string().into(),
        title: track.title.clone().into(),
        artist_name: track.artist_name.clone().into(),
        artist_id: track
            .artist_id
            .map(|id| id.to_string())
            .unwrap_or_default()
            .into(),
        album_title: track.album_title.clone().into(),
        album_id: track.album_id.clone().into(),
        artwork_url: track.album_image_url.clone().unwrap_or_default().into(),
        artwork: slint::Image::default(),
        duration_label: mmss(track.duration).into(),
        reason: track.reason.clone().unwrap_or_default().into(),
        adding: false,
        added: false,
    }
}

/// Project the current session onto `PlaylistSuggestionsState` (visible page +
/// flags) and fire the row-cover artwork jobs. UI thread.
pub(super) fn project(window: &AppWindow) {
    let (rows, has_more, is_empty, loading, loading_more, jobs): (
        Vec<PlaylistSuggestionRow>,
        bool,
        bool,
        bool,
        bool,
        Vec<ArtworkJob>,
    ) = {
        let mut session = SESSION.lock().unwrap();
        let filtered = filtered_indices(&session);
        let total_pages = filtered.len().div_ceil(VISIBLE_COUNT);
        // Clamp + persist the page so a dismiss/add that shrinks the pool below
        // the current window snaps back to the last real page (no empty view).
        session.page = session.page.min(total_pages.saturating_sub(1));
        let page = session.page;
        let start = page * VISIBLE_COUNT;
        let visible: Vec<&qbz_reco::SuggestedTrack> = filtered
            .iter()
            .skip(start)
            .take(VISIBLE_COUNT)
            .map(|&i| &session.pool[i])
            .collect();
        let mut jobs = Vec::new();
        let rows: Vec<PlaylistSuggestionRow> = visible
            .iter()
            .enumerate()
            .map(|(idx, track)| {
                if !track.album_image_url.as_deref().unwrap_or("").is_empty() {
                    jobs.push(ArtworkJob {
                        url: track.album_image_url.clone().unwrap_or_default(),
                        target: ArtworkTarget::PlaylistSuggestionCover { idx },
                    });
                }
                to_row(track)
            })
            .collect();
        let has_more = page + 1 < total_pages;
        let is_empty = filtered.is_empty() && !session.loading && session.loaded_once;
        (
            rows,
            has_more,
            is_empty,
            session.loading,
            session.loading_more,
            jobs,
        )
    };

    let state = window.global::<PlaylistSuggestionsState>();
    state.set_rows(ModelRc::new(VecModel::from(rows)));
    state.set_has_more(has_more);
    state.set_is_empty(is_empty);
    state.set_loading(loading);
    state.set_loading_more(loading_more);

    if !jobs.is_empty() {
        if let Some(cache) = crate::artwork::shared_cache() {
            crate::artwork::spawn_loads(jobs, window.as_weak(), cache);
        }
    }
}
