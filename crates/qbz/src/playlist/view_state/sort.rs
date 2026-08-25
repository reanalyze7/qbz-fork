//! `refresh_view` — re-derive the visible track list from `FULL_ITEMS` by
//! applying the active search filter, then the active sort.

use crate::{AppWindow, PlaylistState, TrackItem};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::hires_filter::keeps;
use super::{custom_key, FULL_ITEMS, HIRES_ONLY, QUERY, SORT};

/// "m:ss" / "h:mm:ss" -> seconds, for duration sorting.
fn duration_secs(s: &str) -> u32 {
    s.split(':')
        .filter_map(|p| p.parse::<u32>().ok())
        .fold(0, |acc, n| acc * 60 + n)
}

/// Re-derive the visible track list from FULL_ITEMS by applying the
/// active search filter, then the active sort. Runs on the event loop.
pub(in crate::playlist) fn refresh_view(window: &AppWindow) {
    let needle = QUERY.with(|q| q.borrow().trim().to_lowercase());
    let (field, asc) = SORT.with(|s| s.borrow().clone());
    let hires_only = HIRES_ONLY.with(|h| h.get());
    let mut view: Vec<TrackItem> = FULL_ITEMS.with(|cell| {
        cell.borrow()
            .iter()
            .filter(|t| keeps(t.quality_tier.as_str(), hires_only))
            .filter(|t| {
                needle.is_empty()
                    || t.title.as_str().to_lowercase().contains(&needle)
                    || t.artist.as_str().to_lowercase().contains(&needle)
                    || t.album.as_str().to_lowercase().contains(&needle)
            })
            .cloned()
            .collect()
    });
    if field == "custom" {
        // Order by the local custom positions; rows not in the map sort
        // to the END in their natural relative order (E6 — deliberate
        // Slint rule; Tauri's addedIndex-0 fallback floats them to the
        // top by accident).
        let order = super::super::custom_order::CUSTOM_ORDER.with(|c| c.borrow().clone());
        view.sort_by_key(|t| {
            custom_key(t)
                .and_then(|k| order.get(&k).copied())
                .unwrap_or(i32::MAX)
        });
    } else if field == "added" {
        // "Date added" is a positional proxy (v1.x parity): FULL_ITEMS is in
        // the playlist's natural insertion order (the API returns tracks
        // oldest-first) and the filter above preserves it, so asc = oldest
        // first and desc = newest first is a plain reversal.
        if !asc {
            view.reverse();
        }
    } else if field != "default" {
        view.sort_by(|a, b| {
            let ord = match field.as_str() {
                "title" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                "artist" => a.artist.to_lowercase().cmp(&b.artist.to_lowercase()),
                "album" => a.album.to_lowercase().cmp(&b.album.to_lowercase()),
                "duration" => {
                    duration_secs(a.duration.as_str()).cmp(&duration_secs(b.duration.as_str()))
                }
                _ => std::cmp::Ordering::Equal,
            };
            if asc {
                ord
            } else {
                ord.reverse()
            }
        });
    }
    window
        .global::<PlaylistState>()
        .set_tracks(ModelRc::new(VecModel::from(view)));
}
