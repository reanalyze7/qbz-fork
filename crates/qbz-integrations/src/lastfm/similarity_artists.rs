//! `artist.getSimilar` — genre-accurate artist similarity.

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::models::LastFmSimilarArtist;
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// Get similar artists for a given artist name
    ///
    /// Uses Last.fm's artist.getSimilar which returns genre-accurate similarity.
    /// Requires authentication (user must have Last.fm connected).
    pub async fn get_similar_artists(
        &self,
        artist: &str,
        limit: u32,
    ) -> IntegrationResult<Vec<LastFmSimilarArtist>> {
        // artist.getSimilar is a public read endpoint - no session key needed.
        // The proxy handles the API key.
        let url = format!("{}/artist.getSimilar", LASTFM_PROXY_URL);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "artist": artist,
                "limit": limit,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "Last.fm artist.getSimilar failed: {}",
                text
            )));
        }

        let text = response.text().await?;

        let data: serde_json::Value = serde_json::from_str(&text)?;

        // Handle Last.fm error responses
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

        let artists = data
            .get("similarartists")
            .and_then(|sa| sa.get("artist"))
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        // Last.fm returns match as string "0" to "1"
                        let match_score: f64 = item
                            .get("match")
                            .and_then(|m| {
                                m.as_str()
                                    .and_then(|s| s.parse().ok())
                                    .or_else(|| m.as_f64())
                            })
                            .unwrap_or(0.0);
                        let mbid = item
                            .get("mbid")
                            .and_then(|m| m.as_str())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        Some(LastFmSimilarArtist {
                            name,
                            match_score,
                            mbid,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(artists)
    }
}
