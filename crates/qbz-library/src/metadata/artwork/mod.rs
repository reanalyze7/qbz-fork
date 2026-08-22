//! Artwork extraction and thumbnail caching.

mod folder;
mod scoring;

use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

use crate::thumbnails::{generate_thumbnail, generate_thumbnail_from_bytes};

use super::MetadataExtractor;

impl MetadataExtractor {
    /// Extract and save artwork as thumbnail to cache directory
    pub fn extract_artwork(file_path: &Path, _cache_dir: &Path) -> Option<String> {
        if qbz_dsd::is_dsd_path(file_path) {
            let demux = qbz_dsd::open_dsd(file_path).ok()?;
            let art = demux.info().tags.artwork.clone()?;
            let cache_key = file_path.to_string_lossy().to_string();
            return match generate_thumbnail_from_bytes(&art, &cache_key) {
                Ok(thumbnail_path) => Some(thumbnail_path.to_string_lossy().to_string()),
                Err(e) => {
                    log::warn!("Failed to generate DSD thumbnail for {:?}: {}", file_path, e);
                    None
                }
            };
        }

        let tagged_file = Probe::open(file_path).ok()?.read().ok()?;
        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())?;

        let picture = tag.pictures().first()?;

        let cache_key = file_path.to_string_lossy().to_string();

        match generate_thumbnail_from_bytes(picture.data(), &cache_key) {
            Ok(thumbnail_path) => Some(thumbnail_path.to_string_lossy().to_string()),
            Err(e) => {
                log::warn!("Failed to generate thumbnail for {:?}: {}", file_path, e);
                None
            }
        }
    }

    /// Generate thumbnail from an existing artwork file
    pub fn cache_artwork_file(artwork_path: &Path, _cache_dir: &Path) -> Option<String> {
        if !artwork_path.is_file() {
            return None;
        }

        match generate_thumbnail(artwork_path) {
            Ok(thumbnail_path) => Some(thumbnail_path.to_string_lossy().to_string()),
            Err(e) => {
                log::warn!("Failed to generate thumbnail for {:?}: {}", artwork_path, e);
                None
            }
        }
    }
}
