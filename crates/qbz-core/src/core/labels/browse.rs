//! Label browse/catalog: page, explore, album catalog, upcoming
//! releases.

use qbz_models::{Album, FrontendAdapter, LabelExploreResponse, LabelListPage, LabelPageData};

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get label page (aggregated: top tracks, releases, playlists, artists)
    pub async fn get_label_page(&self, label_id: u64) -> Result<LabelPageData, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_label_page(label_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Get label explore (discover more labels)
    pub async fn get_label_explore(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<LabelExploreResponse, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_label_explore(limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Get a label's album catalog (paginated, replaces legacy /label/get).
    #[allow(clippy::too_many_arguments)]
    pub async fn get_label_albums(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
        sort: Option<String>,
        order: Option<String>,
        genre_ids: Option<String>,
        from_date: Option<String>,
        to_date: Option<String>,
    ) -> Result<LabelListPage<Album>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_albums(
                label_id,
                limit,
                offset,
                sort.as_deref(),
                order.as_deref(),
                genre_ids.as_deref(),
                from_date.as_deref(),
                to_date.as_deref(),
            )
            .await
            .map_err(CoreError::Api)
    }

    /// Get a label's upcoming releases.
    pub async fn get_label_next_releases(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
        genre_ids: Option<String>,
    ) -> Result<LabelListPage<Album>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client
            .get_label_next_releases(label_id, limit, offset, genre_ids.as_deref())
            .await
            .map_err(CoreError::Api)
    }
}
