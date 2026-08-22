//! Album-suggest and dynamic-mix suggestion endpoints.

use qbz_models::{FrontendAdapter, Track, TrackToAnalyse};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Albums similar to a seed album (`/album/suggest`).
    pub async fn get_album_suggest(
        &self,
        album_id: &str,
    ) -> Result<qbz_models::AlbumSuggestResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_album_suggest(album_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Dynamic mix suggestions (`/dynamic/suggest`) seeded from
    /// recently-listened track ids. Returns the suggested tracks.
    pub async fn get_dynamic_suggest(
        &self,
        listened_track_ids: &[u64],
        limit: u32,
    ) -> Result<Vec<Track>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_dynamic_suggest(listened_track_ids, limit)
            .await
            .map_err(CoreError::Api)
    }

    /// Dynamic mix suggestions with the `track_to_analysed` payload — the
    /// PRIMARY DailyQ/WeeklyQ path (see `QobuzClient::get_dynamic_suggest_full`).
    pub async fn get_dynamic_suggest_full(
        &self,
        listened_track_ids: &[u64],
        tracks_to_analyse: &[TrackToAnalyse],
        limit: u32,
    ) -> Result<Vec<Track>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_dynamic_suggest_full(listened_track_ids, tracks_to_analyse, limit)
            .await
            .map_err(CoreError::Api)
    }
}
