//! Folders tab (tree mode).
//!
//! Slint has no native TreeView and no self-recursive component, so the tree
//! is rendered as a flattened list of visible nodes (`FolderNode`) in a
//! ListView: each node carries its `depth` (drives the indent) and an
//! `expanded` flag. Expanding fetches one level lazily via
//! `list_folder_children`; collapsing drops the contiguous descendant block.

mod detail;
mod load;
pub(crate) mod nodes;
mod search;
pub(crate) mod select;

pub use detail::*;
pub use load::{ensure_folder_tree_loaded, toggle_folder_node};
pub use search::folders_tree_search;
pub use select::{
    collapse_all_tree, toggle_tree_folder_select, toggle_tree_select_mode, toggle_tree_track_select,
    tree_clear_selection, tree_select_all, tree_selected_snapshot,
};
