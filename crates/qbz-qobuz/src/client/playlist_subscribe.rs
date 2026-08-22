use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};

impl QobuzClient {
    /// Subscribe to a Qobuz playlist (follow it in the user's library)
    pub async fn subscribe_playlist(&self, playlist_id: u64) -> Result<()> {
        let url = endpoints::build_url(paths::PLAYLIST_SUBSCRIBE);

        let response = self
            .signed_get_auth(&url, "playlistsubscribe", &[("playlist_id", playlist_id.to_string())])
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiResponse(format!(
                "playlist/subscribe failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    /// Unsubscribe from a Qobuz playlist
    pub async fn unsubscribe_playlist(&self, playlist_id: u64) -> Result<()> {
        let url = endpoints::build_url(paths::PLAYLIST_UNSUBSCRIBE);

        let response = self
            .signed_get_auth(&url, "playlistunsubscribe", &[("playlist_id", playlist_id.to_string())])
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiResponse(format!(
                "playlist/unsubscribe failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }
}
