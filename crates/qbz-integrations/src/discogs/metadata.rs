//! Full release metadata lookup (used by the tag editor after a search hit).

use super::types::DiscogsReleaseMetadata;
use super::{DiscogsClient, DISCOGS_PROXY_URL};

impl DiscogsClient {
    /// Get full release metadata including tracklist
    pub async fn get_release_metadata(
        &self,
        release_id: u64,
    ) -> Result<DiscogsReleaseMetadata, String> {
        let url = format!("{}/release/{}", DISCOGS_PROXY_URL, release_id);

        log::debug!("Fetching Discogs release metadata for ID: {}", release_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch release: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch release: {}", response.status()));
        }

        let metadata: DiscogsReleaseMetadata = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse release metadata: {}", e))?;

        log::info!(
            "Fetched Discogs release: {} ({:?})",
            metadata.title,
            metadata.year
        );
        Ok(metadata)
    }
}
