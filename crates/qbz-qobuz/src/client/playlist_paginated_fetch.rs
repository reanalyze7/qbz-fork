use serde_json::Value;

use super::QobuzClient;
use crate::auth::{get_timestamp, sign_request};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Fetch all remaining pages of a playlist concurrently, given the first
    /// page's `url`, the already-fetched count, and the total track count.
    /// Returns the tracks from each page, sorted by offset (best-effort: a
    /// failed page is logged and skipped rather than aborting the whole
    /// fetch).
    pub(super) async fn fetch_remaining_playlist_pages(
        &self,
        url: &str,
        playlist_id: u64,
        page_size: u32,
        fetched: u32,
        total: u32,
    ) -> Result<Vec<Track>> {
        // Build all remaining page offsets
        let offsets: Vec<u32> = (fetched..total).step_by(page_size as usize).collect();
        log::debug!(
            "[API] get_playlist({}) fetching {} remaining pages concurrently ({}/{})",
            playlist_id,
            offsets.len(),
            fetched,
            total
        );

        // Prepare headers and per-page signatures for concurrent requests
        let headers = self.api_headers().await?;
        let secret = self.secret().await.unwrap_or_default();

        // Offline gate checked ONCE for the whole page batch (the first page
        // already passed through it) — not inside the per-page loop.
        let gated_http = self.http()?;

        // Launch all page requests concurrently
        let futures: Vec<_> = offsets
            .iter()
            .map(|&offset| {
                let http = gated_http;
                let headers = headers.clone();
                let pid = playlist_id.to_string();
                let limit = page_size.to_string();
                let offset_str = offset.to_string();
                let ts = get_timestamp();
                let sig = sign_request(
                    "playlistget",
                    &[("extra", "tracks"), ("limit", &limit), ("offset", &offset_str), ("playlist_id", &pid)],
                    ts,
                    &secret,
                );
                let ts_str = ts.to_string();
                async move {
                    let resp = http
                        .get(url)
                        .headers(headers)
                        .query(&[
                            ("playlist_id", pid.as_str()),
                            ("limit", limit.as_str()),
                            ("offset", offset_str.as_str()),
                            ("extra", "tracks"),
                            ("request_ts", ts_str.as_str()),
                            ("request_sig", sig.as_str()),
                        ])
                        .send()
                        .await?;
                    let value: Value = resp.json().await?;
                    let page: Playlist = serde_json::from_value(value)?;
                    Ok::<_, anyhow::Error>((offset, page))
                }
            })
            .collect();

        let results = futures_util::future::join_all(futures).await;

        // Collect results sorted by offset to maintain track order
        let mut pages: Vec<(u32, Playlist)> = Vec::new();
        for result in results {
            match result {
                Ok(page) => pages.push(page),
                Err(e) => {
                    log::warn!(
                        "[API] get_playlist({}) page fetch failed: {}",
                        playlist_id,
                        e
                    );
                    // Continue with what we have
                }
            }
        }
        pages.sort_by_key(|(offset, _)| *offset);

        // Flatten in order
        let mut tracks = Vec::new();
        for (_, page_playlist) in pages {
            if let Some(page_tracks) = page_playlist.tracks {
                if !page_tracks.items.is_empty() {
                    tracks.extend(page_tracks.items);
                }
            }
        }
        Ok(tracks)
    }
}
