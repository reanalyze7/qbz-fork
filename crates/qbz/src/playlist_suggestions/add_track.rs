//! `add_track` and `play_track` — add a suggestion to the playlist, or
//! preview it now. Plus the shared `set_row_flag` row-mutation helper.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, PlaylistSuggestionsState};

use super::adaptive_artists::make_key;
use super::auto_expand::maybe_auto_expand;
use super::reload::reload_open_playlist;
use super::filter_project::project;
use super::session::SESSION;
use super::{Handle, Runtime, Weak};

/// Add a suggested track to the open playlist, drop it from the pool, and
/// reload the detail so the new track appears in the list. UI thread.
pub fn add_track(window: &AppWindow, runtime: Runtime, handle: Handle, track_id: String) {
    let Ok(tid) = track_id.parse::<u64>() else {
        return;
    };
    let playlist_id = {
        let session = SESSION.lock().unwrap();
        session.playlist_id
    };
    if playlist_id == 0 {
        return;
    }

    // Optimistic "adding" flag on the visible row.
    set_row_flag(window, &track_id, true, false);

    let weak = window.as_weak();
    let runtime2 = runtime.clone();
    let handle2 = handle.clone();
    handle.spawn(async move {
        match runtime2
            .core()
            .add_tracks_to_playlist(playlist_id, &[tid])
            .await
        {
            Ok(()) => {
                {
                    let mut session = SESSION.lock().unwrap();
                    session.exclude_ids.insert(tid);
                    if let Some(track) = session.pool.iter().find(|t| t.track_id == tid) {
                        let key = make_key(&track.title, &track.artist_name);
                        session.existing_keys.insert(key);
                    }
                    session.pool.retain(|t| t.track_id != tid);
                }
                let runtime3 = runtime2.clone();
                let weak3 = weak.clone();
                let handle3 = handle2.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    project(&w);
                    reload_open_playlist(&w, runtime3, handle3, playlist_id);
                    maybe_auto_expand(runtime2, weak3, handle2);
                });
            }
            Err(e) => {
                log::warn!("[qbz-slint] add suggested track {tid} failed: {e}");
                let _ = weak.upgrade_in_event_loop(move |w| {
                    set_row_flag(&w, &track_id, false, false);
                });
            }
        }
    });
}

/// Preview / play a single suggested track now. UI thread.
pub fn play_track(runtime: Runtime, weak: Weak, handle: Handle, track_id: String) {
    let Ok(tid) = track_id.parse::<u64>() else {
        return;
    };
    let handle2 = handle.clone();
    handle.spawn(async move {
        match runtime.core().get_track(tid).await {
            Ok(track) => {
                crate::playback::play_tracks(runtime, weak, handle2, vec![track], 0);
            }
            Err(e) => log::warn!("[qbz-slint] preview suggested track {tid} failed: {e}"),
        }
    });
}

/// Flip the `adding` flag on the visible row that matches `track_id` (so the
/// per-row add button can show its in-flight state).
pub(super) fn set_row_flag(window: &AppWindow, track_id: &str, adding: bool, added: bool) {
    let model = window.global::<PlaylistSuggestionsState>().get_rows();
    for i in 0..model.row_count() {
        if let Some(mut row) = model.row_data(i) {
            if row.track_id.as_str() == track_id {
                row.adding = adding;
                row.added = added;
                model.set_row_data(i, row);
                break;
            }
        }
    }
}
