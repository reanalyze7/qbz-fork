//! Parsing one Popular-Tracks row from the /label/page `top_tracks` JSON.

use serde_json::Value;

use super::value_helpers::{mmss, name_display, parse_image_value, value_to_string};
use super::TopTrack;
use crate::album_map::tier;

pub(super) fn parse_top_track(raw: &Value) -> TopTrack {
    let id = raw
        .get("id")
        .and_then(|v| v.as_u64())
        .map(|n| n.to_string())
        .unwrap_or_default();
    let title = raw
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let duration = raw.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);
    let album = raw.get("album");
    let album_id = album
        .and_then(|a| a.get("id"))
        .map(value_to_string)
        .unwrap_or_default();
    let album_title = album
        .and_then(|a| a.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let artwork_url = album
        .and_then(|a| a.get("image"))
        .map(parse_image_value)
        .unwrap_or_default();
    // Artist: the label /page top_tracks carry the main artist in a track-level
    // `artists` array (roles = "main-artist"), NOT `performer`/`artist` (those
    // are null here) — discussion #631. Prefer the main-artist entry, then the
    // first artist, then the legacy performer/artist/album-artist shapes.
    let main_artist = raw
        .get("artists")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|a| {
                    a.get("roles")
                        .and_then(|r| r.as_array())
                        .map(|roles| roles.iter().any(|x| x.as_str() == Some("main-artist")))
                        .unwrap_or(false)
                })
                .or_else(|| arr.first())
        })
        .or_else(|| raw.get("performer"))
        .or_else(|| raw.get("artist"))
        .or_else(|| album.and_then(|a| a.get("artist")));
    let artist = main_artist
        .and_then(|p| p.get("name"))
        .map(name_display)
        .unwrap_or_default();
    let artist_id = main_artist
        .and_then(|p| p.get("id"))
        .map(value_to_string)
        .unwrap_or_default();
    let bit_depth = raw
        .get("audio_info")
        .and_then(|a| a.get("maximum_bit_depth"))
        .and_then(|v| v.as_u64())
        .or_else(|| raw.get("maximum_bit_depth").and_then(|v| v.as_u64()));
    let sample_rate = raw
        .get("audio_info")
        .and_then(|a| a.get("maximum_sampling_rate"))
        .and_then(|v| v.as_f64())
        .or_else(|| raw.get("maximum_sampling_rate").and_then(|v| v.as_f64()));
    let bit_depth = bit_depth.map(|b| b as u32);
    TopTrack {
        id,
        title,
        artist,
        artist_id,
        album_id,
        album: album_title,
        artwork_url,
        duration: mmss(duration as u32),
        quality_tier: tier(bit_depth).to_string(),
        quality_detail: crate::quality::detail(bit_depth, sample_rate),
    }
}
