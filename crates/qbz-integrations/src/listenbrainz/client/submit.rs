//! Scrobble-submission path: now-playing + listen submission.

use super::ListenBrainzClient;
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Submit "now playing" notification
    pub async fn submit_playing_now(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
        additional_info: Option<AdditionalInfo>,
    ) -> IntegrationResult<()> {
        let token = {
            let config = self.config.lock().await;
            if !config.enabled {
                return Ok(()); // Silently skip if disabled
            }
            config.token.clone()
        };

        let token = token.ok_or(IntegrationError::NotAuthenticated)?;

        let info = self.prepare_additional_info(additional_info);

        let payload = SubmitListensPayload {
            listen_type: ListenType::PlayingNow,
            payload: vec![Listen {
                listened_at: None, // Not used for playing_now
                track_metadata: TrackMetadata {
                    artist_name: artist.to_string(),
                    track_name: track.to_string(),
                    release_name: album.map(|s| s.to_string()),
                    additional_info: Some(info),
                },
            }],
        };

        self.submit_listens(&token, &payload).await
    }

    /// Submit a scrobble (track finished playing)
    pub async fn submit_listen(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
        timestamp: i64,
        additional_info: Option<AdditionalInfo>,
    ) -> IntegrationResult<()> {
        let token = {
            let config = self.config.lock().await;
            if !config.enabled {
                return Ok(()); // Silently skip if disabled
            }
            config.token.clone()
        };

        let token = token.ok_or(IntegrationError::NotAuthenticated)?;

        let info = self.prepare_additional_info(additional_info);

        let payload = SubmitListensPayload {
            listen_type: ListenType::Single,
            payload: vec![Listen {
                listened_at: Some(timestamp),
                track_metadata: TrackMetadata {
                    artist_name: artist.to_string(),
                    track_name: track.to_string(),
                    release_name: album.map(|s| s.to_string()),
                    additional_info: Some(info),
                },
            }],
        };

        self.submit_listens(&token, &payload).await
    }
}
