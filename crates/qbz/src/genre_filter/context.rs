//! The per-context read/query surface.

use std::collections::{HashMap, HashSet};

use super::{GenreItem, STATE};

/// Set the context the popup edits (call when opening it for a surface).
pub fn set_context(ctx: &str) {
    if let Ok(mut s) = STATE.lock() {
        s.current = ctx.to_string();
        s.selected.entry(ctx.to_string()).or_default();
    }
}

/// The context the popup is currently editing.
pub fn current_context() -> String {
    STATE
        .lock()
        .map(|s| s.current.clone())
        .unwrap_or_else(|_| "discover".to_string())
}

/// The explicitly-selected genre ids in the current popup context.
pub fn selected_ids() -> Vec<u64> {
    STATE
        .lock()
        .map(|s| s.selected.get(&s.current).cloned().unwrap_or_default())
        .unwrap_or_default()
}

/// The RAW genre selection for `ctx` (no expansion, no ancestor mapping):
/// the exact ids the user toggled, parent or sub-genre. This is what gets
/// sent to /discover/* in `genre_ids` — Qobuz honors sub-genre ids
/// server-side (1:1 with Tauri discovery-v2, which sent the raw selection
/// straight through and did no client-side narrowing).
pub fn selected_ids_for(ctx: &str) -> Vec<u64> {
    STATE
        .lock()
        .map(|s| s.selected.get(ctx).cloned().unwrap_or_default())
        .unwrap_or_default()
}

/// Selected genre NAMES (+ descendant names) for `ctx` — for the
/// client-side album / track genre filter used by favorites.
///
/// Depends on `s.children`, which `loaders.rs`'s `store_children` populates —
/// a runtime ordering, not a compile-time coupling.
pub fn selected_names(ctx: &str) -> Vec<String> {
    let Ok(s) = STATE.lock() else {
        return Vec::new();
    };
    let mut ids: HashSet<u64> = HashSet::new();
    if let Some(sel) = s.selected.get(ctx) {
        for id in sel {
            ids.insert(*id);
            collect_descendants(&s.children, *id, &mut ids);
        }
    }
    let mut names: Vec<String> = Vec::new();
    for id in ids {
        if let Some(g) = s.parents.iter().find(|g| g.id == id) {
            names.push(g.name.clone());
        } else if let Some(g) = s.children.values().flatten().find(|g| g.id == id) {
            names.push(g.name.clone());
        }
    }
    names
}

pub(super) fn collect_descendants(
    children: &HashMap<u64, Vec<GenreItem>>,
    id: u64,
    out: &mut HashSet<u64>,
) {
    if let Some(kids) = children.get(&id) {
        for kid in kids {
            if out.insert(kid.id) {
                collect_descendants(children, kid.id, out);
            }
        }
    }
}
