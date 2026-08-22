//! Thumbnail generation for album artwork
//!
//! Split into `paths` (directory/filename resolution), `generate`
//! (decode/resize/save), and `cache` (cache maintenance).

mod cache;
mod generate;
mod paths;

pub use cache::{clear_thumbnails, get_cache_size};
pub use generate::{generate_thumbnail, generate_thumbnail_from_bytes};
pub use paths::{get_thumbnail_path, get_thumbnails_dir, thumbnail_exists};

use std::path::{Path, PathBuf};

use crate::LibraryError;

/// Get or generate a thumbnail for an artwork path
/// Returns the path to the thumbnail file
pub fn get_or_generate_thumbnail(artwork_path: &Path) -> Result<PathBuf, LibraryError> {
    let thumbnail_path = get_thumbnail_path(artwork_path)?;

    if thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }

    generate_thumbnail(artwork_path)
}
