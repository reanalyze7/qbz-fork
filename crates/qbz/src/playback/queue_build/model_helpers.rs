//! Pure helpers for building/reordering a queue from a Slint `TrackItem`
//! model (the views with no full-`Track` cache — search).

use crate::TrackItem;
use qbz_models::QueueTrack;
use slint::{Model, ModelRc};

/// "m:ss" / "h:mm:ss" -> seconds (for a queue row built off a display string).
pub(super) fn mmss_to_secs(s: &str) -> u64 {
    s.split(':')
        .filter_map(|p| p.trim().parse::<u64>().ok())
        .fold(0u64, |acc, v| acc * 60 + v)
}

/// Build a `QueueTrack` from a visible Slint `TrackItem` row. Used for views
/// that render Qobuz tracks but keep no full-`Track` cache (search): the
/// audio is resolved by id at play time, so the row's display fields suffice
/// to seed the queue. Returns None for rows whose id is not numeric.
pub(super) fn track_item_to_queue(it: &TrackItem) -> Option<QueueTrack> {
    let id = it.id.as_str().parse::<u64>().ok()?;
    let album_id = {
        let a = it.album_id.to_string();
        if a.is_empty() {
            None
        } else {
            Some(a)
        }
    };
    Some(QueueTrack {
        id,
        title: it.title.to_string(),
        version: None,
        artist: it.artist.to_string(),
        album: it.album.to_string(),
        album_version: None,
        duration_secs: mmss_to_secs(it.duration.as_str()),
        artwork_url: {
            let u = it.artwork_url.to_string();
            if u.is_empty() {
                None
            } else {
                Some(u)
            }
        },
        hires: it.quality_tier.as_str() == "hires",
        bit_depth: None,
        sample_rate: None,
        is_local: it.source.as_str() == "local",
        album_id: album_id.clone(),
        artist_id: it.artist_id.as_str().parse::<u64>().ok(),
        streamable: true,
        source: {
            let s = it.source.to_string();
            Some(if s.is_empty() { "qobuz".to_string() } else { s })
        },
        parental_warning: it.explicit,
        source_item_id_hint: album_id,
        // Stamped by the play path when launched from a container; unset here.
        context_kind: None,
        context_id: None,
    })
}

/// The ids of a view's VISIBLE `TrackItem` model rows, in order.
pub(super) fn model_ids(model: &ModelRc<TrackItem>) -> Vec<String> {
    (0..model.row_count())
        .filter_map(|i| model.row_data(i).map(|it| it.id.to_string()))
        .collect()
}

/// Re-order (and filter) a freshly-built queue to match a view's VISIBLE
/// order: keep only the tracks the user can see, in the order they see them.
/// Used by the re-fetch views (album / artist top tracks) so an active
/// in-page search filter is respected. Empty `visible_ids` (or no overlap)
/// leaves the canonical order untouched.
pub(in super::super) fn reorder_queue_by_visible(
    queue: Vec<QueueTrack>,
    visible_ids: &[String],
) -> Vec<QueueTrack> {
    if visible_ids.is_empty() {
        return queue;
    }
    let pos: std::collections::HashMap<String, usize> =
        queue.iter().enumerate().map(|(i, q)| (q.id.to_string(), i)).collect();
    let order: Vec<usize> = visible_ids.iter().filter_map(|id| pos.get(id).copied()).collect();
    if order.is_empty() {
        return queue;
    }
    let mut slots: Vec<Option<QueueTrack>> = queue.into_iter().map(Some).collect();
    order.iter().filter_map(|&i| slots[i].take()).collect()
}
