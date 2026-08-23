//! Modal open/close for the DJ-mix sampler.

use slint::ComponentHandle;

use super::options::apply_options;
use super::Runtime;
use crate::{AppWindow, MyQbzMixState};

/// Open the DJ-mix modal for the collection currently shown in the detail view
/// (`collection_id`). Shows the modal in a "computing…" state, then resolves the
/// collection in-order on a worker + counts unique tracks (deterministic) and
/// fills the slider. On a resolve failure the modal closes with an error toast.
pub fn open(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    collection_id: String,
) {
    if collection_id.is_empty() {
        return;
    }
    // Show the modal immediately in its loading state.
    {
        let weak = weak.clone();
        let _ = weak.upgrade_in_event_loop(|w| {
            let state = w.global::<MyQbzMixState>();
            state.set_loading(true);
            state.set_busy(false);
            state.set_unique_count(0);
            state.set_size_options(slint::ModelRc::new(slint::VecModel::from(Vec::<i32>::new())));
            state.set_selected_index(0);
            state.set_selected_size(0);
            state.set_selected_is_all(false);
            state.set_open(true);
        });
    }

    handle.spawn(async move {
        let Some(collection) = crate::myqbz_play::load_collection(&collection_id).await else {
            close_with_error(&weak, qbz_i18n::t("Couldn't load this collection"));
            return;
        };
        // Always InOrder for DJ-mix (force_shuffle = false): the sampler does its
        // own randomization; the resolve only needs the full track pool.
        let tracks = crate::myqbz_play::resolve_collection(&runtime, &collection, false).await;
        // Deterministic count (no RNG) — the slider max + "All" size.
        let unique = qbz_mixtape::shuffle::unique_track_count(&tracks) as i32;
        let _ = weak.upgrade_in_event_loop(move |w| {
            if unique <= 0 {
                // Nothing playable — close with a hint (mirrors the resolve-empty
                // toast on the play paths).
                close(&w);
                crate::toast::error(&w, qbz_i18n::t("This collection resolved to 0 playable tracks"));
            } else {
                apply_options(&w, unique);
            }
        });
    });
}

/// Close the modal (UI thread hop, callable from any thread).
pub(super) fn close_with_error(weak: &slint::Weak<AppWindow>, msg: String) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        close(&w);
        crate::toast::error(&w, msg);
    });
}

/// Close the modal + clear its busy flag. UI thread.
pub fn close(window: &AppWindow) {
    let state = window.global::<MyQbzMixState>();
    state.set_open(false);
    state.set_busy(false);
    state.set_loading(false);
}
