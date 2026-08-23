//! Building a queue (+ start index) from a view's VISIBLE row model.

use super::model_helpers::track_item_to_queue;
use crate::TrackItem;
use qbz_models::{QueueTrack, Track};
use slint::{Model, ModelRc};

/// Build a play queue (+ start index) from a view's VISIBLE `TrackItem`
/// model, starting at `clicked_id`. The model IS the visible order, so this
/// never goes out of sync with what the user sees. Used by views with no
/// full-`Track` cache (search).
pub(super) fn queue_from_model(
    model: &ModelRc<TrackItem>,
    clicked_id: &str,
) -> (Vec<QueueTrack>, Option<usize>) {
    let mut queue: Vec<QueueTrack> = Vec::with_capacity(model.row_count());
    let mut found: Option<usize> = None;
    for i in 0..model.row_count() {
        if let Some(it) = model.row_data(i) {
            if let Some(qt) = track_item_to_queue(&it) {
                if it.id.as_str() == clicked_id {
                    found = Some(queue.len());
                }
                queue.push(qt);
            }
        }
    }
    // `found` is None when the clicked track is not a list row (e.g. the
    // search "most popular" hero card) — the caller decides what to do.
    (queue, found)
}

/// Build a play queue (+ start index) from a view's VISIBLE `TrackItem`
/// model and its authoritative `Vec<Track>` cache: the queue follows the
/// visible order (so custom sort / search filter are respected) and starts
/// at `clicked_id`. Falls back to the cache order if the visible/cache
/// mapping comes up empty.
pub(super) fn order_by_visible(
    model: &ModelRc<TrackItem>,
    cache: Vec<Track>,
    clicked_id: &str,
) -> Option<(Vec<Track>, usize)> {
    let visible_ids: Vec<String> = (0..model.row_count())
        .filter_map(|i| model.row_data(i).map(|it| it.id.to_string()))
        .collect();
    let by_id: std::collections::HashMap<String, Track> =
        cache.iter().map(|t| (t.id.to_string(), t.clone())).collect();
    let ordered: Vec<Track> = visible_ids.iter().filter_map(|id| by_id.get(id).cloned()).collect();
    // The clicked track must resolve inside the visible list; if it does not
    // (orphan/hero row, or a cache miss), return None so the caller plays just
    // that track rather than starting the queue at the wrong row.
    let idx = ordered.iter().position(|t| t.id.to_string() == clicked_id)?;
    Some((ordered, idx))
}
