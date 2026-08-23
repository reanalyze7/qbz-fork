//! Building a `QueueItem` from plain row data while reusing prior decoded
//! artwork handles (the core of the CPU-spike fix).

use crate::queue::row::RowData;
use crate::QueueItem;

/// Build a `QueueItem` from plain row data, REUSING a prior decoded artwork
/// handle when the same track id was already on screen. This is the core of the
/// CPU-spike fix: a one-position queue shift keeps the decoded `slint::Image`
/// for every unchanged row instead of resetting it to `Image::default()` and
/// forcing a full re-decode (which also caused the empty-then-fill blink).
///
/// `prior` maps track id -> the decoded image from the model being replaced.
/// Unchanged rows reuse their handle; only genuinely-new rows fall back to the
/// default placeholder (their cover is decoded once by the artwork pipeline).
pub(in crate::queue) fn to_item_reuse(
    row: &RowData,
    prior: &std::collections::HashMap<slint::SharedString, slint::Image>,
) -> QueueItem {
    let id: slint::SharedString = row.id.clone().into();
    let artwork = prior.get(&id).cloned().unwrap_or_default();
    QueueItem {
        id: id.clone(),
        title: row.title.clone().into(),
        artist: row.artist.clone().into(),
        artwork,
        playing: row.playing,
        duration: row.duration.clone().into(),
        explicit: row.explicit,
        is_ephemeral: row.is_ephemeral,
    }
}
