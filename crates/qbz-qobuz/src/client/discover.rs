use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Get list of genres
    pub async fn get_genres(&self, parent_id: Option<u64>) -> Result<Vec<GenreInfo>> {
        let url = endpoints::build_url(paths::GENRE_LIST);
        // Force English for consistent genre names across all user regions
        let mut query: Vec<(&str, String)> = vec![("lang", "en".to_string())];

        if let Some(pid) = parent_id {
            query.push(("parent_id", pid.to_string()));
        }

        let http_response = self
            .signed_get(&url, "genrelist", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?;
        log::debug!(
            "[API] get_genres(parent={:?}) status={}",
            parent_id,
            http_response.status()
        );
        let response: serde_json::Value = http_response.json().await?;

        let genres = response
            .get("genres")
            .and_then(|g| g.get("items"))
            .ok_or_else(|| ApiError::ApiResponse("No genres in response".to_string()))?;

        Ok(serde_json::from_value(genres.clone())?)
    }

    /// Get discover index (home page content: playlists, ideal discography, etc.)
    pub async fn get_discover_index(
        &self,
        genre_ids: Option<Vec<u64>>,
    ) -> Result<DiscoverResponse> {
        let url = endpoints::build_url(paths::DISCOVER_INDEX);
        let mut query: Vec<(&str, String)> = vec![];

        // Add genre_ids as comma-separated list if provided
        if let Some(gids) = genre_ids {
            if !gids.is_empty() {
                let ids_str = gids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                query.push(("genre_ids", ids_str));
            }
        }

        let http_response = self
            .signed_get_auth(&url, "discoverindex", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?;
        log::info!(
            "[API] get_discover_index genre_ids={:?} status={}",
            query,
            http_response.status()
        );
        let response: serde_json::Value = http_response.json().await?;

        // Debug: log the response structure
        if let Some(obj) = response.as_object() {
            log::info!(
                "Discover API response keys: {:?}",
                obj.keys().collect::<Vec<_>>()
            );
            if let Some(err) = obj.get("message") {
                log::error!("Discover API error: {:?}", err);
            }
            if let Some(code) = obj.get("code") {
                log::error!("Discover API error code: {:?}", code);
            }
        }

        Ok(serde_json::from_value(response)?)
    }

    /// Get discover albums from a specific browse endpoint (newReleases, idealDiscography, mostStreamed)
    pub async fn get_discover_albums(
        &self,
        endpoint: &str,
        genre_ids: Option<Vec<u64>>,
        offset: u32,
        limit: u32,
    ) -> Result<DiscoverData<DiscoverAlbum>> {
        let url = endpoints::build_url(endpoint);
        let mut query: Vec<(&str, String)> = vec![];

        if let Some(gids) = genre_ids {
            if !gids.is_empty() {
                let ids_str = gids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                query.push(("genre_ids", ids_str));
            }
        }

        query.push(("offset", offset.to_string()));
        query.push(("limit", limit.to_string()));

        // Derive method name from endpoint path: "/discover/newReleases" -> "discovernewReleases"
        let method_name = endpoint.replace('/', "").replace('.', "");
        let http_response = self
            .signed_get_auth(&url, &method_name, &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?;
        log::info!(
            "[API] get_discover_albums({}) query={:?} status={}",
            endpoint,
            query,
            http_response.status()
        );
        let response: serde_json::Value = http_response.json().await?;

        Ok(serde_json::from_value(response)?)
    }
}
