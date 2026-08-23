//! Folder-detail pane: direct child tracks + immediate subfolders.

mod derive;
mod fetch;
mod select;

pub use derive::{folder_detail_search, set_folder_detail_subfolder_artwork};
pub use select::select_folder;
