//! Paginated fetch of a Tidal playlist's track-id relationships.

use std::time::Duration;

use serde_json::Value;
use tokio::time::sleep;

use crate::errors::PlaylistImportError;

use super::{RATE_LIMIT_DELAY_MS, TIDAL_API_BASE};

pub(super) async fn fetch_track_ids(
    client: &reqwest::Client,
    token: &str,
    playlist_id: &str,
    country_code: &str,
) -> Result<Vec<String>, PlaylistImportError> {
    let mut ids = Vec::new();
    let mut next_path = format!("/playlists/{}/relationships/items?limit=100", playlist_id);

    loop {
        let url = format!("{}{}", TIDAL_API_BASE, next_path);
        sleep(Duration::from_millis(RATE_LIMIT_DELAY_MS)).await;

        let mut request = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token));
        if !next_path.contains("countryCode=") {
            request = request.query(&[("countryCode", country_code)]);
        }

        let resp = request
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
                "Tidal track IDs fetch failed: {} - {}",
                status, body
            )));
        }

        let response: Value = serde_json::from_str(&body)
            .map_err(|e| PlaylistImportError::Parse(format!("Invalid track IDs JSON: {}", e)))?;

        if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            for item in data {
                if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                    ids.push(id.to_string());
                }
            }
        }

        let next = response
            .get("links")
            .and_then(|v| v.get("next"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        match next {
            Some(path) if !path.is_empty() => {
                next_path = path;
            }
            _ => break,
        }
    }

    Ok(ids)
}
