//! Flattened tree-node mapping + the single apply sink that recomputes
//! selection state and commits the model.

use slint::{Model, ModelRc, VecModel};

use crate::{FolderNode, LocalLibraryState};

use super::select::tree_selected;

/// Last path component for display (the registered root paths are absolute).
pub(crate) fn path_basename(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

/// Map a backend tree entry to a flattened `FolderNode` at the given depth.
pub(crate) fn entry_to_node(entry: &qbz_library::FolderTreeEntry, depth: i32) -> FolderNode {
    match entry {
        qbz_library::FolderTreeEntry::Folder {
            path,
            segment,
            track_count_under,
            ..
        } => FolderNode {
            path: path.clone().into(),
            segment: segment.clone().into(),
            depth,
            is_folder: true,
            expanded: false,
            can_expand: *track_count_under > 0,
            track_count: *track_count_under as i32,
            selected: false,
            select_state: 0,
        },
        qbz_library::FolderTreeEntry::Track { path, segment } => FolderNode {
            path: path.clone().into(),
            segment: segment.clone().into(),
            depth,
            is_folder: false,
            expanded: false,
            can_expand: false,
            track_count: 0,
            selected: false,
            select_state: 0,
        },
    }
}

/// Collect the current flattened tree into a plain vec for splicing.
pub(crate) fn collect_tree(s: &LocalLibraryState) -> Vec<FolderNode> {
    let m = s.get_folder_tree();
    (0..m.row_count()).filter_map(|i| m.row_data(i)).collect()
}

/// Apply the current selection to `nodes` (track `selected`, folder tri-state),
/// commit the tree model, refresh the selected count, and derive the visible
/// (search-filtered) set. The single sink for every tree mutation.
pub(crate) fn apply_tree(s: &LocalLibraryState, mut nodes: Vec<FolderNode>) {
    {
        let sel = tree_selected();
        for n in nodes.iter_mut() {
            if n.is_folder {
                let prefix = format!("{}/", n.path);
                let under = sel.keys().filter(|p| p.starts_with(&prefix)).count();
                n.select_state = if under == 0 {
                    0
                } else if n.track_count > 0 && under as i32 >= n.track_count {
                    2
                } else {
                    1
                };
                n.selected = false;
            } else {
                n.selected = sel.contains_key(n.path.as_str());
                n.select_state = 0;
            }
        }
        s.set_tree_selected_count(sel.len() as i32);
    }
    s.set_folder_tree(ModelRc::new(VecModel::from(nodes)));
    super::search::derive_folder_tree_visible(s);
}
