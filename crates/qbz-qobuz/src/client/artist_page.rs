use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    // ============ Artist Page Endpoints ============

    /// Get artist page (aggregated: bio, top tracks, releases, similar, playlists)
    pub async fn get_artist_page(
        &self,
        artist_id: u64,
        sort: Option<&str>,
    ) -> Result<PageArtistResponse> {
        let url = endpoints::build_url(paths::ARTIST_PAGE);
        let mut query = vec![("artist_id", artist_id.to_string())];
        if let Some(s) = sort {
            query.push(("sort", s.to_string()));
        }

        log::debug!("[API] get_artist_page({}) sort={:?}", artist_id, sort);
        let response: serde_json::Value = self
            .signed_get(&url, "artistpage", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?
            .json()
            .await?;

        Ok(serde_json::from_value(response)?)
    }

    /// Get artist releases grid (paginated by release_type)
    pub async fn get_releases_grid(
        &self,
        artist_id: u64,
        release_type: &str,
        limit: u32,
        offset: u32,
        sort: Option<&str>,
    ) -> Result<ReleasesGridResponse> {
        let url = endpoints::build_url(paths::ARTIST_RELEASES_GRID);
        let mut query = vec![
            ("artist_id", artist_id.to_string()),
            ("release_type", release_type.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some(s) = sort {
            query.push(("sort", s.to_string()));
        }

        log::debug!(
            "[API] get_releases_grid({}) type={} limit={} offset={}",
            artist_id,
            release_type,
            limit,
            offset
        );
        let response: serde_json::Value = self
            .signed_get(&url, "artistgetReleasesGrid", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?
            .json()
            .await?;

        Ok(serde_json::from_value(response)?)
    }

    /// Get artist Magazine stories (editorial articles about the artist).
    /// REST header-auth only (no per-op signing). Web client calls offset=0 limit=2.
    pub async fn get_artist_story(
        &self,
        artist_id: u64,
        offset: u32,
        limit: u32,
    ) -> Result<ArtistStoryResponse> {
        let url = endpoints::build_url(paths::ARTIST_STORY);
        let query = vec![
            ("artist_id", artist_id.to_string()),
            ("offset", offset.to_string()),
            ("limit", limit.to_string()),
        ];

        log::debug!(
            "[API] get_artist_story({}) offset={} limit={}",
            artist_id,
            offset,
            limit
        );
        let response: serde_json::Value = self
            .signed_get(&url, "artiststory", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?
            .json()
            .await?;

        Ok(serde_json::from_value(response)?)
    }
}
