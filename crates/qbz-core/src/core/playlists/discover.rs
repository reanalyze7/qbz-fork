//! Discover browse: index/playlists/tags/albums/featured/release-watch,
//! plus artist-page + similar-artists. See `artist_extra.rs` for the
//! remaining artist-detail endpoints (kept separate to stay under the
//! per-file line budget).

use qbz_models::{
    Album, DiscoverAlbum, DiscoverData, DiscoverPlaylistsResponse, DiscoverResponse,
    FrontendAdapter, PlaylistTag, SearchResultsPage,
};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get discover index
    pub async fn get_discover_index(
        &self,
        genre_ids: Option<Vec<u64>>,
    ) -> Result<DiscoverResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_discover_index(genre_ids)
            .await
            .map_err(CoreError::Api)
    }

    /// Get discover playlists
    pub async fn get_discover_playlists(
        &self,
        tag: Option<String>,
        genre_ids: Option<Vec<u64>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<DiscoverPlaylistsResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_discover_playlists(tag, genre_ids, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get playlist tags
    pub async fn get_playlist_tags(&self) -> Result<Vec<PlaylistTag>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client.get_playlist_tags().await.map_err(CoreError::Api)
    }

    /// Get discover albums from a specific browse endpoint
    pub async fn get_discover_albums(
        &self,
        endpoint: &str,
        genre_ids: Option<Vec<u64>>,
        offset: u32,
        limit: u32,
    ) -> Result<DiscoverData<DiscoverAlbum>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_discover_albums(endpoint, genre_ids, offset, limit)
            .await
            .map_err(CoreError::Api)
    }

    /// Get featured albums
    pub async fn get_featured_albums(
        &self,
        featured_type: &str,
        limit: u32,
        offset: u32,
        genre_id: Option<u64>,
    ) -> Result<SearchResultsPage<Album>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_featured_albums(featured_type, limit, offset, genre_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Get Release Watch — new releases from followed artists/labels/awards.
    /// `release_type` must be one of "artists" | "labels" | "awards".
    pub async fn get_release_watch(
        &self,
        release_type: &str,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResultsPage<Album>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_release_watch(release_type, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

}
