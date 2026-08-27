//! Private helpers backing the public submission methods in `submit.rs`.

use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Prepare additional info with QBZ identifiers
    pub(super) fn prepare_additional_info(&self, info: Option<AdditionalInfo>) -> AdditionalInfo {
        let mut info = info.unwrap_or_default();
        info.media_player = "Qoqobuz".to_string();
        info.media_player_version = self.version.clone();
        info.submission_client = "Qoqobuz".to_string();
        info.submission_client_version = self.version.clone();
        info
    }

    /// Internal: Submit listens to API
    pub(super) async fn submit_listens(
        &self,
        token: &str,
        payload: &SubmitListensPayload,
    ) -> IntegrationResult<()> {
        let url = format!("{}/submit-listens", LISTENBRAINZ_API_URL);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", token))
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await?;

        if response.status().is_success() {
            let listen_type = match payload.listen_type {
                ListenType::PlayingNow => "now playing",
                ListenType::Single => "scrobble",
            };
            if let Some(listen) = payload.payload.first() {
                log::debug!(
                    "ListenBrainz {}: {} - {}",
                    listen_type,
                    listen.track_metadata.artist_name,
                    listen.track_metadata.track_name
                );
            }
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(IntegrationError::internal(format!(
                "ListenBrainz submission failed: {} - {}",
                status, text
            )))
        }
    }
}
