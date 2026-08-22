//! `track.getSimilar` — tracks similar to a seed track.

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::json_helpers::extract_mbid;
use super::models::LastFmSimilarTrack;
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// track.getSimilar — tracks similar to a seed track (raw match weight, NOT 0..1).
    pub async fn get_similar_tracks(
        &self,
        artist: &str,
        track: &str,
        limit: u32,
    ) -> IntegrationResult<Vec<LastFmSimilarTrack>> {
        let url = format!("{}/track.getSimilar", LASTFM_PROXY_URL);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "artist": artist,
                "track": track,
                "limit": limit,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "Last.fm track.getSimilar failed: {}",
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
            .get("similartracks")
            .and_then(|st| st.get("track"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        // Last.fm returns match as a string weight (e.g. "0.83492").
                        let match_score: f64 = item
                            .get("match")
                            .and_then(|m| {
                                m.as_str()
                                    .and_then(|s| s.parse().ok())
                                    .or_else(|| m.as_f64())
                            })
                            .unwrap_or(0.0);
                        let mbid = extract_mbid(item);
                        let artist = item
                            .get("artist")
                            .and_then(|a| a.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or_default()
                            .to_string();

                        Some(LastFmSimilarTrack {
                            name,
                            artist,
                            mbid,
                            match_score,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(tracks)
    }
}
