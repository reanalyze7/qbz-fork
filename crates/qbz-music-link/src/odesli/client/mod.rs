//! Odesli/song.link networking client with an in-memory TTL cache.

use std::collections::HashMap;
use std::sync::Mutex;

use reqwest::Client;

mod cache;
mod convert;

use cache::CacheEntry;

use super::error::ShareError;
use super::models::OdesliResponse;
use super::simplified::{ContentType, SongLinkResponse};
use super::{ODESLI_API_URL, REQUEST_TIMEOUT};

/// Odesli/song.link client with caching
pub struct SongLinkClient {
    client: Client,
    cache: Mutex<HashMap<String, CacheEntry>>,
}

impl Default for SongLinkClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SongLinkClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .connect_timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Get song.link URL by URL (fallback when ISRC/UPC are missing)
    pub async fn get_by_url(
        &self,
        url: &str,
        content_type: ContentType,
    ) -> Result<SongLinkResponse, ShareError> {
        let cache_key = format!("url:{}", url);

        if let Some(cached) = self.get_from_cache(&cache_key) {
            log::debug!("Cache hit for URL: {}", url);
            return Ok(cached);
        }

        log::info!("Fetching song.link for URL: {}", url);

        let response = self
            .client
            .get(ODESLI_API_URL)
            .query(&[("url", url), ("userCountry", "US")])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            // Provide a friendlier message for common errors
            if status.as_u16() == 400 && text.contains("could_not_resolve_entity") {
                return Err(ShareError::OdesliError(
                    "Track not found on any supported platform. Try a track with an ISRC code."
                        .to_string(),
                ));
            }
            return Err(ShareError::OdesliError(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        let odesli: OdesliResponse = response.json().await?;
        let result = self.convert_response(odesli, url.to_string(), content_type)?;

        self.store_in_cache(cache_key, result.clone());
        Ok(result)
    }
}
