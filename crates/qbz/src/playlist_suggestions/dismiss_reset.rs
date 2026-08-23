//! `dismiss_track` and `reset`.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, PlaylistSuggestionRow, PlaylistSuggestionsState};

use super::auto_expand::maybe_auto_expand;
use super::filter_project::project;
use super::session::{Session, SESSION};
use super::{Handle, Runtime};

/// Dismiss a suggestion (sticky per-playlist via the T10 store) and drop it from
/// the pool. UI thread.
pub fn dismiss_track(window: &AppWindow, runtime: Runtime, handle: Handle, track_id: String) {
    let Ok(tid) = track_id.parse::<u64>() else {
        return;
    };
    {
        let mut session = SESSION.lock().unwrap();
        if session.playlist_id == 0 {
            return;
        }
        crate::playlist_suggestions_dismiss::dismiss(session.playlist_id, tid);
        session.pool.retain(|t| t.track_id != tid);
    }
    project(window);
    maybe_auto_expand(runtime, window.as_weak(), handle);
}

/// Reset the section to its pre-activation state. Called from
/// `crate::playlist::reset` on every playlist navigation so a new playlist
/// shows its own "Suggest songs" CTA instead of stale rows. UI thread.
pub fn reset(window: &AppWindow) {
    *SESSION.lock().unwrap() = Session::default();
    let state = window.global::<PlaylistSuggestionsState>();
    state.set_activated(false);
    state.set_loading(false);
    state.set_loading_more(false);
    state.set_has_more(false);
    state.set_is_empty(false);
    state.set_error("".into());
    state.set_rows(ModelRc::new(VecModel::from(Vec::<PlaylistSuggestionRow>::new())));
}
