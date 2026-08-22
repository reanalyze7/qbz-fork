use serde_json::Value;

use super::QobuzClient;
use crate::auth::{get_timestamp, sign_search};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Catalog search (combined: albums, tracks, artists, playlists, most_popular).
    /// Returns raw JSON for caller to parse — the response shape is complex.
    pub async fn catalog_search(&self, query: &str, limit: u32, offset: u32) -> Result<Value> {
        let url = endpoints::build_url(paths::CATALOG_SEARCH);
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let signature = sign_search("catalogsearch", query, limit, offset, None, timestamp, &secret);
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
        log::debug!("[API] catalog_search status={}", http_response.status());
        let response: Value = http_response.json().await?;
        Ok(response)
    }

    /// Get similar artists for an artist ID
    pub async fn get_similar_artists(
        &self,
        artist_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResultsPage<Artist>> {
        let url = endpoints::build_url(paths::ARTIST_GET_SIMILAR);
        let http_response = self
            .signed_get(&url, "artistgetSimilarArtists", &[
                ("artist_id", artist_id.to_string()),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .await?;
        log::debug!(
            "[API] get_similar_artists({}) status={}",
            artist_id,
            http_response.status()
        );
        let response: Value = http_response.json().await?;

        let artists = response
            .get("artists")
            .ok_or_else(|| ApiError::ApiResponse("No artists in response".to_string()))?;

        Ok(serde_json::from_value(artists.clone())?)
    }

    /// Get an artist's tracks (public endpoint via artist/get?extra=tracks)
    pub async fn get_artist_tracks(
        &self,
        artist_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<TracksContainer> {
        let url = endpoints::build_url(paths::ARTIST_GET);
        let locale = self.locale().await;

        let http_response = self
            .signed_get(&url, "artistget", &[
                ("artist_id", artist_id.to_string()),
                ("extra", "tracks".to_string()),
                ("lang", locale),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .await?;
        log::debug!(
            "[API] get_artist_tracks({}) status={}",
            artist_id,
            http_response.status()
        );
        let response: Value = http_response.json().await?;

        let tracks = response
            .get("tracks")
            .ok_or_else(|| ApiError::ApiResponse("No tracks in artist response".to_string()))?;

        Ok(serde_json::from_value(tracks.clone())?)
    }
}
