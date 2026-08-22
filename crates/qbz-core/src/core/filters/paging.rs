//! Search-result page parsing: per-item lenient decode, plus the
//! `most_popular` hero-item picker.

use qbz_models::{Album, Artist, MostPopularItem, SearchResultsPage, Track};

use super::super::{AlbumBlacklistFilter, BlacklistFilter};

pub(super) fn parse_page<T: serde::de::DeserializeOwned>(
    value: &serde_json::Value,
    key: &str,
) -> SearchResultsPage<T> {
    // Scalars keep the old fallback semantics (missing/odd-shaped → 0, per
    // the serde defaults on SearchResultsPage), but the items array is parsed
    // PER ITEM: the previous whole-page `from_value(...).ok()` was
    // all-or-nothing, so one malformed entry blanked the entire search tab
    // (same class as favorites #556).
    let scalar = |name: &str| {
        value
            .get(key)
            .and_then(|p| p.get(name))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32
    };
    SearchResultsPage {
        items: qbz_models::lenient::parse_items_array(value, key, key),
        total: scalar("total"),
        offset: scalar("offset"),
        limit: scalar("limit"),
    }
}

/// Pick the first `most_popular` entry that survives the blacklist.
pub(super) fn pick_most_popular(
    value: &serde_json::Value,
    blacklist: &BlacklistFilter,
    album_bl: &AlbumBlacklistFilter,
) -> Option<MostPopularItem> {
    let items = value.get("most_popular")?.get("items")?.as_array()?;
    for entry in items {
        let Some(kind) = entry.get("type").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(content) = entry.get("content") else {
            continue;
        };
        match kind {
            "artists" => {
                // Artists have no album id — artist axis only.
                if let Ok(a) = serde_json::from_value::<Artist>(content.clone()) {
                    if !blacklist.contains(&a.id) {
                        return Some(MostPopularItem::Artists(a));
                    }
                }
            }
            "albums" => {
                if let Ok(al) = serde_json::from_value::<Album>(content.clone()) {
                    if !album_bl.contains(&al.id) && !blacklist.contains(&al.artist.id) {
                        return Some(MostPopularItem::Albums(al));
                    }
                }
            }
            "tracks" => {
                if let Ok(t) = serde_json::from_value::<Track>(content.clone()) {
                    let blocked = t
                        .album
                        .as_ref()
                        .is_some_and(|a| album_bl.contains(&a.id))
                        || t.performer
                            .as_ref()
                            .map_or(false, |p| blacklist.contains(&p.id));
                    if !blocked {
                        return Some(MostPopularItem::Tracks(t));
                    }
                }
            }
            _ => {}
        }
    }
    None
}
