//! Multi-select mode toggle + collapse-all.

use slint::ComponentHandle;

use crate::{AppWindow, LocalLibraryState};

use crate::local_library::folders_tree::nodes::{apply_tree, collect_tree};

use super::state::tree_selected;

/// Collapse every expanded folder — keep only the depth-0 roots.
pub fn collapse_all_tree(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let mut nodes = collect_tree(&s);
    nodes.retain(|n| n.depth == 0);
    for n in nodes.iter_mut() {
        n.expanded = false;
    }
    apply_tree(&s, nodes);
}

/// Toggle multi-select mode; leaving it clears the selection.
pub fn toggle_tree_select_mode(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let on = !s.get_tree_select_mode();
    s.set_tree_select_mode(on);
    if !on {
        tree_selected().clear();
        let nodes = collect_tree(&s);
        apply_tree(&s, nodes);
    }
}

/// Clear the tree selection.
pub fn tree_clear_selection(window: &AppWindow) {
    tree_selected().clear();
    let s = window.global::<LocalLibraryState>();
    let nodes = collect_tree(&s);
    apply_tree(&s, nodes);
}
