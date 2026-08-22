//! Label detail/editorial: awarded releases, curated playlists, top
//! artists, story, and the bulk-hydrate lookup.

use qbz_models::{Album, Artist, FrontendAdapter, LabelGetListResponse, LabelListPage, LabelStoryResponse, Playlist};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get a label's press-awarded releases.
    pub async fn get_label_awarded_releases(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
        sort: Option<String>,
        order: Option<String>,
        genre_ids: Option<String>,
    ) -> Result<LabelListPage<Album>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_awarded_releases(
                label_id,
                limit,
                offset,
                sort.as_deref(),
                order.as_deref(),
                genre_ids.as_deref(),
            )
            .await
            .map_err(CoreError::Api)
    }

    /// Get a label's curated playlists.
    pub async fn get_label_playlists(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<LabelListPage<Playlist>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_playlists(label_id, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get a label's top artists.
    pub async fn get_label_top_artists(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<LabelListPage<Artist>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_top_artists(label_id, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get a label's editorial story.
    pub async fn get_label_story(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<LabelStoryResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_story(label_id, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Bulk hydrate labels by ID list.
    pub async fn get_label_list(
        &self,
        label_ids: Vec<u64>,
    ) -> Result<LabelGetListResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_list(&label_ids)
            .await
            .map_err(CoreError::Api)
    }
}
