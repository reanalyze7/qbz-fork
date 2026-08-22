//! Multi-image artwork picker: search + per-release detail fetch.

use super::artwork_options_helpers::{build_artwork_query, collect_other_result_images, split_top_releases};
use super::types::{DiscogsImageOption, SearchResponse};
use super::{DiscogsClient, DISCOGS_PROXY_URL};

impl DiscogsClient {
    /// Search for album artwork options
    /// Returns up to 10 image options, with detailed images from top 2 releases interleaved
    /// If catalog_number is provided, searches by that first, then falls back to artist + album
    pub async fn search_artwork_options(
        &self,
        artist: &str,
        album: &str,
        catalog_number: Option<&str>,
    ) -> Result<Vec<DiscogsImageOption>, String> {
        let query = build_artwork_query(artist, album, catalog_number);
        let url = format!(
            "{}/search?q={}&type=release",
            DISCOGS_PROXY_URL,
            urlencoding::encode(&query)
        );

        log::debug!(
            "Searching Discogs artwork options for: {} - {} (catalog: {:?})",
            artist,
            album,
            catalog_number
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to search Discogs: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Discogs search failed with status: {}",
                response.status()
            ));
        }

        let search: SearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Discogs response: {}", e))?;

        let (release_ids, other_results) = split_top_releases(&search.results);

        if release_ids.is_empty() {
            return Err("No releases found on Discogs".to_string());
        }

        let mut all_images = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        // Fetch detailed images from top 2 releases
        self.collect_top_release_images(&release_ids, &mut all_images, &mut seen_urls)
            .await;

        // Add up to 2 more images from other search results
        collect_other_result_images(&other_results, &mut all_images, &mut seen_urls);

        if all_images.is_empty() {
            return Err("No artwork found on Discogs".to_string());
        }

        // Return up to 10 unique images
        all_images.truncate(10);
        log::info!(
            "Returning {} artwork options from Discogs",
            all_images.len()
        );
        Ok(all_images)
    }

}
