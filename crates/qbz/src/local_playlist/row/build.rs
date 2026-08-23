use std::collections::HashMap;

use qbz_models::QueueTrack;

use super::item::row_item;
use super::queue::row_queue_track;
use super::LoadedRow;
use crate::TrackItem;

/// Build the queue snapshot + display rows + id->position map for a resolved
/// row list. Shared by the LOCAL detail [`super::super::detail_local::apply`],
/// the offline MIXED detail (`detail_offline_mixed::apply_qobuz_offline`) and
/// the ONLINE mixed detail (`playlist::apply`) — one row-identity contract
/// for all three (E11).
pub(crate) fn build_row_models(
    rows: &[LoadedRow],
) -> (Vec<QueueTrack>, Vec<TrackItem>, HashMap<String, i32>) {
    let mut queue: Vec<QueueTrack> = Vec::new();
    let mut items: Vec<TrackItem> = Vec::with_capacity(rows.len());
    let mut positions: HashMap<String, i32> = HashMap::new();
    for row in rows {
        let qt = row_queue_track(&row.item);
        let item = row_item(&row.item, qt.as_ref());
        positions.insert(item.id.to_string(), row.position);
        if let Some(qt) = qt {
            queue.push(qt);
        }
        items.push(item);
    }
    (queue, items, positions)
}
