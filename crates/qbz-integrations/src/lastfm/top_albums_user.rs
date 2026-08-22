//! `user.getTopAlbums` — the user's own scrobbled albums (their playcount).

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::json_helpers::{extract_image, extract_mbid, parse_u64};
use super::models::LastFmAlbum;
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// user.getTopAlbums — the USER's scrobbled albums (their playcount).
    ///
    /// The "already heard" exclusion set for Recommended Albums. Public read
    /// endpoint — no session key needed (the proxy injects the API key).
    /// `period` must be one of: `overall|7day|1month|3month|6month|12month`.
    pub async fn get_user_top_albums(
        &self,
        user: &str,
        period: &str,
        limit: u32,
        page: u32,
    ) -> IntegrationResult<Vec<LastFmAlbum>> {
        let url = format!("{}/user.getTopAlbums", LASTFM_PROXY_URL);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "user": user,
                "period": period,
                "limit": limit,
                "page": page,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "Last.fm user.getTopAlbums failed: {}",
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

        let albums = data
            .get("topalbums")
            .and_then(|ta| ta.get("album"))
            .and_then(|a| a.as_array())
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
                        let playcount = parse_u64(item.get("playcount"));

                        Some(LastFmAlbum {
                            name,
                            artist,
                            artist_mbid,
                            mbid,
                            image,
                            playcount,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(albums)
    }
}
