use reqwest::StatusCode;
use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    // === Get endpoints ===

    /// Get album by ID
    pub async fn get_album(&self, album_id: &str) -> Result<Album> {
        let url = endpoints::build_url(paths::ALBUM_GET);
        let http_response = self
            .signed_get(&url, "albumget", &[("album_id", album_id.to_string())])
            .await?;
        let status = http_response.status();
        log::debug!("[API] get_album({}) status={}", album_id, status);

        if status == StatusCode::NOT_FOUND {
            log::warn!(
                "[API] get_album({}) returned 404 — album not found",
                album_id
            );
            return Err(ApiError::ApiResponse(format!(
                "Album {} not found (404)",
                album_id
            )));
        }
        if !status.is_success() {
            log::error!("[API] get_album({}) unexpected status={}", album_id, status);
            return Err(ApiError::ApiResponse(format!(
                "get_album({}) status {}",
                album_id, status
            )));
        }

        let response: Value = http_response.json().await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Get featured albums by type (new-releases, press-awards, most-streamed)
    pub async fn get_featured_albums(
        &self,
        featured_type: &str,
        limit: u32,
        offset: u32,
        genre_id: Option<u64>,
    ) -> Result<SearchResultsPage<Album>> {
        let url = endpoints::build_url(paths::ALBUM_GET_FEATURED);
        let mut query = vec![
            ("type".to_string(), featured_type.to_string()),
            ("limit".to_string(), limit.to_string()),
            ("offset".to_string(), offset.to_string()),
        ];

        if let Some(gid) = genre_id {
            query.push(("genre_id".to_string(), gid.to_string()));
        }

        let params: Vec<(&str, String)> = query.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        let http_response = self
            .signed_get(&url, "albumgetFeatured", &params)
            .await?;
        log::debug!(
            "[API] get_featured_albums({}) status={}",
            featured_type,
            http_response.status()
        );
        let response: Value = http_response.json().await?;

        let albums = response
            .get("albums")
            .ok_or_else(|| ApiError::ApiResponse("No albums in response".to_string()))?;

        Ok(serde_json::from_value(albums.clone())?)
    }

    /// Albums similar to a seed album (`/album/suggest`). Ported from
    /// the legacy api client.
    pub async fn get_album_suggest(&self, album_id: &str) -> Result<AlbumSuggestResponse> {
        let url = endpoints::build_url(paths::ALBUM_SUGGEST);
        let http_response = self
            .http()?
            .get(&url)
            .headers(self.api_headers().await?)
            .query(&[("album_id", album_id)])
            .send()
            .await?;
        let status = http_response.status();
        if !status.is_success() {
            return Err(ApiError::ApiResponse(format!(
                "get_album_suggest({album_id}) status {status}"
            )));
        }
        let response: Value = http_response.json().await?;
        Ok(serde_json::from_value(response)?)
    }
}
