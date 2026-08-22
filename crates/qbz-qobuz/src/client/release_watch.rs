use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Get Release Watch — new releases from artists, labels or awards the
    /// user follows. Qobuz mobile surfaces this as "Radar de Novedades" /
    /// "Release Watch". Endpoint and signature confirmed in
    /// `qbz-nix-docs/qobuz-api-inferred-openapi-v9.7.0.3.yaml`.
    ///
    /// `release_type` is required and must be one of `artists`, `labels`,
    /// `awards` (matches the three tabs in the mobile UI).
    pub async fn get_release_watch(
        &self,
        release_type: &str,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResultsPage<Album>> {
        let url = endpoints::build_url(paths::FAVORITE_GET_NEW_RELEASES);
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();

        let http_response = self
            .http()?
            .get(&url)
            .headers(self.authenticated_headers().await?)
            .query(&[
                ("type", release_type),
                ("limit", limit_str.as_str()),
                ("offset", offset_str.as_str()),
            ])
            .send()
            .await?;

        let status = http_response.status();
        log::info!(
            "[API] get_release_watch(type={}, limit={}, offset={}) status={}",
            release_type,
            limit,
            offset,
            status
        );

        let body_text = http_response.text().await?;
        if !status.is_success() {
            log::warn!(
                "[API] get_release_watch non-success body (first 400 chars): {}",
                body_text.chars().take(400).collect::<String>()
            );
            return Err(ApiError::ApiResponse(format!(
                "get_release_watch status {}",
                status
            )));
        }

        let response: Value = serde_json::from_str(&body_text)?;

        // Shape is V2GenericListDto<AlbumDto> per x20/b.java — a thin
        // pagination envelope: {has_more: bool, items: [...]}. No total/
        // offset/limit fields are returned. We only need items for the UI
        // and the caller already knows the offset/limit it asked for, so
        // we project onto SearchResultsPage<Album> to stay compatible with
        // the rest of the album-list plumbing.
        //
        // Note on shape: AlbumDto carries both `artist` (singular, legacy)
        // and `artists` (array, current). `/favorite/getNewReleases` tends
        // to omit `artist` and only populate `artists[]`, which made our
        // `Album` struct deserialize an empty `artist` and the UI show
        // "Unknown Artist". Backfill `artist` from `artists[0]` before
        // parsing so the existing deserializer just works.
        let mut items_value = response.get("items").cloned().unwrap_or(Value::Null);
        if let Some(items_arr) = items_value.as_array_mut() {
            for item in items_arr {
                if let Some(obj) = item.as_object_mut() {
                    let needs_backfill = obj
                        .get("artist")
                        .map(|v| v.is_null())
                        .unwrap_or(true);
                    if needs_backfill {
                        if let Some(first_artist) = obj
                            .get("artists")
                            .and_then(|a| a.as_array())
                            .and_then(|arr| arr.first())
                            .cloned()
                        {
                            obj.insert("artist".to_string(), first_artist);
                        }
                    }
                }
            }
        }
        let items: Vec<Album> = serde_json::from_value(items_value).map_err(|e| {
            log::warn!(
                "[API] get_release_watch items parse error: {}. Body (first 600 chars): {}",
                e,
                body_text.chars().take(600).collect::<String>()
            );
            ApiError::ApiResponse(format!("get_release_watch items parse: {}", e))
        })?;

        let has_more = response
            .get("has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let total_hint = if has_more {
            // We don't know the real total; signal "at least one more page"
            // by bumping past the items we have.
            offset + items.len() as u32 + 1
        } else {
            offset + items.len() as u32
        };

        Ok(SearchResultsPage::<Album> {
            items,
            total: total_hint,
            offset,
            limit,
        })
    }
}
