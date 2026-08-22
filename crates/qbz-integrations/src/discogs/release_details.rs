//! Per-release detail fetch + image collection for the artwork picker.

use super::types::{DiscogsImageOption, ReleaseDetails};
use super::{DiscogsClient, DISCOGS_PROXY_URL};

impl DiscogsClient {
    /// Get detailed release information including all images
    pub(super) async fn get_release_details(
        &self,
        release_id: u64,
    ) -> Result<ReleaseDetails, String> {
        let url = format!("{}/release/{}", DISCOGS_PROXY_URL, release_id);

        log::debug!("Fetching Discogs release details for ID: {}", release_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch release details: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to fetch release details: {}",
                response.status()
            ));
        }

        let details: ReleaseDetails = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse release details: {}", e))?;

        Ok(details)
    }

    /// Fetch and append images from the top N release IDs (up to 4 images each).
    pub(super) async fn collect_top_release_images(
        &self,
        release_ids: &[u64],
        all_images: &mut Vec<DiscogsImageOption>,
        seen_urls: &mut std::collections::HashSet<String>,
    ) {
        for (idx, release_id) in release_ids.iter().enumerate() {
            match self.get_release_details(*release_id).await {
                Ok(details) => {
                    if let Some(images) = details.images {
                        let mut count = 0;
                        for img in images {
                            if !img.uri.is_empty()
                                && !img.uri.contains("spacer.gif")
                                && seen_urls.insert(img.uri.clone())
                                && count < 4
                            {
                                all_images.push(DiscogsImageOption {
                                    url: img.uri,
                                    width: img.width,
                                    height: img.height,
                                    image_type: img.image_type,
                                    release_title: Some(details.title.clone()),
                                    release_year: details.year,
                                });
                                count += 1;
                            }
                        }
                        log::debug!(
                            "Added {} images from release #{} ({})",
                            count,
                            idx + 1,
                            details.title
                        );
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch details for release {}: {}", release_id, e);
                }
            }
        }
    }
}
