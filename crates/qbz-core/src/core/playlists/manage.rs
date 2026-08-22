//! Playlist subscribe/unsubscribe/delete/update — split out of
//! `crud.rs` for line budget.

use qbz_models::{FrontendAdapter, Playlist};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Follow (subscribe to) a Qobuz playlist so it appears in the user's Qobuz
    /// account across every Qobuz client (and in their user-playlists list).
    pub async fn subscribe_playlist(&self, playlist_id: u64) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .subscribe_playlist(playlist_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Unfollow (unsubscribe from) a Qobuz playlist.
    pub async fn unsubscribe_playlist(&self, playlist_id: u64) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .unsubscribe_playlist(playlist_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Delete a playlist
    pub async fn delete_playlist(&self, playlist_id: u64) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .delete_playlist(playlist_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Update a playlist
    pub async fn update_playlist(
        &self,
        playlist_id: u64,
        name: Option<&str>,
        description: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<Playlist, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .update_playlist(playlist_id, name, description, is_public)
            .await
            .map_err(CoreError::Api)
    }
}
