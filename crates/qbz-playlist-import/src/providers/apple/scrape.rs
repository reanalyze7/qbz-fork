//! Apple Music playlist-page scraping — network I/O + assembly.

use serde_json::Value;

use crate::errors::PlaylistImportError;
use crate::http::http;
use crate::models::{ImportPlaylist, ImportProvider, ImportTrack};

use super::html::{extract_meta, extract_script};
use super::json::find_track_items;

pub async fn fetch_playlist(
    storefront: &str,
    playlist_id: &str,
) -> Result<ImportPlaylist, PlaylistImportError> {
    let url = format!(
        "https://music.apple.com/{}/playlist/{}",
        storefront, playlist_id
    );
    let html = http()
        .get(&url)
        .send()
        .await
        .map_err(|e| PlaylistImportError::Http(e.to_string()))?
        .text()
        .await
        .map_err(|e| PlaylistImportError::Http(e.to_string()))?;

    let name =
        extract_meta(&html, "og:title").unwrap_or_else(|| "Apple Music Playlist".to_string());
    let description = extract_meta(&html, "og:description").filter(|v| !v.is_empty());

    let json_text = extract_script(&html, "serialized-server-data").ok_or_else(|| {
        PlaylistImportError::Parse("Apple Music serialized-server-data not found".to_string())
    })?;

    let data: Value =
        serde_json::from_str(&json_text).map_err(|e| PlaylistImportError::Parse(e.to_string()))?;

    let items = find_track_items(&data).ok_or_else(|| {
        PlaylistImportError::Parse("Apple Music track list not found".to_string())
    })?;

    let mut tracks = Vec::new();
    for item in items {
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let artist = item
            .get("artistName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let duration_ms = item.get("duration").and_then(|v| v.as_u64());
        let provider_id = item
            .get("contentDescriptor")
            .and_then(|v| v.get("identifiers"))
            .and_then(|v| v.get("storeAdamID"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let provider_url = item
            .get("contentDescriptor")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let album = item
            .get("tertiaryLinks")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("title"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        tracks.push(ImportTrack {
            title,
            artist,
            album,
            duration_ms,
            isrc: None,
            provider_id,
            provider_url,
        });
    }

    Ok(ImportPlaylist {
        provider: ImportProvider::AppleMusic,
        provider_id: playlist_id.to_string(),
        name,
        description,
        tracks,
    })
}
