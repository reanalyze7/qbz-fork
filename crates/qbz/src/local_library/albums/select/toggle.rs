//! Click/select-all/clear toggles for the Albums multi-select bar.

use slint::ComponentHandle;

use crate::{AppWindow, LocalLibraryState};

use super::basics::{album_is_selected, recount_albums_selected, rendered_album_ids, set_albums_selected};

/// Enter/leave Albums multi-select; leaving clears the selection + anchor.
pub fn set_albums_multi_select(window: &AppWindow, on: bool) {
    window.global::<LocalLibraryState>().set_albums_multi_select(on);
    crate::selection::clear_anchor();
    if !on {
        clear_albums_selection(window);
    }
}

/// Per-card toggle (by id). Plain/Ctrl+Click = single toggle; Shift+Click =
/// additive range from the anchor over the rendered album order.
pub fn toggle_album_select(window: &AppWindow, id: &str) {
    let ids = rendered_album_ids(window);
    let Some(clicked) = ids.iter().position(|a| a == id) else {
        return;
    };
    let shift = crate::keybindings::mods().2;
    if shift {
        if let Some((a_idx, a_id)) =
            crate::selection::anchor_for(crate::selection::SURFACE_LOCAL_ALBUMS)
        {
            // Re-resolve the anchor against the live order (resilient to re-sort).
            let anchor = if ids.get(a_idx).map(|s| *s == a_id).unwrap_or(false) {
                Some(a_idx)
            } else {
                ids.iter().position(|s| *s == a_id)
            };
            if let Some(anchor) = anchor {
                let lo = anchor.min(clicked);
                let hi = anchor.max(clicked);
                let range: std::collections::HashSet<String> =
                    ids[lo..=hi].iter().cloned().collect();
                set_albums_selected(window, &range, true);
                crate::selection::set_anchor(
                    crate::selection::SURFACE_LOCAL_ALBUMS,
                    clicked,
                    id,
                );
                recount_albums_selected(window);
                return;
            }
        }
    }
    let now = !album_is_selected(window, id);
    let one: std::collections::HashSet<String> = std::iter::once(id.to_string()).collect();
    set_albums_selected(window, &one, now);
    crate::selection::set_anchor(crate::selection::SURFACE_LOCAL_ALBUMS, clicked, id);
    recount_albums_selected(window);
}

/// Select-all toggle (the bulk bar button) — select all, or clear if all are
/// already selected. Ctrl+A uses `select_all_albums_only` (never clears).
pub fn select_all_albums(window: &AppWindow) {
    let ids = rendered_album_ids(window);
    let total = ids.len();
    let selected = ids.iter().filter(|id| album_is_selected(window, id)).count();
    let target = selected != total;
    let all: std::collections::HashSet<String> = ids.into_iter().collect();
    set_albums_selected(window, &all, target);
    recount_albums_selected(window);
}

/// Select every rendered album (Ctrl+A — never clears).
pub fn select_all_albums_only(window: &AppWindow) {
    let all: std::collections::HashSet<String> =
        rendered_album_ids(window).into_iter().collect();
    set_albums_selected(window, &all, true);
    recount_albums_selected(window);
}

/// Clear the album selection (multi-select mode stays on).
pub fn clear_albums_selection(window: &AppWindow) {
    let all: std::collections::HashSet<String> =
        rendered_album_ids(window).into_iter().collect();
    set_albums_selected(window, &all, false);
    window
        .global::<LocalLibraryState>()
        .set_albums_selected_count(0);
}
