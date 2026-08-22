//! Top-level Tidal playlist fetch: token → metadata → track ids → tracks.

use serde_json::Value;

use crate::errors::PlaylistImportError;
use crate::http::http;
use crate::models::{ImportPlaylist, ImportProvider};

use super::auth::get_app_token;
use super::track_ids::fetch_track_ids;
use super::tracks::fetch_tracks_by_ids;
use super::{DEFAULT_COUNTRY_CODE, TIDAL_API_BASE};

/// Fetch a Tidal playlist.
///
/// `country_code` replaces the Tauri original's `TIDAL_COUNTRY_CODE` env read
/// — `None` keeps the same "US" default; callers wanting the env behavior
/// read it at their edge and pass `Some(..)`.
pub async fn fetch_playlist(
    playlist_id: &str,
    country_code: Option<&str>,
) -> Result<ImportPlaylist, PlaylistImportError> {
    let token = get_app_token().await?;
    let country_code = country_code.unwrap_or(DEFAULT_COUNTRY_CODE).to_string();

    let client = http();
    let meta_url = format!("{}/playlists/{}", TIDAL_API_BASE, playlist_id);
    let resp = client
        .get(&meta_url)
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("countryCode", &country_code)])
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
            "Tidal playlist fetch failed: {} - {}",
            status, body
        )));
    }

    let meta: Value = serde_json::from_str(&body)
        .map_err(|e| PlaylistImportError::Parse(format!("Invalid playlist JSON: {}", e)))?;

    let name = meta
        .get("data")
        .and_then(|v| v.get("attributes"))
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Tidal Playlist")
        .to_string();
    let description = meta
        .get("data")
        .and_then(|v| v.get("attributes"))
        .and_then(|v| v.get("description"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .filter(|v| !v.is_empty());

    let track_ids = fetch_track_ids(client, &token, playlist_id, &country_code).await?;
    let tracks = fetch_tracks_by_ids(client, &token, &track_ids, &country_code).await?;

    Ok(ImportPlaylist {
        provider: ImportProvider::Tidal,
        provider_id: playlist_id.to_string(),
        name,
        description,
        tracks,
    })
}
