//! Load the tree roots, and expand/collapse a folder node.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, FolderNode, LocalLibraryState};

use super::nodes::{apply_tree, collect_tree, entry_to_node, path_basename};

/// Load the tree roots (registered library folders) on first switch to tree
/// mode. Each root gets a recursive track count so the rail can show totals
/// and gate the expand affordance.
pub fn ensure_folder_tree_loaded(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<LocalLibraryState>();
        if s.get_folder_tree().row_count() != 0 || s.get_folder_tree_loading() {
            return;
        }
        s.set_folder_tree_loading(true);
        let weak2 = w.as_weak();
        handle.spawn(async move {
            let roots = tokio::task::spawn_blocking(|| {
                crate::library_db::with_db(|db| {
                    let paths = db.get_folders()?;
                    let mut out: Vec<(String, u32)> = Vec::with_capacity(paths.len());
                    for p in paths {
                        let cnt = db.count_folder_tracks_recursive(&p, false)?;
                        out.push((p, cnt));
                    }
                    Ok::<_, qbz_library::LibraryError>(out)
                })
                .unwrap_or_default()
            })
            .await
            .unwrap_or_default();
            let _ = weak2.upgrade_in_event_loop(move |w| {
                let nodes: Vec<FolderNode> = roots
                    .into_iter()
                    .map(|(p, cnt)| FolderNode {
                        segment: path_basename(&p).into(),
                        path: p.into(),
                        depth: 0,
                        is_folder: true,
                        expanded: false,
                        can_expand: cnt > 0,
                        track_count: cnt as i32,
                        selected: false,
                        select_state: 0,
                    })
                    .collect();
                let s = w.global::<LocalLibraryState>();
                apply_tree(&s, nodes);
                s.set_folder_tree_loading(false);
            });
        });
    });
}

/// Expand or collapse a folder node. Collapsing is pure UI (drop the
/// contiguous descendant block); expanding fetches one child level lazily.
pub fn toggle_folder_node(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    path: String,
    expand: bool,
) {
    if !expand {
        let _ = weak.upgrade_in_event_loop(move |w| {
            let s = w.global::<LocalLibraryState>();
            let mut nodes = collect_tree(&s);
            if let Some(pos) = nodes.iter().position(|n| n.path == path) {
                let depth = nodes[pos].depth;
                nodes[pos].expanded = false;
                let mut end = pos + 1;
                while end < nodes.len() && nodes[end].depth > depth {
                    end += 1;
                }
                nodes.drain(pos + 1..end);
                apply_tree(&s, nodes);
            }
        });
        return;
    }
    // Expand: fetch this level's children off-thread, then splice them in.
    let path_for_fetch = path.clone();
    handle.spawn(async move {
        let children = tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| db.list_folder_children(&path_for_fetch, false))
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let s = w.global::<LocalLibraryState>();
            let mut nodes = collect_tree(&s);
            if let Some(pos) = nodes.iter().position(|n| n.path == path) {
                let depth = nodes[pos].depth;
                nodes[pos].expanded = true;
                let child_nodes: Vec<FolderNode> = children
                    .iter()
                    .map(|e| entry_to_node(e, depth + 1))
                    .collect();
                for (i, cn) in child_nodes.into_iter().enumerate() {
                    nodes.insert(pos + 1 + i, cn);
                }
                apply_tree(&s, nodes);
            }
        });
    });
}
