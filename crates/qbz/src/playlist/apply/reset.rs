//! Clear every per-view static back to its empty/default state ahead of a
//! fresh navigation into a playlist detail.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, PlaylistState, TrackItem};

use super::statics::MIXED;
use crate::playlist::custom_order::CUSTOM_ORDER;
use crate::playlist::view_state::{FULL_ITEMS, QUERY, SORT};

pub fn reset(window: &AppWindow) {
    FULL_ITEMS.with(|cell| cell.borrow_mut().clear());
    QUERY.with(|q| q.borrow_mut().clear());
    SORT.with(|s| *s.borrow_mut() = ("default".to_string(), true));
    CUSTOM_ORDER.with(|c| c.borrow_mut().clear());
    MIXED.store(false, Ordering::Relaxed);
    // Drop the previous detail's queue snapshot — the local/offline/mixed
    // applies repopulate it after this shared reset.
    crate::local_playlist::clear_open_snapshot();
    // Reset the "Suggested Songs" section so a new playlist shows its own
    // "Suggest songs" CTA instead of the previous playlist's rows (T8).
    crate::playlist_suggestions::reset(window);
    let state = window.global::<PlaylistState>();
    state.set_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_track_count(0);
    state.set_total_duration("".into());
    state.set_cover(slint::Image::default());
    state.set_sort_field("default".into());
    state.set_sort_asc(true);
    // Local-playlist flags reset on every navigation; the local detail
    // path re-sets them after this shared reset. The offline-subset flag
    // (D11.a mixed-playlist offline rendering) resets the same way.
    state.set_is_local(false);
    state.set_offline_only(false);
    state.set_offline_subset(false);
    // Ownership / follow / copied flags also reset per navigation; the load
    // path (main.rs) re-derives is-owner / is-following from the playlist owner
    // id vs the current user. Clearing here prevents the previous playlist's
    // state from leaking while the next loads (e.g. a followed playlist briefly
    // showing Delete instead of Unfollow).
    state.set_is_owner(false);
    state.set_is_following(false);
    state.set_is_copied(false);
    state.set_pinned(false);
    state.set_loading(true);
}
