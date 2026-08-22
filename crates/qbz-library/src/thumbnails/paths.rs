//! Thumbnails directory and filename resolution

use std::fs;
use std::path::{Path, PathBuf};

use crate::LibraryError;

/// Default thumbnail size (width and height)
/// 500px is a good balance for UI display while keeping file size reasonable
pub(super) const THUMBNAIL_SIZE: u32 = 500;

/// Get the thumbnails directory path
pub fn get_thumbnails_dir() -> Result<PathBuf, LibraryError> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| LibraryError::Other("Could not find data directory".into()))?;
    let thumbnails_dir = data_dir.join("qbz").join("thumbnails");

    // Create directory if it doesn't exist
    if !thumbnails_dir.exists() {
        fs::create_dir_all(&thumbnails_dir).map_err(|e| {
            LibraryError::Other(format!("Failed to create thumbnails directory: {}", e))
        })?;
    }

    Ok(thumbnails_dir)
}

/// Hash an arbitrary string input into a `.jpg` thumbnail filename
pub(super) fn hash_to_filename(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}.jpg", hasher.finish())
}

/// Generate a unique filename for a thumbnail based on the source path
fn get_thumbnail_filename(source_path: &Path) -> String {
    hash_to_filename(&source_path.to_string_lossy())
}

/// Get the thumbnail path for a source image
pub fn get_thumbnail_path(source_path: &Path) -> Result<PathBuf, LibraryError> {
    let thumbnails_dir = get_thumbnails_dir()?;
    let filename = get_thumbnail_filename(source_path);
    Ok(thumbnails_dir.join(filename))
}

/// Check if a thumbnail exists for the given source path
pub fn thumbnail_exists(source_path: &Path) -> Result<bool, LibraryError> {
    let thumbnail_path = get_thumbnail_path(source_path)?;
    Ok(thumbnail_path.exists())
}
