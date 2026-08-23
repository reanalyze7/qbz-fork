//! Tree rail: multi-select store + operations.

mod mode;
mod select_all;
mod state;
mod toggle;

pub use mode::{collapse_all_tree, toggle_tree_select_mode, tree_clear_selection};
pub use select_all::tree_select_all;
pub use state::tree_selected_snapshot;
pub use toggle::{toggle_tree_folder_select, toggle_tree_track_select};

pub(crate) use state::tree_selected;
