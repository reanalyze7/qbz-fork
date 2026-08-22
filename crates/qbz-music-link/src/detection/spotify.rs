//! Spotify URL detection and embed-metadata scrape.

use super::{MusicProvider, MusicResource};
use serde_json::Value;

/// Detect if a URL is a Spotify track, album, or playlist.
pub fn detect_resource(url: &str) -> Option<MusicResource> {
    let lower = url.to_ascii_lowercase();
    if !lower.contains("spotify.com/") && !lower.starts_with("spotify:") {
        return None;
    }

    // Playlist check first (so parse_playlist_id takes priority for playlists)
    if parse_playlist_id(url).is_some() {
        return Some(MusicResource::Playlist {
            provider: MusicProvider::Spotify,
        });
    }

    // Track: open.spotify.com/track/<id> or spotify:track:<id>
    if lower.contains("/track/") || lower.contains(":track:") {
        return Some(MusicResource::Track {
            provider: MusicProvider::Spotify,
            url: url.to_string(),
        });
    }

    // Album: open.spotify.com/album/<id> or spotify:album:<id>
    if lower.contains("/album/") || lower.contains(":album:") {
        return Some(MusicResource::Album {
            provider: MusicProvider::Spotify,
            url: url.to_string(),
        });
    }

    None
}

pub fn parse_playlist_id(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix("spotify:playlist:") {
        if !rest.is_empty() {
            return Some(rest.to_string());
        }
    }

    let patterns = [
        "open.spotify.com/playlist/",
        "open.spotify.com/embed/playlist/",
    ];
    for pattern in patterns {
        if let Some(idx) = url.find(pattern) {
            let mut part = &url[idx + pattern.len()..];
            if let Some(end) = part.find('?') {
                part = &part[..end];
            }
            if !part.is_empty() {
                return Some(part.to_string());
            }
        }
    }

    None
}

/// Fetch track or album metadata from Spotify embed page.
/// Returns (title, artist) if successful.
pub async fn fetch_embed_metadata(entity_type: &str, entity_id: &str) -> Option<(String, String)> {
    let url = format!(
        "https://open.spotify.com/embed/{}/{}",
        entity_type, entity_id
    );
    let html = reqwest::get(&url).await.ok()?.text().await.ok()?;
    let json_text = extract_script(&html, "__NEXT_DATA__")?;
    let data: Value = serde_json::from_str(&json_text).ok()?;

    let entity = data
        .get("props")?
        .get("pageProps")?
        .get("state")?
        .get("data")?
        .get("entity")?;

    let title = entity
        .get("title")
        .or_else(|| entity.get("name"))
        .and_then(|v| v.as_str())?
        .to_string();

    // Tracks have "artists" array, albums have "subtitle" string
    let artist = entity
        .get("artists")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("name").and_then(|v| v.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .or_else(|| {
            entity
                .get("subtitle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    if title.is_empty() {
        return None;
    }

    Some((title, artist))
}

fn extract_script(html: &str, id: &str) -> Option<String> {
    let marker = format!("id=\"{}\"", id);
    let start = html.find(&marker)?;
    let script_start = html[start..].find('>')? + start + 1;
    let script_end = html[script_start..].find("</script>")? + script_start;
    Some(html[script_start..script_end].to_string())
}
