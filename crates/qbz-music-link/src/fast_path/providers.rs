//! Per-platform direct metadata fetchers (I/O).

use super::entity_id::{extract_entity_id, extract_spotify_entity_id};
use super::QBZ_PROXY_BASE;
use crate::detection::spotify;

pub(super) async fn try_deezer_metadata(url: &str, is_track: bool) -> Option<(String, String)> {
    let entity = if is_track { "track" } else { "album" };
    let id = extract_entity_id(url, entity).or_else(|| {
        if is_track {
            None
        } else {
            extract_entity_id(url, "track")
        }
    })?;
    let api_url = format!("https://api.deezer.com/{}/{}", entity, id);

    log::debug!("Link resolver: Deezer direct API: {}", api_url);
    let data: serde_json::Value = reqwest::get(&api_url).await.ok()?.json().await.ok()?;
    if data.get("error").is_some() {
        return None;
    }

    let title = data.get("title")?.as_str()?.to_string();
    let artist = data
        .get("artist")
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some((title, artist))
}

pub(super) async fn try_spotify_metadata(
    url: &str,
    is_track: bool,
) -> Option<(String, String)> {
    let entity = if is_track { "track" } else { "album" };
    let id = extract_spotify_entity_id(url, entity)?;

    log::debug!("Link resolver: Spotify embed scrape for {} {}", entity, id);
    spotify::fetch_embed_metadata(entity, &id).await
}

pub(super) async fn try_tidal_metadata(url: &str, is_track: bool) -> Option<(String, String)> {
    let entity = if is_track { "track" } else { "album" };
    let id = extract_entity_id(url, entity)
        // Also try /browse/track/ pattern
        .or_else(|| extract_entity_id(url, &format!("browse/{}", entity)))?;
    let token = get_proxy_token("tidal").await?;
    let api_url = format!(
        "https://openapi.tidal.com/v2/{}s/{}?countryCode=US&include=artists",
        entity, id
    );

    log::debug!("Link resolver: Tidal direct API: {}", api_url);
    let data: serde_json::Value = reqwest::Client::new()
        .get(&api_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    let title = data
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("title"))
        .and_then(|v| v.as_str())?
        .to_string();

    // Artist name is in the "included" array
    let artist = data
        .get("included")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|item| item.get("type").and_then(|v| v.as_str()) == Some("artists"))
        })
        .and_then(|item| item.get("attributes"))
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some((title, artist))
}

/// Get an OAuth token from the QBZ proxy for the given platform.
pub(super) async fn get_proxy_token(platform: &str) -> Option<String> {
    let url = format!("{}/{}/token", QBZ_PROXY_BASE, platform);
    let data: serde_json::Value = reqwest::Client::builder()
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("QBZ/1.0.0"),
            );
            h
        })
        .build()
        .ok()?
        .get(&url)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    data.get("access_token")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
}
