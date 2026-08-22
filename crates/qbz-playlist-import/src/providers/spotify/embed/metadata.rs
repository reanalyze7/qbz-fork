use serde_json::Value;

use crate::http::http;

use super::super::html::extract_script;

/// Fetch track or album metadata from Spotify embed page.
/// Returns (title, artist) if successful.
pub async fn fetch_embed_metadata(entity_type: &str, entity_id: &str) -> Option<(String, String)> {
    let url = format!(
        "https://open.spotify.com/embed/{}/{}",
        entity_type, entity_id
    );
    let html = http().get(&url).send().await.ok()?.text().await.ok()?;
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
