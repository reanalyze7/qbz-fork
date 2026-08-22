//! Thumbnail cache maintenance

use std::fs;

use crate::LibraryError;

use super::paths::get_thumbnails_dir;

/// Clear all thumbnails (useful for cache cleanup)
pub fn clear_thumbnails() -> Result<(), LibraryError> {
    let thumbnails_dir = get_thumbnails_dir()?;

    if thumbnails_dir.exists() {
        fs::remove_dir_all(&thumbnails_dir)
            .map_err(|e| LibraryError::Other(format!("Failed to clear thumbnails: {}", e)))?;
        fs::create_dir_all(&thumbnails_dir).map_err(|e| {
            LibraryError::Other(format!("Failed to recreate thumbnails directory: {}", e))
        })?;
    }

    Ok(())
}

/// Get the total size of the thumbnails cache in bytes
pub fn get_cache_size() -> Result<u64, LibraryError> {
    let thumbnails_dir = get_thumbnails_dir()?;

    if !thumbnails_dir.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;

    for entry in fs::read_dir(&thumbnails_dir)
        .map_err(|e| LibraryError::Other(format!("Failed to read thumbnails directory: {}", e)))?
    {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }

    Ok(total_size)
}
