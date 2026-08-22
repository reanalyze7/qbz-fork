use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Dynamic suggestions for mixes (`POST /dynamic/suggest`). Seeds
    /// from recently-listened track ids. Returns the suggested tracks
    /// (items parsed leniently). Ported from the legacy api client;
    /// the `track_to_analysed` payload is optional and omitted here.
    pub async fn get_dynamic_suggest(
        &self,
        listened_track_ids: &[u64],
        limit: u32,
    ) -> Result<Vec<Track>> {
        self.get_dynamic_suggest_full(listened_track_ids, &[], limit)
            .await
    }

    /// Like [`get_dynamic_suggest`] but carrying the `track_to_analysed`
    /// payload — the PRIMARY DailyQ/WeeklyQ path. Tauri seeds this with up to 9
    /// resolved `{track_id, artist_id, genre_id, label_id}` tuples and only
    /// falls back to an empty analysis when a call returns zero items.
    pub async fn get_dynamic_suggest_full(
        &self,
        listened_track_ids: &[u64],
        tracks_to_analyse: &[TrackToAnalyse],
        limit: u32,
    ) -> Result<Vec<Track>> {
        let url = endpoints::build_url(paths::DYNAMIC_SUGGEST);
        let body = serde_json::json!({
            "limit": limit,
            "listened_tracks_ids": listened_track_ids,
            "track_to_analysed": tracks_to_analyse,
        });
        let http_response = self
            .http()?
            .post(&url)
            .headers(self.authenticated_headers().await?)
            .json(&body)
            .send()
            .await?;
        let status = http_response.status();
        if !status.is_success() {
            return Err(ApiError::ApiResponse(format!(
                "get_dynamic_suggest status {status}"
            )));
        }
        let response: Value = http_response.json().await?;
        Ok(qbz_models::lenient::parse_items_array(
            &response,
            "tracks",
            "dynamic-suggest track",
        ))
    }
}
