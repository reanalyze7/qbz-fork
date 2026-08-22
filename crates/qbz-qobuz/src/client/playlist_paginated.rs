use serde_json::Value;

use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Get playlist by ID (paginates automatically to fetch all tracks)
    ///
    /// After the first page, remaining pages are fetched concurrently
    /// since we know the total track count from the first response.
    pub async fn get_playlist(&self, playlist_id: u64) -> Result<Playlist> {
        let url = endpoints::build_url(paths::PLAYLIST_GET);
        const PAGE_SIZE: u32 = 500;

        let start = std::time::Instant::now();

        // First page — gives us metadata + total track count
        let http_response = self
            .signed_get(&url, "playlistget", &[
                ("playlist_id", playlist_id.to_string()),
                ("limit", PAGE_SIZE.to_string()),
                ("offset", "0".to_string()),
                ("extra", "tracks".to_string()),
            ])
            .await?;
        log::debug!(
            "[API] get_playlist({}) status={}",
            playlist_id,
            http_response.status()
        );
        let response: Value = http_response.json().await?;
        let mut playlist: Playlist = serde_json::from_value(response)?;

        // Fetch remaining pages concurrently
        if let Some(ref mut container) = playlist.tracks {
            let total = container.total;
            let fetched = container.items.len() as u32;

            if fetched < total {
                let more_tracks = self
                    .fetch_remaining_playlist_pages(&url, playlist_id, PAGE_SIZE, fetched, total)
                    .await?;
                container.items.extend(more_tracks);
            }
        }

        let elapsed = start.elapsed();
        log::debug!(
            "[API] get_playlist({}) complete: {} tracks in {:.2}s",
            playlist_id,
            playlist.tracks.as_ref().map(|t| t.items.len()).unwrap_or(0),
            elapsed.as_secs_f64()
        );

        Ok(playlist)
    }
}
