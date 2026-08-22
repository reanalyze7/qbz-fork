//! Fetch artwork bytes and write them somewhere: embed into a FLAC's
//! primary tag, or save as a standalone `cover.jpg` next to an album dir.

use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::config::WriteOptions;
use lofty::tag::Tag;
use std::path::Path;

/// Download and embed artwork in FLAC file
pub async fn embed_artwork(file_path: &str, artwork_url: &str) -> Result<(), String> {
    log::info!("Embedding artwork from: {}", artwork_url);

    // Download artwork
    let response = reqwest::get(artwork_url)
        .await
        .map_err(|e| format!("Failed to download artwork: {}", e))?;

    let artwork_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read artwork bytes: {}", e))?;

    // Determine MIME type from URL
    let mime_type = if artwork_url.contains(".jpg") || artwork_url.contains(".jpeg") {
        MimeType::Jpeg
    } else if artwork_url.contains(".png") {
        MimeType::Png
    } else {
        MimeType::Jpeg // Default to JPEG
    };

    // Create picture
    let picture = Picture::unchecked(artwork_bytes.to_vec())
        .pic_type(PictureType::CoverFront)
        .mime_type(mime_type)
        .build();

    // Read file
    let path = Path::new(file_path);
    let mut tagged_file =
        lofty::read_from_path(path).map_err(|e| format!("Failed to read FLAC file: {}", e))?;

    // Add picture to primary tag
    if let Some(tag) = tagged_file.primary_tag_mut() {
        tag.push_picture(picture);
    } else {
        let tag_type = tagged_file.primary_tag_type();
        let mut tag = Tag::new(tag_type);
        tag.push_picture(picture);
        tagged_file.insert_tag(tag);
    }

    // Save
    tagged_file
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Failed to save artwork: {}", e))?;

    Ok(())
}

/// Download and save album cover art as a file
pub async fn save_album_artwork(album_dir: &Path, artwork_url: &str) -> Result<(), String> {
    log::info!("Downloading album artwork to: {:?}", album_dir);

    let cover_path = album_dir.join("cover.jpg");

    // Skip if cover already exists
    if cover_path.exists() {
        log::debug!("Cover art already exists at {:?}", cover_path);
        return Ok(());
    }

    // Download artwork
    let response = reqwest::get(artwork_url)
        .await
        .map_err(|e| format!("Failed to download album artwork: {}", e))?;

    let artwork_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read album artwork bytes: {}", e))?;

    // Write to file
    std::fs::write(&cover_path, artwork_bytes)
        .map_err(|e| format!("Failed to write cover.jpg: {}", e))?;

    log::info!("Album artwork saved to: {:?}", cover_path);
    Ok(())
}
