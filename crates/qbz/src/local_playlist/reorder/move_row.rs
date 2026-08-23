use std::collections::HashMap;

use qbz_library::local_playlists as repo;
use slint::ComponentHandle;

use crate::local_playlist::state::{CURRENT_QUEUE, ROW_POSITIONS};
use crate::local_playlist::is_local_id;
use crate::{AppWindow, PlaylistState};

/// Move the clicked row one slot up/down within the open LOCAL playlist's
/// natural (repo) order — the local branch of the detail view's reorder
/// chevrons (`("track","move-up"/"move-down")`). Optimistic on the UI
/// thread (FULL_ITEMS swap + position-map transform + queue-snapshot
/// reorder), then the direct repo write (`repo::reorder`) off-thread — no
/// custom-order sidecar, the repo `position` IS the order. UI thread entry.
pub fn move_row(
    window: &AppWindow,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
    up: bool,
) {
    let playlist_id = window.global::<PlaylistState>().get_id().to_string();
    if !is_local_id(&playlist_id) {
        return;
    }
    // Neighbor in the FULL natural order (mirrors `playlist::move_track`,
    // which also moves within the full list while a search is active).
    let mut ids = crate::playlist::full_item_ids();
    let Some(idx) = ids.iter().position(|id| id == clicked_id) else {
        return;
    };
    let neighbor = if up {
        idx.checked_sub(1)
    } else {
        (idx + 1 < ids.len()).then_some(idx + 1)
    };
    let Some(nidx) = neighbor else {
        return; // already first / last
    };
    // Repo positions of both rows. Hidden (unresolvable) rows keep their
    // own positions in the DB; remove-then-insert semantics make the move
    // land exactly at the neighbor's slot even across those gaps.
    let (from, to) = {
        let map = ROW_POSITIONS.lock().map(|m| m.clone()).unwrap_or_default();
        match (map.get(clicked_id), map.get(&ids[nidx])) {
            (Some(&f), Some(&t)) => (f, t),
            _ => return,
        }
    };
    if from == to {
        return;
    }

    // Optimistic UI: swap the FULL rows (re-renders through search/sort).
    crate::playlist::swap_full_items(window, idx, nidx);
    ids.swap(idx, nidx);

    // Keep the cached position map consistent with what `repo::reorder`
    // writes: the moved row lands at `to`, the in-between rows shift one.
    if let Ok(mut map) = ROW_POSITIONS.lock() {
        for (id, pos) in map.iter_mut() {
            if id == clicked_id {
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
                log::error!("[qbz-slint] local playlist reorder {from}->{to}: {e}");
            }
        })
        .await
        .ok();
    });
}
