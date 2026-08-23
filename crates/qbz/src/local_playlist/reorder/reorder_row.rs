use std::collections::HashMap;

use qbz_library::local_playlists as repo;
use slint::{ComponentHandle, Model};

use crate::local_playlist::state::{CURRENT_QUEUE, ROW_POSITIONS};
use crate::local_playlist::is_local_id;
use crate::{AppWindow, PlaylistState};

/// Drag-reorder (issue #589): move the VISIBLE row at index `from_row` to
/// insertion slot `to_slot` within the open LOCAL playlist's natural (repo)
/// order — the arbitrary-index sibling of [`super::move_row::move_row`], same
/// contract: optimistic on the UI thread (FULL_ITEMS move + position-map
/// transform + queue-snapshot re-sort), then ONE direct repo write
/// (`repo::reorder`) off-thread. UI thread entry.
pub fn reorder_row(
    window: &AppWindow,
    handle: &tokio::runtime::Handle,
    from_row: usize,
    to_slot: usize,
) {
    let playlist_id = window.global::<PlaylistState>().get_id().to_string();
    if !is_local_id(&playlist_id) {
        return;
    }
    // Slots `from` and `from + 1` drop back onto the same gap (the UI
    // already skips them; keep the guard for safety).
    if to_slot == from_row || to_slot == from_row + 1 {
        return;
    }
    // Resolve the dragged row + the ANCHOR row from the visible model —
    // moving down, the anchor is the row just above the insertion gap (the
    // dragged row lands right after it); moving up, the row at the gap
    // (the dragged row lands right before it).
    let model = window.global::<PlaylistState>().get_tracks();
    let len = model.row_count();
    if from_row >= len || to_slot > len {
        return;
    }
    let moving_down = to_slot > from_row;
    let anchor_row = if moving_down { to_slot - 1 } else { to_slot };
    let Some(clicked_id) = model.row_data(from_row).map(|t| t.id.to_string()) else {
        return;
    };
    let Some(anchor_id) = model.row_data(anchor_row).map(|t| t.id.to_string()) else {
        return;
    };
    // Both rows' indices in the FULL natural order (mirrors move_row: the
    // move works over the full list even while a search filter is active).
    let mut ids = crate::playlist::full_item_ids();
    let (Some(idx), Some(aidx)) = (
        ids.iter().position(|id| *id == clicked_id),
        ids.iter().position(|id| *id == anchor_id),
    ) else {
        return;
    };
    if idx == aidx {
        return;
    }
    // Repo positions of both rows (remove-then-insert semantics: the moved
    // row lands exactly at the anchor's slot, in-between rows shift one —
    // correct even across hidden-row position gaps).
    let (from, to) = {
        let map = ROW_POSITIONS.lock().map(|m| m.clone()).unwrap_or_default();
        match (map.get(&clicked_id), map.get(&anchor_id)) {
            (Some(&f), Some(&t)) => (f, t),
            _ => return,
        }
    };
    if from == to {
        return;
    }

    // Optimistic UI: move the FULL row (re-renders through search/sort).
    // Removing at `idx` then inserting at the anchor's ORIGINAL index puts
    // the dragged row right after the anchor when moving down (the anchor
    // shifted one up) and right before it when moving up (anchor unmoved).
    crate::playlist::move_full_item(window, idx, aidx);
    let moved = ids.remove(idx);
    ids.insert(aidx, moved);

    // Keep the cached position map consistent with what `repo::reorder`
    // writes: the moved row lands at `to`, the in-between rows shift one.
    if let Ok(mut map) = ROW_POSITIONS.lock() {
        for (id, pos) in map.iter_mut() {
            if *id == clicked_id {
                *pos = to;
            } else if from < to && *pos > from && *pos <= to {
                *pos -= 1;
            } else if from > to && *pos >= to && *pos < from {
                *pos += 1;
            }
        }
    }
    // Keep the playable queue snapshot in the rows' new order.
    if let Ok(mut queue) = CURRENT_QUEUE.lock() {
        let order: HashMap<&str, usize> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();
        queue.sort_by_key(|q| {
            let qid = q.id.to_string();
            order.get(qid.as_str()).copied().unwrap_or(usize::MAX)
        });
    }

    let pid = playlist_id.clone();
    handle.spawn(async move {
        tokio::task::spawn_blocking(move || {
            let result = crate::library_db::with_db(|db| {
                Ok(db.with_connection(|conn| repo::reorder(conn, &pid, from, to)))
            });
            if let Some(Err(e)) = result {
                log::error!("[qbz-slint] local playlist drag-reorder {from}->{to}: {e}");
            }
        })
        .await
        .ok();
    });
}
