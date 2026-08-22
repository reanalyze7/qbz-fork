//! Shared image download helper + the URL-based artwork download entry point.

use std::fs;
use std::path::Path;

use super::{DiscogsClient, DISCOGS_PROXY_URL};

impl DiscogsClient {
    /// Download image from URL and return local path
    pub async fn download_artwork_from_url(
        &self,
        image_url: &str,
        cache_dir: &Path,
        artist: &str,
        album: &str,
    ) -> Result<String, String> {
        // Generate cache filename
        let filename = format!(
            "discogs_{:x}.jpg",
            Self::simple_hash(&format!("{}_{}", artist, album))
        );
        let cache_path = cache_dir.join(&filename);

        // Download the image
        self.download_image(image_url, &cache_path)
            .await
            .ok_or_else(|| "Failed to download image".to_string())?;

        Ok(cache_path.to_string_lossy().to_string())
    }

    /// Download an image to the cache directory
    pub(super) async fn download_image(&self, image_url: &str, path: &Path) -> Option<()> {
        log::debug!("Downloading Discogs artwork: {}", image_url);

        // Use proxy to download image with authentication
        let proxy_url = format!(
            "{}/image?url={}",
            DISCOGS_PROXY_URL,
            urlencoding::encode(image_url)
        );

        let response = self.client.get(&proxy_url).send().await.ok()?;

        if !response.status().is_success() {
            log::warn!("Failed to download Discogs image: {}", response.status());
            return None;
        }

        let bytes = response.bytes().await.ok()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok()?;
        }

        fs::write(path, &bytes).ok()?;

        log::info!("Saved Discogs artwork to: {}", path.display());
        Some(())
    }
}
