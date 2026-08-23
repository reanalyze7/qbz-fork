//! Tree rail search filter.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, FolderNode, LocalLibraryState};

/// Derive `folder-tree-visible` from `folder-tree`, filtered by the rail search
/// (keeps matching nodes AND their ancestors so the tree stays navigable).
pub(crate) fn derive_folder_tree_visible(s: &LocalLibraryState) {
    let full = s.get_folder_tree();
    let nodes: Vec<FolderNode> = (0..full.row_count()).filter_map(|i| full.row_data(i)).collect();
    let q = s.get_folders_tree_search().as_str().trim().to_lowercase();
    if q.is_empty() {
        s.set_folder_tree_visible(ModelRc::new(VecModel::from(nodes)));
        return;
    }
    let matches: Vec<String> = nodes
        .iter()
        .filter(|n| n.segment.as_str().to_lowercase().contains(&q))
        .map(|n| n.path.to_string())
        .collect();
    let kept: Vec<FolderNode> = nodes
        .into_iter()
        .filter(|n| {
            let p = n.path.as_str();
            n.segment.as_str().to_lowercase().contains(&q)
                || matches.iter().any(|m| m.starts_with(&format!("{p}/")))
        })
        .collect();
    s.set_folder_tree_visible(ModelRc::new(VecModel::from(kept)));
}

/// Re-run the visible filter after a search-text change.
pub fn folders_tree_search(window: &AppWindow, query: &str) {
    let s = window.global::<LocalLibraryState>();
    s.set_folders_tree_search(query.into());
    derive_folder_tree_visible(&s);
}
