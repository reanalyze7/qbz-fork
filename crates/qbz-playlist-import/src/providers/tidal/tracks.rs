//! Chunked track-detail fetch for a list of Tidal track ids.

use std::time::Duration;

use serde_json::Value;
use tokio::time::sleep;

use crate::errors::PlaylistImportError;
use crate::models::ImportTrack;

use super::tracks_map::{build_included_maps, track_from_json};
use super::{RATE_LIMIT_DELAY_MS, TIDAL_API_BASE};

pub(super) async fn fetch_tracks_by_ids(
    client: &reqwest::Client,
    token: &str,
    track_ids: &[String],
    country_code: &str,
) -> Result<Vec<ImportTrack>, PlaylistImportError> {
    let mut tracks = Vec::new();
    let mut chunk_start = 0usize;
    let chunk_size = 20usize; // Tidal API limit

    while chunk_start < track_ids.len() {
        let end = (chunk_start + chunk_size).min(track_ids.len());
        let chunk = &track_ids[chunk_start..end];

        let url = format!("{}/tracks", TIDAL_API_BASE);
        sleep(Duration::from_millis(RATE_LIMIT_DELAY_MS)).await;

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("filter[id]", chunk.join(",")),
                ("include", "artists,albums".to_string()),
                ("countryCode", country_code.to_string()),
            ])
            .send()
            .await
            .map_err(|e| PlaylistImportError::Http(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| PlaylistImportError::Parse(e.to_string()))?;

        if !status.is_success() {
            return Err(PlaylistImportError::Http(format!(
                "Tidal tracks fetch failed: {} - {}",
                status, body
            )));
        }

        let response: Value = serde_json::from_str(&body)
            .map_err(|e| PlaylistImportError::Parse(format!("Invalid tracks JSON: {}", e)))?;

        let included = response
            .get("included")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let (artist_map, album_map) = build_included_maps(&included);

        if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            for item in data {
                tracks.push(track_from_json(item, &artist_map, &album_map));
            }
        }

        chunk_start = end;
    }

    Ok(tracks)
}
