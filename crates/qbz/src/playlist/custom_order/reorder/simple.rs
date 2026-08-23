//! Seed derivation + the swap/move/apply ops that don't need the visible
//! (Slint-model) row order.

use crate::AppWindow;

use super::super::CUSTOM_ORDER;
use crate::playlist::view_state::{refresh_view, FULL_ITEMS};

/// Custom-order SEED keys on first entry: Qobuz rows in natural order,
/// then LOCAL sidecar rows — Tauri parity (§1.3
/// `initCustomOrderFromCurrentTracks` covers `tracks` + `localTracks`).
/// Offline-copy sidecar rows render source
/// "qobuz" with the REAL catalog id here (Slint's queue-id row identity),
/// so they seed as Qobuz keys — a documented divergence from Tauri's
/// `(local_tracks.id, 1)`; unmapped rows just sort to the end (E6).
pub fn custom_seed_keys() -> Vec<(i64, bool)> {
    FULL_ITEMS.with(|cell| {
        let items = cell.borrow();
        let mut out: Vec<(i64, bool)> = Vec::new();
        for item in items.iter().filter(|t| t.source.as_str() == "qobuz") {
            if let Ok(id) = item.id.parse::<i64>() {
                out.push((id, false));
            }
        }
        for item in items.iter().filter(|t| t.source.as_str() == "local") {
            if let Ok(id) = item.id.parse::<i64>() {
                out.push((id, true));
            }
        }
        out
    })
}

/// The FULL (unfiltered, natural-order) row ids as strings. The LOCAL
/// detail's reorder works over these — rows that don't parse as u64 can't
/// be served by the keyed custom-order helpers. UI thread.
pub fn full_item_ids() -> Vec<String> {
    FULL_ITEMS.with(|cell| cell.borrow().iter().map(|t| t.id.to_string()).collect())
}

/// Swap the FULL_ITEMS entries at natural-order indexes `a` and `b`, then
/// re-render through the active search/sort. The LOCAL detail's optimistic
/// reorder move (B2) — under its "default" sort the visible order IS the
/// FULL order, so the swap shows immediately. UI thread.
pub fn swap_full_items(window: &AppWindow, a: usize, b: usize) {
    FULL_ITEMS.with(|cell| {
        let mut items = cell.borrow_mut();
        if a < items.len() && b < items.len() {
            items.swap(a, b);
        }
    });
    refresh_view(window);
}

/// Remove the FULL row at natural-order index `from` and re-insert it at
/// `insert_at`, then re-render through the active search/sort. The LOCAL
/// detail's optimistic DRAG-reorder move (issue #589) — the arbitrary-index
/// sibling of [`swap_full_items`]. UI thread.
pub fn move_full_item(window: &AppWindow, from: usize, insert_at: usize) {
    FULL_ITEMS.with(|cell| {
        let mut items = cell.borrow_mut();
        if from < items.len() && insert_at < items.len() {
            let item = items.remove(from);
            items.insert(insert_at, item);
        }
    });
    refresh_view(window);
}

/// Store a freshly-loaded custom order + re-render. UI thread.
pub fn apply_custom_order(window: &AppWindow, orders: Vec<((u64, bool), i32)>) {
    CUSTOM_ORDER.with(|c| {
        let mut m = c.borrow_mut();
        m.clear();
        for (key, pos) in orders {
            m.insert(key, pos);
        }
    });
    refresh_view(window);
}
