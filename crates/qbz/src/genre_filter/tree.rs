//! Tree flattening + expand/search state mutation. Pure `State` mutation
//! only, no IO/network.

use crate::GenreTreeRow;

use super::{GenreItem, State, STATE};

pub(super) fn tree_row(item: &GenreItem, level: i32, s: &State) -> GenreTreeRow {
    let loaded = s.children.get(&item.id);
    let count = loaded.map(|c| c.len()).unwrap_or(0);
    // Parents always have children; deeper levels show an expand arrow
    // optimistically until a load proves them empty.
    let has_children = if level == 0 {
        true
    } else if level == 1 {
        count > 0 || loaded.is_none()
    } else {
        false
    };
    GenreTreeRow {
        id: item.id.to_string().into(),
        name: item.name.clone().into(),
        level,
        selected: s.is_selected(item.id),
        expanded: s.expanded.contains(&item.id),
        has_children,
        count: count as i32,
    }
}

/// Flatten the genre tree into the currently-visible rows. With a search
/// query, returns a flat list of all loaded genres matching the query
/// (ignoring expansion); otherwise honors per-node expansion down three
/// levels.
pub(super) fn build_tree_rows(s: &State) -> Vec<GenreTreeRow> {
    let query = s.search.trim().to_lowercase();
    let mut rows: Vec<GenreTreeRow> = Vec::new();

    if !query.is_empty() {
        // Search rows are a flat list that ignores expansion — never show
        // an expand chevron (level 0 would otherwise force has_children on
        // child matches).
        let matches = |g: &GenreItem| g.name.to_lowercase().contains(&query);
        let flat_row = |g: &GenreItem| {
            let mut row = tree_row(g, 0, s);
            row.has_children = false;
            row
        };
        for p in &s.parents {
            if matches(p) {
                rows.push(flat_row(p));
            }
        }
        for kids in s.children.values() {
            for k in kids {
                if matches(k) {
                    rows.push(flat_row(k));
                }
            }
        }
        return rows;
    }

    for parent in &s.parents {
        rows.push(tree_row(parent, 0, s));
        if !s.expanded.contains(&parent.id) {
            continue;
        }
        let Some(children) = s.children.get(&parent.id) else {
            continue;
        };
        for child in children {
            rows.push(tree_row(child, 1, s));
            if !s.expanded.contains(&child.id) {
                continue;
            }
            if let Some(grandchildren) = s.children.get(&child.id) {
                for gc in grandchildren {
                    rows.push(tree_row(gc, 2, s));
                }
            }
        }
    }
    rows
}

/// Toggle a tree node's expanded state. Returns true if it is now expanded
/// (so the caller can lazy-load its children).
pub fn toggle_expand(id_str: &str) -> bool {
    let Ok(id) = id_str.parse::<u64>() else {
        return false;
    };
    let Ok(mut s) = STATE.lock() else {
        return false;
    };
    if s.expanded.contains(&id) {
        s.expanded.remove(&id);
        false
    } else {
        s.expanded.insert(id);
        true
    }
}

pub fn set_search(query: &str) {
    if let Ok(mut s) = STATE.lock() {
        s.search = query.to_string();
    }
}
