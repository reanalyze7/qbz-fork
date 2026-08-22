use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Get label page (aggregated: top tracks, releases, playlists, artists)
    pub async fn get_label_page(&self, label_id: u64) -> Result<LabelPageData> {
        let url = endpoints::build_url(paths::LABEL_PAGE);

        log::debug!("[API] get_label_page({})", label_id);
        let response: serde_json::Value = self
            .signed_get(&url, "labelpage", &[("label_id", label_id.to_string())])
            .await?
            .json()
            .await?;

        Ok(serde_json::from_value(response)?)
    }

    /// Get label explore (discover more labels)
    pub async fn get_label_explore(&self, limit: u32, offset: u32) -> Result<LabelExploreResponse> {
        let url = endpoints::build_url(paths::LABEL_EXPLORE);

        log::debug!(
            "[API] get_label_explore(limit={}, offset={})",
            limit,
            offset
        );
        let response: serde_json::Value = self
            .signed_get(&url, "labelexplore", &[("limit", limit.to_string()), ("offset", offset.to_string())])
            .await?
            .json()
            .await?;

        Ok(serde_json::from_value(response)?)
    }

    /// Get a label's album catalog (paginated, filterable).
    ///
    /// Replaces the legacy `/label/get?extra=albums` path.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_label_albums(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
        sort: Option<&str>,
        order: Option<&str>,
        genre_ids: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<LabelListPage<Album>> {
        let url = endpoints::build_url(paths::LABEL_GET_ALBUMS);
        let mut params: Vec<(&str, String)> = vec![
            ("label_id", label_id.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some(v) = sort { params.push(("sort", v.to_string())); }
        if let Some(v) = order { params.push(("order", v.to_string())); }
        if let Some(v) = genre_ids { params.push(("genre_ids", v.to_string())); }
        if let Some(v) = from_date { params.push(("from_date", v.to_string())); }
        if let Some(v) = to_date { params.push(("to_date", v.to_string())); }

        log::debug!("[API] get_label_albums({}, limit={}, offset={})", label_id, limit, offset);
        let response: Value = self
            .signed_get(&url, "labelgetalbums", &params)
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Get a label's upcoming releases.
    pub async fn get_label_next_releases(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
        genre_ids: Option<&str>,
    ) -> Result<LabelListPage<Album>> {
        let url = endpoints::build_url(paths::LABEL_GET_NEXT_RELEASES);
        let mut params: Vec<(&str, String)> = vec![
            ("label_id", label_id.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some(v) = genre_ids { params.push(("genre_ids", v.to_string())); }

        log::debug!("[API] get_label_next_releases({})", label_id);
        let response: Value = self
            .signed_get(&url, "labelgetnextreleases", &params)
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }
}
