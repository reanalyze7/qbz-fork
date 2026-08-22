//! Artist-detail endpoints (paginated albums/tracks/releases/story).
//! See `suggest.rs` for album/dynamic suggestion endpoints.

use qbz_models::{Artist, ArtistAlbums, ArtistStoryResponse, FrontendAdapter, ReleasesGridResponse, TracksContainer};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get artist with albums (for album pagination)
    pub async fn get_artist_with_albums(
        &self,
        artist_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Artist, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_artist_with_pagination(artist_id, true, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get an artist's albums collection (paginated `ArtistAlbums` only).
    ///
    /// Equivalent to `get_artist_with_albums` but projects only the `albums`
    /// field for callers that don't need the full artist envelope.
    pub async fn get_artist_albums(
        &self,
        artist_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<ArtistAlbums, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        let artist = client
            .get_artist_with_pagination(artist_id, true, limit, offset)
            .await
            .map_err(CoreError::Api)?;

        artist
            .albums
            .ok_or_else(|| CoreError::Api(qbz_qobuz::ApiError::ApiResponse(
                "No albums in artist response".to_string(),
            )))
    }

    /// Get artist detail with albums, playlists and appears-on tracks.
    ///
    /// Backs the suggestions panel: requests `extra=albums,tracks_appears_on,playlists`
    /// from `/artist/get` so callers can read `playlists` and `tracks_appears_on`
    /// without a second round-trip.
    pub async fn get_artist_detail(
        &self,
        artist_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Artist, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_artist_detail(artist_id, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get an artist's popular/top tracks (`/artist/get?extra=tracks`).
    pub async fn get_artist_tracks(
        &self,
        artist_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<TracksContainer, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_artist_tracks(artist_id, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get an artist's releases grid (paginated by `release_type`).
    pub async fn get_releases_grid(
        &self,
        artist_id: u64,
        release_type: &str,
        limit: u32,
        offset: u32,
        sort: Option<&str>,
    ) -> Result<ReleasesGridResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_releases_grid(artist_id, release_type, limit, offset, sort)
            .await
            .map_err(CoreError::Api)
    }

    /// Get an artist's Magazine stories (editorial articles). Web client: offset=0 limit=2.
    pub async fn get_artist_story(
        &self,
        artist_id: u64,
        offset: u32,
        limit: u32,
    ) -> Result<ArtistStoryResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_artist_story(artist_id, offset, limit)
            .await
            .map_err(CoreError::Api)
    }
}
