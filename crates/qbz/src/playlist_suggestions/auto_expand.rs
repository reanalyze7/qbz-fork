//! Pool-exhaustion auto-refresh: grow the pool when the filtered pool runs low.

use slint::ComponentHandle;

use crate::PlaylistSuggestionsState;

use super::fetch::spawn_fetch;
use super::filter_project::filtered_indices;
use super::session::{Phase, SESSION};
use super::{Handle, Runtime, Weak, MAX_POOL, MIN_AVAILABLE_THRESHOLD};

/// Grow the pool to MAX_POOL when the filtered (available) tracks fall below
/// the threshold (Svelte's pool-exhaustion auto-refresh). One-shot per session
/// (guarded by `max_requested`) so a thin engine result never loops.
pub(super) fn maybe_auto_expand(runtime: Runtime, weak: Weak, handle: Handle) {
    let should = {
        let session = SESSION.lock().unwrap();
        let available = filtered_indices(&session).len();
        available > 0
            && available < MIN_AVAILABLE_THRESHOLD
            && session.loaded_once
            && !session.loading
            && !session.loading_more
            && !session.max_requested
            && session.pool.len() < MAX_POOL
    };
    if should {
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<PlaylistSuggestionsState>().set_loading_more(true);
        });
        spawn_fetch(runtime, weak, handle, MAX_POOL, Phase::Merge);
    }
}
