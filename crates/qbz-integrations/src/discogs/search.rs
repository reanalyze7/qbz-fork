//! Tag-editor search surface: artist search and extended release search.

use serde::Deserialize;

use super::types::{DiscogsSearchResultExtended, SearchResponse};
use super::{DiscogsClient, DISCOGS_PROXY_URL};

impl DiscogsClient {
    /// Search for artists and return search results
    pub async fn search_artist(&self, query: &str) -> Result<SearchResponse, String> {
        let url = format!(
            "{}/search?q={}&type=artist",
            DISCOGS_PROXY_URL,
            urlencoding::encode(query)
        );

        log::debug!("Searching Discogs for artist: {}", query);

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

        Ok(search)
    }

    /// Search for releases with extended metadata
    /// Returns up to `limit` results with detailed release information
    pub async fn search_releases(
        &self,
        artist: &str,
        album: &str,
        catalog_number: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DiscogsSearchResultExtended>, String> {
        // Build search query - prefer catalog number if available
        let query = if let Some(catno) = catalog_number.filter(|s| !s.trim().is_empty()) {
            catno.to_string()
        } else {
            format!("{} {}", artist, album)
        };

        let url = format!(
            "{}/search?q={}&type=release&per_page={}",
            DISCOGS_PROXY_URL,
            urlencoding::encode(&query),
            limit.min(25)
        );

        log::debug!(
            "Searching Discogs releases for: {} - {} (catalog: {:?})",
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

        // Parse response with extended fields
        #[derive(Debug, Deserialize)]
        struct ExtendedSearchResponse {
            results: Vec<DiscogsSearchResultExtended>,
        }

        let search: ExtendedSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Discogs response: {}", e))?;

        // Filter to releases only
        let results: Vec<_> = search
            .results
            .into_iter()
            .filter(|r| r.result_type == "release" || r.result_type == "master")
            .collect();

        log::info!("Found {} Discogs releases", results.len());
        Ok(results)
    }
}
