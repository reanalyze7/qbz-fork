use reqwest::StatusCode;
use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Get track by ID
    pub async fn get_track(&self, track_id: u64) -> Result<Track> {
        let url = endpoints::build_url(paths::TRACK_GET);
        let http_response = self
            .signed_get(&url, "trackget", &[("track_id", track_id.to_string())])
            .await?;
        let status = http_response.status();
        log::debug!("[API] get_track({}) status={}", track_id, status);

        if status == StatusCode::NOT_FOUND {
            log::warn!(
                "[API] get_track({}) returned 404 — track no longer available",
                track_id
            );
            return Err(ApiError::TrackUnavailable(track_id));
        }
        if !status.is_success() {
            log::error!("[API] get_track({}) unexpected status={}", track_id, status);
            return Err(ApiError::ApiResponse(format!(
                "get_track({}) status {}",
                track_id, status
            )));
        }

        let response: Value = http_response.json().await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Get artist by ID (basic info only - no albums, faster response)
    pub async fn get_artist_basic(&self, artist_id: u64) -> Result<Artist> {
        let url = endpoints::build_url(paths::ARTIST_GET);
        let locale = self.locale().await;
        let query = vec![
            ("artist_id", artist_id.to_string()),
            ("lang", locale),
            // No "extra" parameter = only basic info (id, name, image)
        ];

        let http_response = self
            .signed_get(&url, "artistget", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?;
        log::debug!(
            "[API] get_artist_basic({}) status={}",
            artist_id,
            http_response.status()
        );
        let response: Value = http_response.json().await?;

        Ok(serde_json::from_value(response)?)
    }

    /// Get artist by ID
    pub async fn get_artist(&self, artist_id: u64, with_albums: bool) -> Result<Artist> {
        self.get_artist_with_pagination(artist_id, with_albums, None, None)
            .await
    }

    /// Get artist detail by ID with albums, playlists, and appears-on tracks
    pub async fn get_artist_detail(
        &self,
        artist_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Artist> {
        let url = endpoints::build_url(paths::ARTIST_GET);
        let locale = self.locale().await;
        let mut query = vec![
            ("artist_id", artist_id.to_string()),
            ("extra", "albums,tracks_appears_on,playlists".to_string()),
            ("lang", locale),
        ];
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }
        if let Some(o) = offset {
            query.push(("offset", o.to_string()));
        }

        let http_response = self
            .signed_get(&url, "artistget", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?;
        log::debug!(
            "[API] get_artist_detail({}) status={}",
            artist_id,
            http_response.status()
        );
        let response: Value = http_response.json().await?;

        Ok(serde_json::from_value(response)?)
    }
}
