//! Artwork jobs for the grid/list/tree covers, plus the folders' decoded
//! custom images (local files, read + decoded directly here — the URL
//! pipeline only handles http(s)).

mod folder_images;
mod jobs;

pub use folder_images::{folder_for_edit, load_editor_custom_image, load_folder_custom_images};
pub use jobs::artwork_jobs;
