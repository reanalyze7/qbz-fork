//! `activate` and `refresh` — open the section and page through it.

use std::collections::HashSet;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, PlaylistState, PlaylistSuggestionRow, PlaylistSuggestionsState, SettingsState};

use super::adaptive_artists::{extract_adaptive_artists, make_key};
use super::auto_expand::maybe_auto_expand;
use super::fetch::spawn_fetch;
use super::filter_project::{filtered_indices, project};
use super::session::{Phase, Session, SESSION};
use super::{Handle, Runtime, EXPANDED_POOL, INITIAL_POOL, VISIBLE_COUNT};

/// Launch suggestions for the open playlist: gather the seed artists + excludes
/// off the loaded Qobuz tracks, then fetch the first pool page. UI thread.
pub fn activate(window: &AppWindow, runtime: Runtime, handle: Handle) {
    // MusicBrainz opt-out: the suggestion engine resolves each seed artist via
    // MusicBrainz, so with MB off it can only ever return an empty pool. Skip the
    // fetch entirely and present the closed/empty state. The wand CTA is also
    // hidden in PlaylistView.slint when MB is off, so this is belt-and-suspenders.
    if !window.global::<SettingsState>().get_musicbrainz_enabled() {
        let state = window.global::<PlaylistSuggestionsState>();
        state.set_activated(false);
        state.set_loading(false);
        state.set_loading_more(false);
        state.set_is_empty(true);
        state.set_rows(ModelRc::new(VecModel::from(Vec::<PlaylistSuggestionRow>::new())));
        return;
    }

    let playlist_id = window
        .global::<PlaylistState>()
        .get_id()
        .parse::<u64>()
        .unwrap_or(0);
    let tracks = crate::playlist::current_tracks();
    let artists = extract_adaptive_artists(&tracks, playlist_id);
    let exclude_ids: HashSet<u64> = tracks.iter().map(|t| t.id).collect();
    let existing_keys: HashSet<String> = tracks
        .iter()
        .map(|t| {
            let artist = t.performer.as_ref().map(|p| p.name.as_str()).unwrap_or("");
            make_key(&t.title, artist)
        })
        .collect();

    {
        let mut session = SESSION.lock().unwrap();
        *session = Session {
            playlist_id,
            artists: artists.clone(),
            exclude_ids,
            existing_keys,
            ..Default::default()
        };
    }

    let state = window.global::<PlaylistSuggestionsState>();
    state.set_activated(true);
    state.set_error("".into());
    state.set_is_empty(false);
    state.set_rows(ModelRc::new(VecModel::from(Vec::<PlaylistSuggestionRow>::new())));

    // No resolvable seed artists (e.g. a fully-local playlist) -> empty, hidden.
    if playlist_id == 0 || artists.is_empty() {
        let mut session = SESSION.lock().unwrap();
        session.loaded_once = true;
        drop(session);
        state.set_loading(false);
        state.set_is_empty(true);
        return;
    }

    state.set_loading(true);
    spawn_fetch(runtime, window.as_weak(), handle, INITIAL_POOL, Phase::Initial);
}

/// Advance the visible page; on a full cycle, wrap to page 0 and (first cycle)
/// kick the EXPANDED_POOL load-more. UI thread.
pub fn refresh(window: &AppWindow, runtime: Runtime, handle: Handle) {
    let expand = {
        let mut session = SESSION.lock().unwrap();
        if session.loading {
            return;
        }
        let total_pages = filtered_indices(&session).len().div_ceil(VISIBLE_COUNT);
        if session.page + 1 < total_pages {
            session.page += 1;
            false
        } else if total_pages > 0 {
            session.page = 0;
            session.completed_cycles += 1;
            session.completed_cycles == 1
                && !session.loading_more
                && session.pool.len() < EXPANDED_POOL
        } else {
            false
        }
    };
    project(window);
    if expand {
        window.global::<PlaylistSuggestionsState>().set_loading_more(true);
        spawn_fetch(
            runtime.clone(),
            window.as_weak(),
            handle.clone(),
            EXPANDED_POOL,
            Phase::Merge,
        );
    } else {
        maybe_auto_expand(runtime, window.as_weak(), handle);
    }
}
