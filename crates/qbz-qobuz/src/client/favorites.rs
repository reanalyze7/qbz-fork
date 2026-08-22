use reqwest::StatusCode;
use serde_json::Value;

use super::{body_preview, QobuzClient};
use crate::auth::{get_timestamp, sign_get_favorites, sign_search};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Get user favorites (requires auth + signature)
    pub async fn get_favorites(&self, fav_type: &str, limit: u32, offset: u32) -> Result<Value> {
        // Back off before the network if the 403 breaker is open (issue #637).
        self.forbidden_guard()?;
        let url = endpoints::build_url(paths::FAVORITE_GET_USER_FAVORITES);
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let signature = sign_get_favorites(timestamp, &secret);

        let http_response = self
            .http()?
            .get(&url)
            .headers(self.authenticated_headers().await?)
            .query(&[
                ("type", fav_type),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
                ("request_ts", &timestamp.to_string()),
                ("request_sig", &signature),
            ])
            .send()
            .await?;
        let status = http_response.status();
        log::debug!("[API] get_favorites({}) status={}", fav_type, status);
        // Feed the breaker AND check status BEFORE decoding: a 403's body is not
        // our JSON envelope (it's an edge/WAF HTML/empty body), so a bare
        // `.json()` surfaced it as a misleading "error decoding response body"
        // instead of the real 403 (issue #637).
        self.note_forbidden_status(status);
        if !status.is_success() {
            let preview = body_preview(http_response).await;
            if status == StatusCode::FORBIDDEN {
                log::warn!("get_favorites({}) 403{}", fav_type, preview);
                return Err(ApiError::Forbidden(preview));
            }
            return Err(ApiError::ApiResponse(format!(
                "get_favorites failed with status {}{}",
                status, preview
            )));
        }
        let response: Value = http_response.json().await?;

        Ok(response)
    }

    /// Get user's playlists
    pub async fn get_user_playlists(&self) -> Result<Vec<Playlist>> {
        let url = endpoints::build_url(paths::PLAYLIST_GET_USER_PLAYLISTS);
        let http_response = self
            .signed_get_auth(&url, "playlistgetUserPlaylists", &[])
            .await?;
        log::debug!("[API] get_user_playlists status={}", http_response.status());
        let response: Value = http_response.json().await?;

        let playlists = response
            .get("playlists")
            .and_then(|p| p.get("items"))
            .ok_or_else(|| ApiError::ApiResponse("No playlists in response".to_string()))?;

        Ok(serde_json::from_value(playlists.clone())?)
    }

    /// Search playlists
    pub async fn search_playlists(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResultsPage<Playlist>> {
        let url = endpoints::build_url(paths::PLAYLIST_SEARCH);
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let signature = sign_search("playlistsearch", query, limit, offset, None, timestamp, &secret);
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        let ts_str = timestamp.to_string();

        let http_response = self
            .http()?
            .get(&url)
            .headers(self.api_headers().await?)
            .query(&[
                ("query", query),
                ("limit", &limit_str),
                ("offset", &offset_str),
                ("request_ts", &ts_str),
                ("request_sig", &signature),
            ])
            .send()
            .await?;
        log::debug!("[API] search_playlists status={}", http_response.status());
        let response: Value = http_response.json().await?;

        let playlists = response
            .get("playlists")
            .ok_or_else(|| ApiError::ApiResponse("No playlists in response".to_string()))?;

        Ok(serde_json::from_value(playlists.clone())?)
    }

}
