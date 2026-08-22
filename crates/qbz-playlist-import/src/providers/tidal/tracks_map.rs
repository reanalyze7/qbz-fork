//! Builds the `included` artist/album lookup maps and extracts one
//! `ImportTrack` from a Tidal track JSON item.

use std::collections::HashMap;

use serde_json::Value;

use crate::models::ImportTrack;

use super::duration::parse_duration_ms;

pub(super) fn build_included_maps(
    included: &[Value],
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut artist_map = HashMap::new();
    let mut album_map = HashMap::new();

    for item in included {
        if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() {
                continue;
            }

            match item_type {
                "artists" => {
                    if let Some(name) = item
                        .get("attributes")
                        .and_then(|v| v.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        artist_map.insert(id.to_string(), name.to_string());
                    }
                }
                "albums" => {
                    if let Some(name) = item
                        .get("attributes")
                        .and_then(|v| v.get("title"))
                        .and_then(|v| v.as_str())
                    {
                        album_map.insert(id.to_string(), name.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    (artist_map, album_map)
}

pub(super) fn track_from_json(
    item: &Value,
    artist_map: &HashMap<String, String>,
    album_map: &HashMap<String, String>,
) -> ImportTrack {
    let title = item
        .get("attributes")
        .and_then(|v| v.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let isrc = item
        .get("attributes")
        .and_then(|v| v.get("isrc"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let duration_ms = item
        .get("attributes")
        .and_then(|v| v.get("duration"))
        .and_then(|v| v.as_str())
        .and_then(parse_duration_ms);

    let artist = item
        .get("relationships")
        .and_then(|v| v.get("artists"))
        .and_then(|v| v.get("data"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .and_then(|id| artist_map.get(id))
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    let album = item
        .get("relationships")
        .and_then(|v| v.get("albums"))
        .and_then(|v| v.get("data"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .and_then(|id| album_map.get(id))
        .cloned();

    let provider_id = item
        .get("id")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    ImportTrack {
        title,
        artist,
        album,
        duration_ms,
        isrc,
        provider_id,
        provider_url: None,
    }
}
