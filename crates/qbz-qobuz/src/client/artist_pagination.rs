use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Get artist by ID with album pagination
    pub async fn get_artist_with_pagination(
        &self,
        artist_id: u64,
        with_albums: bool,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Artist> {
        self.get_artist_with_pagination_and_locale(artist_id, with_albums, limit, offset, None)
            .await
    }

    /// Get artist by ID with album pagination and optional locale override
    /// Use locale_override to force a specific language (e.g., "en" for genre checking)
    pub async fn get_artist_with_pagination_and_locale(
        &self,
        artist_id: u64,
        with_albums: bool,
        limit: Option<u32>,
        offset: Option<u32>,
        locale_override: Option<&str>,
    ) -> Result<Artist> {
        let url = endpoints::build_url(paths::ARTIST_GET);
        let locale = match locale_override {
            Some(l) => l.to_string(),
            None => self.locale().await,
        };
        let mut query = vec![("artist_id", artist_id.to_string()), ("lang", locale)];
        if with_albums {
            query.push(("extra", "albums".to_string()));
        }
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }
        if let Some(o) = offset {
            query.push(("offset", o.to_string()));
        }

        let http_response = self
            .signed_get(&url, "artistget", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?;
        log::debug!(
            "[API] get_artist({}, albums={}) status={}",
            artist_id,
            with_albums,
            http_response.status()
        );
        let response: Value = http_response.json().await?;

        Ok(serde_json::from_value(response)?)
    }
}
