//! Playlist CRUD + duplicate-checking. See `catalog.rs` for the small
//! catalog lookups (search playlists, tracks batch, genres) used by
//! the playlist editor.

use qbz_models::{FrontendAdapter, Playlist, PlaylistDuplicateResult};

use crate::error::CoreError;

use super::super::{helpers::compute_playlist_duplicates, QbzCore};

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get user playlists
    pub async fn get_user_playlists(&self) -> Result<Vec<Playlist>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client.get_user_playlists().await.map_err(CoreError::Api)
    }

    /// Get playlist by ID
    pub async fn get_playlist(&self, playlist_id: u64) -> Result<Playlist, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_playlist(playlist_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Add tracks to playlist
    pub async fn add_tracks_to_playlist(
        &self,
        playlist_id: u64,
        track_ids: &[u64],
    ) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .add_tracks_to_playlist(playlist_id, track_ids)
            .await
            .map_err(CoreError::Api)
    }

    /// Check how many of `track_ids` are already in the Qobuz playlist
    /// `playlist_id`. Mirrors Tauri's `v2_check_playlist_duplicates`
    /// (commands_v2/playlists.rs): fetch the playlist's existing track ids and
    /// set-intersect with the input. This is Qobuz-tracks-into-Qobuz-playlist
    /// only — callers gate out local / local-playlist targets before
    /// calling (those never duplicate-check, mirroring the Tauri flow).
    pub async fn check_playlist_duplicates(
        &self,
        playlist_id: u64,
        track_ids: &[u64],
    ) -> Result<PlaylistDuplicateResult, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        let playlist = client
            .get_playlist_track_ids(playlist_id)
            .await
            .map_err(CoreError::Api)?;
        Ok(compute_playlist_duplicates(&playlist.track_ids, track_ids))
    }

    /// Remove tracks from playlist
    pub async fn remove_tracks_from_playlist(
        &self,
        playlist_id: u64,
        playlist_track_ids: &[u64],
    ) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .remove_tracks_from_playlist(playlist_id, playlist_track_ids)
            .await
            .map_err(CoreError::Api)
    }

    /// Create a new playlist
    pub async fn create_playlist(
        &self,
        name: &str,
        description: Option<&str>,
        is_public: bool,
    ) -> Result<Playlist, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .create_playlist(name, description, is_public)
            .await
            .map_err(CoreError::Api)
    }

}
