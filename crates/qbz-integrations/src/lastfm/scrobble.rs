//! Last.fm scrobbling and now-playing updates (write endpoints, require auth).

use serde_json::json;

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// Scrobble a track (mark as played)
    ///
    /// Requires authentication.
    pub async fn scrobble(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
        timestamp: u64,
    ) -> IntegrationResult<()> {
        let session_key = self
            .session_key
            .as_ref()
            .ok_or(IntegrationError::NotAuthenticated)?;

        let url = format!("{}/track.scrobble", LASTFM_PROXY_URL);

        let mut body = json!({
            "sk": session_key,
            "artist": artist,
            "track": track,
            "timestamp": timestamp.to_string(),
        });

        if let Some(album_name) = album {
            body["album"] = json!(album_name);
        }

        let response = self.client.post(&url).json(&body).send().await?;

        if response.status().is_success() {
            log::info!("Scrobbled: {} - {}", artist, track);
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(IntegrationError::internal(format!(
                "Scrobble failed: {}",
                text
            )))
        }
    }

    /// Update "now playing" status
    ///
    /// Requires authentication.
    pub async fn update_now_playing(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
    ) -> IntegrationResult<()> {
        let session_key = self
            .session_key
            .as_ref()
            .ok_or(IntegrationError::NotAuthenticated)?;

        let url = format!("{}/track.updateNowPlaying", LASTFM_PROXY_URL);

        let mut body = json!({
            "sk": session_key,
            "artist": artist,
            "track": track,
        });

        if let Some(album_name) = album {
            body["album"] = json!(album_name);
        }

        let response = self.client.post(&url).json(&body).send().await?;

        if response.status().is_success() {
            log::debug!("Updated now playing: {} - {}", artist, track);
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(IntegrationError::internal(format!(
                "Update now playing failed: {}",
                text
            )))
        }
    }
}
