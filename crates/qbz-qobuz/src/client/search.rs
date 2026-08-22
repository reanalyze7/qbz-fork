use serde_json::Value;

use super::QobuzClient;
use crate::auth::{get_timestamp, sign_search};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    // === Search endpoints ===

    /// Search for albums
    /// Optional search_type: "MainArtist", "Performer", "Composer", "Label", "ReleaseName"
    pub async fn search_albums(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
        search_type: Option<&str>,
    ) -> Result<SearchResultsPage<Album>> {
        let url = endpoints::build_url(paths::ALBUM_SEARCH);
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let signature = sign_search("albumsearch", query, limit, offset, search_type, timestamp, &secret);
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        let ts_str = timestamp.to_string();

        let mut params: Vec<(&str, &str)> = vec![
            ("query", query),
            ("limit", &limit_str),
            ("offset", &offset_str),
            ("request_ts", &ts_str),
            ("request_sig", &signature),
        ];

        if let Some(st) = search_type {
            params.push(("type", st));
        }

        let http_response = self
            .http()?
            .get(&url)
            .headers(self.api_headers().await?)
            .query(&params)
            .send()
            .await?;
        log::debug!("[API] search_albums status={}", http_response.status());
        let response: Value = http_response.json().await?;

        let albums = response
            .get("albums")
            .ok_or_else(|| ApiError::ApiResponse("No albums in response".to_string()))?;

        Ok(serde_json::from_value(albums.clone())?)
    }

    /// Search for tracks
    /// Optional search_type: "MainArtist", "Performer", "Composer", "Label", "ReleaseName"
    pub async fn search_tracks(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
        search_type: Option<&str>,
    ) -> Result<SearchResultsPage<Track>> {
        let url = endpoints::build_url(paths::TRACK_SEARCH);
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let signature = sign_search("tracksearch", query, limit, offset, search_type, timestamp, &secret);
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        let ts_str = timestamp.to_string();

        let mut params: Vec<(&str, &str)> = vec![
            ("query", query),
            ("limit", &limit_str),
            ("offset", &offset_str),
            ("request_ts", &ts_str),
            ("request_sig", &signature),
        ];

        if let Some(st) = search_type {
            params.push(("type", st));
        }

        let http_response = self
            .http()?
            .get(&url)
            .headers(self.api_headers().await?)
            .query(&params)
            .send()
            .await?;
        log::debug!("[API] search_tracks status={}", http_response.status());
        let response: Value = http_response.json().await?;

        let tracks = response
            .get("tracks")
            .ok_or_else(|| ApiError::ApiResponse("No tracks in response".to_string()))?;

        Ok(serde_json::from_value(tracks.clone())?)
    }

}
