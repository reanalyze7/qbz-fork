//! Keyboard-move (`move_track`) and drag-reorder (`reorder_track`) — both
//! rebuild the whole custom order with clean 0..N-1 positions.

use crate::AppWindow;

use super::super::CUSTOM_ORDER;
use crate::playlist::view_state::{custom_key, refresh_view, FULL_ITEMS};

/// Move a track one slot up/down in the custom order. Rebuilds the
/// whole order with clean 0..N-1 positions (self-healing), re-renders,
/// and returns the new `(id, is_local, position)` rows to persist.
/// Rows with no stable key can't participate and stay at the end. UI thread.
pub fn move_track(window: &AppWindow, track_id: &str, up: bool) -> Vec<(u64, bool, i32)> {
    let target = FULL_ITEMS.with(|cell| {
        cell.borrow()
            .iter()
            .find(|t| t.id.as_str() == track_id)
            .and_then(custom_key)
    });
    let Some(target) = target else {
        return Vec::new();
    };
    // Current visible custom order of keyed rows.
    let order = CUSTOM_ORDER.with(|c| c.borrow().clone());
    let mut keys: Vec<(u64, bool)> =
        FULL_ITEMS.with(|cell| cell.borrow().iter().filter_map(custom_key).collect());
    keys.sort_by_key(|key| order.get(key).copied().unwrap_or(i32::MAX));
    let Some(idx) = keys.iter().position(|&key| key == target) else {
        return Vec::new();
    };
    let swap = if up {
        if idx == 0 {
            return Vec::new();
        }
        idx - 1
    } else {
        if idx + 1 >= keys.len() {
            return Vec::new();
        }
        idx + 1
    };
    keys.swap(idx, swap);
    // Rebuild contiguous positions.
    let orders: Vec<(u64, bool, i32)> = keys
        .iter()
        .enumerate()
        .map(|(i, &(id, is_local))| (id, is_local, i as i32))
        .collect();
    CUSTOM_ORDER.with(|c| {
        let mut m = c.borrow_mut();
        m.clear();
        for &(id, is_local, pos) in &orders {
            m.insert((id, is_local), pos);
        }
    });
    refresh_view(window);
    orders
}

/// Drag-reorder (issue #589): move the row at VISIBLE index `from` to
/// insertion slot `to` (0..=N, visible-list slots) in the custom order —
/// the arbitrary-index sibling of [`move_track`]. Same rebuild contract:
/// clean 0..N-1 positions over the keyed rows, optimistic re-render, and
/// the new `(id, is_local, position)` rows returned for persisting.
/// Un-keyed rows (no stable numeric id) can't participate: dragging one,
/// or dropping against one, is a no-op. UI thread.
pub fn reorder_track(window: &AppWindow, from: usize, to: usize) -> Vec<(u64, bool, i32)> {
    use crate::PlaylistState;
    use slint::{ComponentHandle, Model};
    // Slots `from` and `from + 1` drop back onto the same gap (the UI
    // already skips them; keep the guard for safety).
    if to == from || to == from + 1 {
        return Vec::new();
    }
    let model = window.global::<PlaylistState>().get_tracks();
    let len = model.row_count();
    if from >= len || to > len {
        return Vec::new();
    }
    let Some(target) = model.row_data(from).as_ref().and_then(custom_key) else {
        return Vec::new();
    };
    // Anchor: the visible row the dragged row lands AGAINST — moving down,
    // the row just above the insertion gap (dragged goes after it); moving
    // up, the row at the gap (dragged goes before it).
    let moving_down = to > from;
    let anchor_visible = if moving_down { to - 1 } else { to };
    let Some(anchor) = model.row_data(anchor_visible).as_ref().and_then(custom_key) else {
        return Vec::new();
    };
    if anchor == target {
        return Vec::new();
    }
    // Current keyed custom order (same derivation as move_track).
    let order = CUSTOM_ORDER.with(|c| c.borrow().clone());
    let mut keys: Vec<(u64, bool)> =
        FULL_ITEMS.with(|cell| cell.borrow().iter().filter_map(custom_key).collect());
    keys.sort_by_key(|key| order.get(key).copied().unwrap_or(i32::MAX));
    let Some(idx) = keys.iter().position(|&key| key == target) else {
        return Vec::new();
    };
    let moved = keys.remove(idx);
    let insert_at = match keys.iter().position(|&key| key == anchor) {
        Some(a) => {
            if moving_down {
                a + 1
            } else {
                a
            }
        }
        None => keys.len(),
    };
    keys.insert(insert_at.min(keys.len()), moved);
    // Rebuild contiguous positions.
    let orders: Vec<(u64, bool, i32)> = keys
        .iter()
        .enumerate()
        .map(|(i, &(id, is_local))| (id, is_local, i as i32))
        .collect();
    CUSTOM_ORDER.with(|c| {
        let mut m = c.borrow_mut();
        m.clear();
        for &(id, is_local, pos) in &orders {
            m.insert((id, is_local), pos);
        }
    });
    refresh_view(window);
    orders
}
