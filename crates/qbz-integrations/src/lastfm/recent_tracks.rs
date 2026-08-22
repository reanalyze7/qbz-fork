//! `user.getRecentTracks` — timestamped scrobble history.

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::json_helpers::{extract_image, extract_mbid, extract_uts};
use super::models::LastFmTrack;
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// user.getRecentTracks — timestamped scrobble history (max limit 200; paginate via `page`).
    pub async fn get_recent_tracks(
        &self,
        user: &str,
        limit: u32,
        page: u32,
    ) -> IntegrationResult<Vec<LastFmTrack>> {
        // Last.fm caps this endpoint at 200 items per page.
        let limit = limit.min(200);
        let url = format!("{}/user.getRecentTracks", LASTFM_PROXY_URL);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "user": user,
                "limit": limit,
                "page": page,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "Last.fm user.getRecentTracks failed: {}",
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
            .get("recenttracks")
            .and_then(|rt| rt.get("track"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        // Skip the currently-playing track: it has no scrobble timestamp.
                        let now_playing = item
                            .get("@attr")
                            .and_then(|a| a.get("nowplaying"))
                            .and_then(|np| np.as_str())
                            == Some("true");
                        if now_playing {
                            return None;
                        }

                        let name = item.get("name")?.as_str()?.to_string();
                        let mbid = extract_mbid(item);

                        // `artist` may be an object ({"#text"|"name", "mbid"}) or a bare string.
                        let artist_obj = item.get("artist");
                        let artist = artist_obj
                            .and_then(|a| {
                                a.get("#text")
                                    .and_then(|v| v.as_str())
                                    .or_else(|| a.get("name").and_then(|v| v.as_str()))
                                    .or_else(|| a.as_str())
                            })
                            .unwrap_or_default()
                            .to_string();
                        let artist_mbid = artist_obj.and_then(|a| extract_mbid(a));

                        let album = item
                            .get("album")
                            .and_then(|al| al.get("#text"))
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        let uts = extract_uts(item);
                        let image = extract_image(item);

                        Some(LastFmTrack {
                            name,
                            artist,
                            artist_mbid,
                            mbid,
                            album,
                            image,
                            uts,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(tracks)
    }
}
