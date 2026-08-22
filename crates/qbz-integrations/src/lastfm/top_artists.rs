//! `user.getTopArtists` — top artists for a Last.fm user.

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::json_helpers::{extract_image, extract_mbid, parse_u64};
use super::models::LastFmArtist;
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// user.getTopArtists — top artists for the user (taste seed + known-artist set).
    ///
    /// `period` must be one of: `overall|7day|1month|3month|6month|12month`.
    /// Public read endpoint — no session key needed (the proxy injects the API key).
    pub async fn get_top_artists(
        &self,
        user: &str,
        period: &str,
        limit: u32,
    ) -> IntegrationResult<Vec<LastFmArtist>> {
        let url = format!("{}/user.getTopArtists", LASTFM_PROXY_URL);

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
                "Last.fm user.getTopArtists failed: {}",
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
            .get("topartists")
            .and_then(|ta| ta.get("artist"))
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        let mbid = extract_mbid(item);
                        let playcount = parse_u64(item.get("playcount"));
                        let image = extract_image(item);

                        Some(LastFmArtist {
                            name,
                            mbid,
                            playcount,
                            image,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(artists)
    }
}
