use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Get discover playlists with optional tag and genre filters
    /// Example: tags=label, genre_ids=112,119
    pub async fn get_discover_playlists(
        &self,
        tag: Option<String>,
        genre_ids: Option<Vec<u64>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<DiscoverPlaylistsResponse> {
        let url = endpoints::build_url(paths::DISCOVER_PLAYLISTS);
        let mut query: Vec<(&str, String)> = vec![];

        // Add tag filter if provided (e.g., "label", "partner")
        if let Some(ref t) = tag {
            query.push(("tags", t.clone()));
        }

        // Add genre_ids as comma-separated list if provided
        if let Some(gids) = genre_ids {
            if !gids.is_empty() {
                let ids_str = gids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                query.push(("genre_ids", ids_str));
            }
        }

        // Add limit (default 20)
        let lim = limit.unwrap_or(20);
        query.push(("limit", lim.to_string()));

        // Add offset (default 0)
        let off = offset.unwrap_or(0);
        query.push(("offset", off.to_string()));

        log::debug!(
            "[API] get_discover_playlists URL: {} query: {:?}",
            url,
            query
        );

        // First get raw JSON to debug structure
        let raw_response: serde_json::Value = self
            .signed_get_auth(&url, "discoverplaylists", &query.iter().map(|(k, v)| (*k, v.clone())).collect::<Vec<_>>())
            .await?
            .json()
            .await?;

        log::debug!(
            "[API] get_discover_playlists raw response keys: {:?}",
            raw_response
                .as_object()
                .map(|o| o.keys().collect::<Vec<_>>())
        );

        // Try to parse as expected structure
        let response: DiscoverPlaylistsResponse = serde_json::from_value(raw_response.clone())
            .map_err(|e| {
                log::error!("[API] Failed to parse discover playlists response: {}", e);
                log::error!(
                    "[API] Raw response: {}",
                    serde_json::to_string_pretty(&raw_response).unwrap_or_default()
                );
                e
            })?;

        log::debug!(
            "[API] get_discover_playlists response: {} playlists",
            response.items.len()
        );

        Ok(response)
    }

    /// Get playlist tags with localized names
    pub async fn get_playlist_tags(&self) -> Result<Vec<PlaylistTag>> {
        let url = endpoints::build_url(paths::PLAYLIST_GET_TAGS);

        let http_response = self
            .signed_get_auth(&url, "playlistgetTags", &[])
            .await?;
        log::info!("[API] get_playlist_tags status={}", http_response.status());

        let raw: PlaylistTagsResponse = http_response.json().await?;

        // Get current locale (e.g., "en", "es", "fr", "de")
        let locale = self.locale().await;
        let lang = locale.split('-').next().unwrap_or("en");

        // Convert raw tags to PlaylistTag with localized name
        let tags: Vec<PlaylistTag> = raw
            .tags
            .into_iter()
            .filter(|tag| tag.is_discover.as_deref() == Some("true"))
            .filter_map(|tag| {
                // Parse name_json to get localized name
                let name_map: serde_json::Value = serde_json::from_str(&tag.name_json).ok()?;
                let name = name_map
                    .get(lang)
                    .or_else(|| name_map.get("en"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())?;
                let id = tag
                    .featured_tag_id
                    .as_ref()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                Some(PlaylistTag {
                    id,
                    slug: tag.slug,
                    name,
                })
            })
            .collect();

        log::debug!("[API] get_playlist_tags: {} tags", tags.len());
        Ok(tags)
    }
}
