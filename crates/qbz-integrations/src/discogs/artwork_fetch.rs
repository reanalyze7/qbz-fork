//! Simple artwork search-and-cache path used by the background artwork fetcher.

use std::path::Path;

use super::types::SearchResponse;
use super::{DiscogsClient, DISCOGS_PROXY_URL};

impl DiscogsClient {
    /// Search for album artwork and download if found
    /// Returns the path to the downloaded image or None
    pub async fn fetch_artwork(
        &self,
        artist: &str,
        album: &str,
        cache_dir: &Path,
    ) -> Option<String> {
        // Search for the release
        let cover_url = self.search_release(artist, album).await?;

        // Generate cache filename
        let filename = format!(
            "discogs_{:x}.jpg",
            Self::simple_hash(&format!("{}_{}", artist, album))
        );
        let cache_path = cache_dir.join(&filename);

        // Return cached if exists
        if cache_path.exists() {
            return Some(cache_path.to_string_lossy().to_string());
        }

        // Download the image
        self.download_image(&cover_url, &cache_path).await?;

        Some(cache_path.to_string_lossy().to_string())
    }

    /// Search for a release and return the cover image URL
    async fn search_release(&self, artist: &str, album: &str) -> Option<String> {
        // Build search query
        let query = format!("{} {}", artist, album);
        let url = format!(
            "{}/search?q={}&type=release",
            DISCOGS_PROXY_URL,
            urlencoding::encode(&query)
        );

        log::debug!("Searching Discogs for: {} - {}", artist, album);

        let response = self.client.get(&url).send().await.ok()?;

        if !response.status().is_success() {
            log::warn!("Discogs search failed with status: {}", response.status());
            return None;
        }

        let search: SearchResponse = response.json().await.ok()?;

        // Find first result with a cover image
        for result in search.results {
            if result.result_type == "release" || result.result_type == "master" {
                if let Some(cover) = result.cover_image {
                    if !cover.is_empty() && !cover.contains("spacer.gif") {
                        log::debug!("Found Discogs cover for {} - {}", artist, album);
                        return Some(cover);
                    }
                }
                // Fallback to thumbnail
                if let Some(thumb) = result.thumb {
                    if !thumb.is_empty() && !thumb.contains("spacer.gif") {
                        return Some(thumb);
                    }
                }
            }
        }

        log::debug!("No Discogs cover found for {} - {}", artist, album);
        None
    }
}
