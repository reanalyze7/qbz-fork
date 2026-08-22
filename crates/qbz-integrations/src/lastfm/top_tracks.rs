//! `user.getTopTracks` — top tracks for a Last.fm user.

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::json_helpers::{extract_image, extract_mbid};
use super::models::LastFmTrack;
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// user.getTopTracks — top tracks for the user (period gives coarse recency).
    ///
    /// `period` must be one of: `overall|7day|1month|3month|6month|12month`.
    pub async fn get_top_tracks(
        &self,
        user: &str,
        period: &str,
        limit: u32,
    ) -> IntegrationResult<Vec<LastFmTrack>> {
        let url = format!("{}/user.getTopTracks", LASTFM_PROXY_URL);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "user": user,
                "period": period,
                "limit": limit,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "Last.fm user.getTopTracks failed: {}",
                text
            )));
        }

        let text = response.text().await?;

        let data: serde_json::Value = serde_json::from_str(&text)?;

        if let Some(error) = data.get("error") {
            let message = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(IntegrationError::api(
                error.as_u64().unwrap_or(0) as u32,
                message.to_string(),
            ));
        }

        let tracks = data
            .get("toptracks")
            .and_then(|tt| tt.get("track"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        let mbid = extract_mbid(item);
                        let artist_obj = item.get("artist");
                        let artist = artist_obj
                            .and_then(|a| a.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let artist_mbid = artist_obj.and_then(|a| extract_mbid(a));
                        let image = extract_image(item);

                        Some(LastFmTrack {
                            name,
                            artist,
                            artist_mbid,
                            mbid,
                            album: None,
                            image,
                            uts: None,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(tracks)
    }
}
