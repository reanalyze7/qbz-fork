//! Settings > Local Library controller (Slint).
//!
//! Hosts the folder-management surface that Tauri renders inline in the
//! browse view's gear panel: the folder list (add / remove / edit / enable /
//! alias / network override), maintenance (cleanup missing files), and the
//! two-step danger-zone clear. The scan engine + progress live in Slice B.
//!
//! All DB access goes through the frontend-agnostic `qbz_library` crate via
//! `crate::library_db::with_db(|db| …)` on `spawn_blocking` (rusqlite is
//! blocking). The authoritative full folder set (with per-row selection) is
//! kept in a module static; the Slint `LibraryFoldersState.folders` model is
//! the filtered render set derived from it.

mod crud_picker;
mod crud_remove;
mod danger_zone;
mod edit_modal;
mod load;
mod maintenance;
mod scan;
mod scan_actions;
mod scan_sink;
mod state;

pub use crud_picker::{add_folder, change_folder_path};
pub use crud_remove::{remove_folder, remove_folders, toggle_select};
pub use danger_zone::{clear_library, set_filter};
pub use edit_modal::{edit_folder, save_folder_settings};
pub use load::load_folders;
pub use maintenance::cleanup_missing;
pub use scan_actions::{scan_all, scan_folder, stop_scan};
