//! Thumbnail decode/resize/save generation

use image::imageops::FilterType;
use image::ImageReader;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::LibraryError;

use super::paths::{get_thumbnail_path, get_thumbnails_dir, hash_to_filename, THUMBNAIL_SIZE};

/// Generate a thumbnail for the given source image
pub fn generate_thumbnail(source_path: &Path) -> Result<PathBuf, LibraryError> {
    let thumbnail_path = get_thumbnail_path(source_path)?;

    // If thumbnail already exists, return it
    if thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }

    // Read source image
    let img = ImageReader::open(source_path)
        .map_err(|e| LibraryError::Other(format!("Failed to open image: {}", e)))?
        .decode()
        .map_err(|e| LibraryError::Other(format!("Failed to decode image: {}", e)))?;

    // Resize to thumbnail size (maintaining aspect ratio, fit within square)
    let thumbnail = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);

    // Save as JPEG with quality 85
    thumbnail
        .save(&thumbnail_path)
        .map_err(|e| LibraryError::Other(format!("Failed to save thumbnail: {}", e)))?;

    Ok(thumbnail_path)
}

/// Generate a thumbnail from image bytes (for embedded artwork)
pub fn generate_thumbnail_from_bytes(
    bytes: &[u8],
    cache_key: &str,
) -> Result<PathBuf, LibraryError> {
    let thumbnails_dir = get_thumbnails_dir()?;

    // Generate filename from cache key
    let filename = hash_to_filename(cache_key);
    let thumbnail_path = thumbnails_dir.join(&filename);

    // If thumbnail already exists, return it
    if thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }

    // Decode image from bytes
    let cursor = Cursor::new(bytes);
    let img = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| LibraryError::Other(format!("Failed to guess image format: {}", e)))?
        .decode()
        .map_err(|e| LibraryError::Other(format!("Failed to decode image: {}", e)))?;

    // Resize to thumbnail size
    let thumbnail = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);

    // Save as JPEG
    thumbnail
        .save(&thumbnail_path)
        .map_err(|e| LibraryError::Other(format!("Failed to save thumbnail: {}", e)))?;

    Ok(thumbnail_path)
}
