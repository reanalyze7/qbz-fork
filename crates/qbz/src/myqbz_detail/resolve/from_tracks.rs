//! Deriving a row's resolved display values from its resolved tracks.

use qbz_models::mixtape::{ItemType, MixtapeCollectionItem};

use super::super::resolved_item::ResolvedItem;
use super::super::strings::{source_str, type_label};

/// Derive a row's resolved display values from its resolved tracks (spec §17
/// resolveItems). Album-level quality = the item's first resolved track's
/// quality (24-bit+ = Hi-Res); the same tier/detail rule every other surface
/// uses. Source kind = the first track's `source` (or `is_local` fallback) —
/// Type label: non-album rows keep their stored type;
/// album rows resolve to ALBUM/EP/SINGLE by the resolved track count (the
/// `QueueTrack` payload carries no release_type, so the track-count heuristic
/// — the same one favorites/labels use — applies).
pub(in crate::myqbz_detail) fn resolve_from_tracks(
    item: &MixtapeCollectionItem,
    tracks: &[qbz_models::QueueTrack],
) -> ResolvedItem {
    let stored = source_str(item.source);
    let first = tracks.first();

    let source_kind = match first {
        Some(t) => t
            .source
            .clone()
            .unwrap_or_else(|| if t.is_local { "local".into() } else { "qobuz".into() }),
        None => stored.to_string(),
    };

    let quality_tier = match first {
        Some(t) => match t.bit_depth {
            Some(d) if d >= 24 => "hires",
            Some(_) => "cd",
            None if t.hires => "hires",
            None => "",
        },
        None => "",
    };
    let quality_detail = match (first, quality_tier.is_empty()) {
        (Some(t), false) => crate::quality::detail(t.bit_depth, t.sample_rate),
        _ => String::new(),
    };

    // Type label: albums resolve their release type from the resolved track
    // count; tracks/playlists keep their stored type. Uppercased to match the
    // column eyebrow.
    let type_label_v = match item.item_type {
        ItemType::Album => {
            crate::album_map::classify_release_type(Some(tracks.len() as u32)).to_uppercase()
        }
        other => type_label(other),
    };

    // First resolved track's artwork — backfills rows whose stored
    // `artwork_url` was empty (disco-builder local items saved with NULL art).
    // Strip the `file://` prefix that `local_queue_track` adds: the source-aware
    // artwork dispatch reads a bare filesystem path (a raw `tokio::fs::read` of a
    // `file://…` URI fails). Qobuz CDN urls have no prefix and pass through
    // unchanged.
    let artwork_url = first
        .and_then(|t| t.artwork_url.clone())
        .map(|u| u.strip_prefix("file://").map(str::to_string).unwrap_or(u))
        .unwrap_or_default();

    // First resolved track's Qobuz artist id — empty for local tracks
    // (QueueTrack.artist_id is None there). Feeds the row's artist link.
    let artist_id = first
        .and_then(|t| t.artist_id)
        .map(|id| id.to_string())
        .unwrap_or_default();

    ResolvedItem {
        source_kind,
        quality_tier: quality_tier.to_string(),
        quality_detail,
        type_label: type_label_v,
        artwork_url,
        artist_id,
    }
}
