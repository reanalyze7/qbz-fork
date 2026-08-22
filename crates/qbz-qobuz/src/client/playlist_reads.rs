use serde_json::Value;

use super::QobuzClient;
use crate::auth::{get_timestamp, sign_request};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Get playlist metadata + ordered track IDs (lightweight, no full Track objects).
    /// Uses `playlist/get?extra=track_ids` which returns the playlist with a flat
    /// array of track IDs instead of nested Track objects.
    pub async fn get_playlist_track_ids(&self, playlist_id: u64) -> Result<PlaylistWithTrackIds> {
        let url = endpoints::build_url(paths::PLAYLIST_GET);
        let http_response = self
            .signed_get(&url, "playlistget", &[
                ("playlist_id", playlist_id.to_string()),
                ("extra", "track_ids".to_string()),
            ])
            .await?;
        log::debug!(
            "[API] get_playlist_track_ids({}) status={}",
            playlist_id,
            http_response.status()
        );
        let response: Value = http_response.json().await?;
        let result: PlaylistWithTrackIds = serde_json::from_value(response)?;
        log::info!(
            "[API] get_playlist_track_ids({}) — {} track IDs",
            playlist_id,
            result.track_ids.len()
        );
        Ok(result)
    }

    /// Fetch full Track objects for a batch of track IDs.
    /// Uses the `track/getList` endpoint, which caps at 50 IDs per call,
    /// so larger inputs are split into 50-ID windows and fetched serially.
    /// Input order is preserved in the returned vector.
    pub async fn get_tracks_batch(&self, track_ids: &[u64]) -> Result<Vec<Track>> {
        const MAX_PER_CALL: usize = 50;

        if track_ids.is_empty() {
            return Ok(Vec::new());
        }

        if track_ids.len() <= MAX_PER_CALL {
            return self.get_tracks_batch_chunk(track_ids).await;
        }

        log::debug!(
            "[API] get_tracks_batch chunking {} IDs into {}-windows",
            track_ids.len(),
            MAX_PER_CALL
        );
        let mut all = Vec::with_capacity(track_ids.len());
        for chunk in track_ids.chunks(MAX_PER_CALL) {
            let mut tracks = self.get_tracks_batch_chunk(chunk).await?;
            all.append(&mut tracks);
        }
        Ok(all)
    }

    /// Single `track/getList` POST. Caller is responsible for keeping
    /// `track_ids.len() <= 50` — `get_tracks_batch` handles that.
    async fn get_tracks_batch_chunk(&self, track_ids: &[u64]) -> Result<Vec<Track>> {
        let url = endpoints::build_url(paths::TRACK_GET_LIST);
        let headers = self.api_headers().await?;
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let ids_str: String = track_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
        let sig = sign_request("trackgetList", &[("tracks_id", &ids_str)], timestamp, &secret);

        let body = serde_json::json!({ "tracks_id": track_ids });
        log::debug!("[API] get_tracks_batch POST ({} IDs)", track_ids.len());

        let http_response = self
            .http()?
            .post(&url)
            .headers(headers)
            .query(&[("request_ts", timestamp.to_string()), ("request_sig", sig)])
            .json(&body)
            .send()
            .await?;

        let status = http_response.status();
        log::debug!("[API] get_tracks_batch POST status={}", status);

        let value: Value = http_response.json().await?;

        // Response: { "tracks": { "total": N, "items": [...] } }
        let items = value
            .get("tracks")
            .and_then(|t| t.get("items"))
            .ok_or_else(|| {
                let preview = serde_json::to_string(&value)
                    .unwrap_or_default()
                    .chars()
                    .take(500)
                    .collect::<String>();
                ApiError::ApiResponse(format!(
                    "Missing tracks.items in getList response: {}",
                    preview
                ))
            })?;

        let tracks: Vec<Track> = serde_json::from_value(items.clone())?;
        log::debug!("[API] get_tracks_batch returned {} tracks", tracks.len());
        Ok(tracks)
    }
}
