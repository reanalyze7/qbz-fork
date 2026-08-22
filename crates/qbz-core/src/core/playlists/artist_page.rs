//! Artist page (full artist details) + similar artists.

use qbz_models::{Artist, FrontendAdapter, PageArtistResponse, SearchResultsPage};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get artist page (full artist details with albums, tracks, similar)
    pub async fn get_artist_page(
        &self,
        artist_id: u64,
        sort: Option<&str>,
    ) -> Result<PageArtistResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_artist_page(artist_id, sort)
            .await
            .map_err(CoreError::Api)
    }

    /// Get similar artists
    pub async fn get_similar_artists(
        &self,
        artist_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResultsPage<Artist>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_similar_artists(artist_id, limit, offset)
            .await
            .map_err(CoreError::Api)
    }
}
