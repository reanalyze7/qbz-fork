//! Parsing playlists, artists, and more-labels rows from the /label/page
//! and /label/explore JSON.

use std::collections::HashSet;

use qbz_models::LabelExploreResponse;
use serde_json::Value;

use super::value_helpers::{name_display, parse_artist_image, parse_explore_image, parse_playlist_image, value_to_string};
use super::{ArtistSlim, LabelSlim, PlaylistSlim};

pub(super) fn parse_playlist(raw: &Value) -> PlaylistSlim {
    let id = raw.get("id").map(value_to_string).unwrap_or_default();
    let title = raw
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let owner = raw
        .get("owner")
        .and_then(|o| o.get("name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Qobuz")
        .to_string();
    let track_count = raw.get("tracks_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let subtitle = format!("{owner} · {track_count}");
    PlaylistSlim {
        id,
        title,
        subtitle,
        image_url: parse_playlist_image(raw),
    }
}

pub(super) fn parse_artist(raw: &Value) -> ArtistSlim {
    ArtistSlim {
        id: raw.get("id").map(value_to_string).unwrap_or_default(),
        name: raw.get("name").map(name_display).unwrap_or_default(),
        image_url: parse_artist_image(raw),
    }
}

pub(super) fn parse_more_labels(
    resp: &LabelExploreResponse,
    current: u64,
    follow: &HashSet<u64>,
) -> Vec<LabelSlim> {
    let Some(items) = resp.items.as_ref() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let id = item.get("id").and_then(|x| x.as_u64())?;
            if id == current {
                return None;
            }
            let name = item
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let image_url = item
                .get("image")
                .map(parse_explore_image)
                .unwrap_or_default();
            Some(LabelSlim {
                id: id.to_string(),
                name,
                image_url,
                following: follow.contains(&id),
            })
        })
        .collect()
}
