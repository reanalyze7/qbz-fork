//! Search & Catalog.

use qbz_models::{Album, Artist, FrontendAdapter, SearchAllResults, SearchResultsPage, Track};

use crate::error::CoreError;

use super::{AlbumBlacklistFilter, BlacklistFilter, QbzCore};

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Search for albums
    pub async fn search_albums(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
        search_type: Option<&str>,
    ) -> Result<SearchResultsPage<Album>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .search_albums(query, limit, offset, search_type)
            .await
            .map_err(CoreError::Api)
    }

    /// Search for tracks
    pub async fn search_tracks(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
        search_type: Option<&str>,
    ) -> Result<SearchResultsPage<Track>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .search_tracks(query, limit, offset, search_type)
            .await
            .map_err(CoreError::Api)
    }

    /// Search for artists
    pub async fn search_artists(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
        search_type: Option<&str>,
    ) -> Result<SearchResultsPage<Artist>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .search_artists(query, limit, offset, search_type)
            .await
            .map_err(CoreError::Api)
    }

    /// Catalog search (combined: albums, tracks, artists, playlists, most_popular).
    /// Returns raw JSON for the caller to parse.
    pub async fn catalog_search(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> Result<serde_json::Value, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .catalog_search(query, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Combined search: `catalog_search` plus parsing of the four category
    /// pages and the `most_popular` hero, with blacklist filtering applied.
    /// The blacklist is a parameter so Search does not depend on the
    /// un-migrated `artist_blacklist` module.
    pub async fn search_all(
        &self,
        query: &str,
        blacklist: &BlacklistFilter,
        album_blacklist: &AlbumBlacklistFilter,
    ) -> Result<SearchAllResults, CoreError> {
        let json = self.catalog_search(query, 30, 0).await?;
        Ok(super::parse_search_all(&json, blacklist, album_blacklist))
    }

    /// Get album by ID
    pub async fn get_album(&self, album_id: &str) -> Result<Album, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client.get_album(album_id).await.map_err(CoreError::Api)
    }

    /// Get track by ID
    pub async fn get_track(&self, track_id: u64) -> Result<Track, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client.get_track(track_id).await.map_err(CoreError::Api)
    }

    /// Get artist by ID
    pub async fn get_artist(&self, artist_id: u64) -> Result<Artist, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_artist_basic(artist_id)
            .await
            .map_err(CoreError::Api)
    }
}
