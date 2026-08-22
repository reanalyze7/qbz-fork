use serde_json::Value;

use crate::errors::PlaylistImportError;
use crate::http::http;
use crate::models::{ImportPlaylist, ImportProvider, ImportTrack};

use super::super::html::extract_script;

pub async fn fetch_playlist(playlist_id: &str) -> Result<ImportPlaylist, PlaylistImportError> {
    log::info!(
        "Spotify: fetching playlist {} via embed (API no longer available)",
        playlist_id
    );
    fetch_playlist_from_embed(playlist_id).await
}

async fn fetch_playlist_from_embed(
    playlist_id: &str,
) -> Result<ImportPlaylist, PlaylistImportError> {
    let url = format!("https://open.spotify.com/embed/playlist/{}", playlist_id);
    let html = http()
        .get(&url)
        .send()
        .await
        .map_err(|e| PlaylistImportError::Http(e.to_string()))?
        .text()
        .await
        .map_err(|e| PlaylistImportError::Http(e.to_string()))?;

    let json_text = extract_script(&html, "__NEXT_DATA__").ok_or_else(|| {
        PlaylistImportError::Parse("Spotify embed missing __NEXT_DATA__".to_string())
    })?;

    let data: Value =
        serde_json::from_str(&json_text).map_err(|e| PlaylistImportError::Parse(e.to_string()))?;

    let entity = data
        .get("props")
        .and_then(|v| v.get("pageProps"))
        .and_then(|v| v.get("state"))
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("entity"))
        .ok_or_else(|| PlaylistImportError::Parse("Spotify embed missing entity".to_string()))?;

    let name = entity
        .get("title")
        .or_else(|| entity.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Spotify Playlist")
        .to_string();

    let mut tracks = Vec::new();
    let track_list = entity
        .get("trackList")
        .and_then(|v| v.as_array())
        .ok_or_else(|| PlaylistImportError::Parse("Spotify embed missing trackList".to_string()))?;

    for track in track_list {
        let title = track
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let artist = track
            .get("subtitle")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let duration_ms = track.get("duration").and_then(|v| v.as_u64());
        let uri = track.get("uri").and_then(|v| v.as_str()).unwrap_or("");
        let provider_id = uri
            .split(':')
            .last()
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());
        let provider_url = provider_id
            .as_ref()
            .map(|id| format!("https://open.spotify.com/track/{}", id));

        tracks.push(ImportTrack {
            title,
            artist,
            album: None,
            duration_ms,
            isrc: None,
            provider_id,
            provider_url,
        });
    }

    log::info!(
        "Spotify: embed returned {} tracks for '{}' (embed limit is ~50, no ISRC/album data)",
        tracks.len(),
        name
    );

    Ok(ImportPlaylist {
        provider: ImportProvider::Spotify,
        provider_id: playlist_id.to_string(),
        name,
        description: None,
        tracks,
    })
}
