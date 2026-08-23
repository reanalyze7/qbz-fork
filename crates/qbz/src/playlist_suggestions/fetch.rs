//! Async fetch/merge/error handling. `spawn_fetch` and `maybe_auto_expand`
//! (`auto_expand.rs`) call each other recursively (`maybe_auto_expand` →
//! `spawn_fetch` → on success → `maybe_auto_expand`) — kept as a tight pair of
//! sibling files, both `pub(super)`, to avoid a circular-import headache.

use std::collections::HashSet;

use slint::ComponentHandle;

use crate::PlaylistSuggestionsState;

use super::auto_expand::maybe_auto_expand;
use super::filter_project::project;
use super::session::{Phase, SESSION};
use super::{Handle, Runtime, Weak, MAX_POOL};

/// Fetch a pool page and (Initial) replace or (Merge) merge it, then re-project.
pub(super) fn spawn_fetch(runtime: Runtime, weak: Weak, handle: Handle, pool_size: usize, phase: Phase) {
    // Capture which playlist this fetch is for; a navigation / re-activate that
    // swaps the session before it returns must discard the stale result.
    let (pid, artists, exclude): (u64, Vec<(Option<u64>, String)>, Vec<u64>) = {
        let mut session = SESSION.lock().unwrap();
        match phase {
            Phase::Initial => session.loading = true,
            Phase::Merge => {
                session.loading_more = true;
                if pool_size >= MAX_POOL {
                    session.max_requested = true;
                }
            }
        }
        (
            session.playlist_id,
            session.artists.clone(),
            session.exclude_ids.iter().copied().collect(),
        )
    };

    let runtime2 = runtime.clone();
    let weak2 = weak.clone();
    let handle2 = handle.clone();
    handle.spawn(async move {
        let config = qbz_reco::SuggestionConfig {
            max_pool_size: pool_size,
            ..Default::default()
        };
        let result = runtime2
            .core()
            .generate_playlist_suggestions(artists, exclude, false, Some(config))
            .await;

        match result {
            Ok(result) => {
                let applied = {
                    let mut session = SESSION.lock().unwrap();
                    if session.playlist_id != pid {
                        false // superseded by a navigation / re-activate
                    } else {
                        match phase {
                            Phase::Initial => {
                                session.pool = result.tracks;
                                session.page = 0;
                                session.completed_cycles = 0;
                                session.loading = false;
                            }
                            Phase::Merge => {
                                let existing: HashSet<u64> =
                                    session.pool.iter().map(|t| t.track_id).collect();
                                for track in result.tracks {
                                    if !existing.contains(&track.track_id) {
                                        session.pool.push(track);
                                    }
                                }
                                session.loading_more = false;
                            }
                        }
                        session.loaded_once = true;
                        true
                    }
                };
                if applied {
                    let _ = weak2.upgrade_in_event_loop(|w| project(&w));
                    maybe_auto_expand(runtime2, weak2, handle2);
                }
            }
            Err(e) => {
                let surface = {
                    let mut session = SESSION.lock().unwrap();
                    if session.playlist_id != pid {
                        None // stale — leave the current session untouched
                    } else {
                        match phase {
                            Phase::Initial => {
                                session.loading = false;
                                session.pool.clear();
                                session.loaded_once = true;
                                Some(true)
                            }
                            Phase::Merge => {
                                session.loading_more = false;
                                Some(false)
                            }
                        }
                    }
                };
                let Some(surface) = surface else {
                    return;
                };
                log::warn!("[qbz-slint] playlist suggestions fetch failed: {e}");
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    let state = w.global::<PlaylistSuggestionsState>();
                    state.set_loading(false);
                    state.set_loading_more(false);
                    if surface {
                        state.set_error(e.into());
                        project(&w);
                    }
                });
            }
        }
    });
}
