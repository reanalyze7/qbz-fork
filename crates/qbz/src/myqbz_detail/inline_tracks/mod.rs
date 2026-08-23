//! Expanded-mode inline-track fetch + cache (spec 12 §8).

mod track_map;

use qbz_models::mixtape::MixtapeCollectionItem;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use track_map::track_to_item;

use super::strings::inline_cache_key;
use super::{FULL_ITEMS, INLINE_CACHE};
use crate::{AppWindow, MyQbzDetailState, TrackItem};

/// The full `MixtapeCollectionItem` for one `source_item_id` (the row's stable
/// key). Sourced from `FULL_ITEMS` so the resolver gets the numeric
/// year/track_count + the typed item_type/source. UI thread.
pub(super) fn full_item_by_source_id(source_item_id: &str) -> Option<MixtapeCollectionItem> {
    FULL_ITEMS.with(|cell| {
        cell.borrow()
            .iter()
            .find(|it| it.source_item_id == source_item_id)
            .cloned()
    })
}

/// Find the rendered row for `source_item_id` and mutate it in place. UI thread.
pub(super) fn with_row_by_source_id<F: FnOnce(&mut crate::MixtapeDetailItem)>(
    window: &AppWindow,
    source_item_id: &str,
    f: F,
) {
    let model = window.global::<MyQbzDetailState>().get_items();
    for i in 0..model.row_count() {
        if let Some(mut it) = model.row_data(i) {
            if it.source_item_id == source_item_id {
                f(&mut it);
                model.set_row_data(i, it);
                break;
            }
        }
    }
}

/// Ensure every expandable item's inline tracks are loaded (spec 12 §8). Fired
/// when the "expanded" view-mode becomes active. For each rendered row that
/// `can_expand` and is not already loaded / loading, flips `expand-loading` on
/// and spawns a per-item fetch via the shared enqueue resolver
/// (`myqbz_play::fetch_item_tracks`); on completion it populates that row's
/// inline-tracks model + marks it loaded. Idempotent: already-cached rows are
/// skipped, so re-entering expanded mode is instant (and re-deriving the model
/// after a filter/sort resets `tracks_loaded`, so the new rows re-fetch).
pub fn ensure_expanded(
    runtime: std::sync::Arc<qbz_app::shell::AppRuntime<crate::adapter::SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    let Some(window) = weak.upgrade() else { return };
    let model = window.global::<MyQbzDetailState>().get_items();

    // Snapshot the rows that still need a fetch (source + source-item-id), then
    // mark them loading in one pass (mutating the model while iterating is fine
    // — we set_row_data the same index we read). `tracks_loaded` rows are
    // skipped: the cache already re-hydrated them in `to_item`, so a re-derive
    // is instant (no re-fetch).
    let mut pending: Vec<(String, String)> = Vec::new();
    for i in 0..model.row_count() {
        if let Some(mut it) = model.row_data(i) {
            if it.can_expand && !it.tracks_loaded && !it.expand_loading {
                it.expand_loading = true;
                let source = it.source.to_string();
                let id = it.source_item_id.to_string();
                model.set_row_data(i, it);
                pending.push((source, id));
            }
        }
    }

    for (source, source_item_id) in pending {
        let Some(full_item) = full_item_by_source_id(&source_item_id) else {
            // No backing item (shouldn't happen) — clear the spinner.
            with_row_by_source_id(&window, &source_item_id, |it| it.expand_loading = false);
            continue;
        };
        let runtime = runtime.clone();
        let weak = weak.clone();
        handle.spawn(async move {
            // `Vec<QueueTrack>` is `Send`; the mapped `Vec<TrackItem>` carries
            // a `slint::Image` (!Send), so it must be built INSIDE the event
            // loop, not moved across the thread boundary.
            let tracks = crate::myqbz_play::fetch_item_tracks(&runtime, &full_item).await;
            let _ = weak.upgrade_in_event_loop(move |w| {
                let items: Vec<TrackItem> = tracks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| track_to_item(t, i))
                    .collect();
                // Persist into the controller-level cache (keyed
                // `source|source_item_id`) so a later filter/sort/search
                // re-derive re-hydrates this row from the cache instead of
                // re-fetching (spec 12 §8 — cache survives the re-derive).
                let cache_key = inline_cache_key(&source, &source_item_id);
                INLINE_CACHE.with(|cell| {
                    cell.borrow_mut().insert(cache_key, items.clone());
                });
                with_row_by_source_id(&w, &source_item_id, |it| {
                    it.expand_loading = false;
                    it.tracks_loaded = true;
                    it.inline_tracks = ModelRc::new(VecModel::from(items));
                });
            });
        });
    }
}
