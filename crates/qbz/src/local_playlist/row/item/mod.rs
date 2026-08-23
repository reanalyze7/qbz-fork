mod cached;
mod fallback;
mod local;

use qbz_models::QueueTrack;

use super::RowItem;
use crate::TrackItem;

/// Build the display row for a resolved item. `queue` (when playable)
/// dictates the row id so visible-order playback maps 1:1.
pub(crate) fn row_item(item: &RowItem, queue: Option<&QueueTrack>) -> TrackItem {
    match item {
        RowItem::Qobuz(track) => crate::playlist::to_item(track),
        RowItem::Cached {
            track_id,
            title,
            artist,
            album,
            duration_secs,
            bit_depth,
            sample_rate,
            artwork_path,
        } => cached::cached_item(
            *track_id,
            title,
            artist,
            album,
            *duration_secs,
            *bit_depth,
            *sample_rate,
            artwork_path,
        ),
        RowItem::Local(track) => local::local_item(track, queue),
        RowItem::LocalFile { path } => fallback::local_file_item(path),
        RowItem::Unresolved { kind, reference } => fallback::unresolved_item(kind, reference),
    }
}
