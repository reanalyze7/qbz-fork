use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Get a label's press-awarded releases.
    pub async fn get_label_awarded_releases(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
        sort: Option<&str>,
        order: Option<&str>,
        genre_ids: Option<&str>,
    ) -> Result<LabelListPage<Album>> {
        let url = endpoints::build_url(paths::LABEL_GET_AWARDED_RELEASES);
        let mut params: Vec<(&str, String)> = vec![
            ("label_id", label_id.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some(v) = sort { params.push(("sort", v.to_string())); }
        if let Some(v) = order { params.push(("order", v.to_string())); }
        if let Some(v) = genre_ids { params.push(("genre_ids", v.to_string())); }

        log::debug!("[API] get_label_awarded_releases({})", label_id);
        let response: Value = self
            .signed_get(&url, "labelgetawardedreleases", &params)
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Get a label's curated playlists (paginated).
    pub async fn get_label_playlists(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<LabelListPage<Playlist>> {
        let url = endpoints::build_url(paths::LABEL_GET_PLAYLISTS);
        log::debug!("[API] get_label_playlists({})", label_id);
        let response: Value = self
            .signed_get(&url, "labelgetplaylists", &[
                ("label_id", label_id.to_string()),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Get a label's top artists (paginated).
    pub async fn get_label_top_artists(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<LabelListPage<Artist>> {
        let url = endpoints::build_url(paths::LABEL_GET_TOP_ARTISTS);
        log::debug!("[API] get_label_top_artists({})", label_id);
        let response: Value = self
            .signed_get(&url, "labelgettopartists", &[
                ("label_id", label_id.to_string()),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Get a label's editorial / story content.
    pub async fn get_label_story(
        &self,
        label_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<LabelStoryResponse> {
        let url = endpoints::build_url(paths::LABEL_STORY);
        log::debug!("[API] get_label_story({})", label_id);
        let response: Value = self
            .signed_get(&url, "labelstory", &[
                ("label_id", label_id.to_string()),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }

}
