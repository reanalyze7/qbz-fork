//! B10 — OFFLINE: derive a cached Qobuz item's badge values from local
//! sources instead of the (gate-refused) Qobuz API.

use qbz_models::mixtape::{AlbumSource, ItemType, MixtapeCollectionItem};

use super::super::resolved_item::ResolvedItem;
use super::super::strings::type_label;

/// B10 — OFFLINE: derive a cached Qobuz item's badge values from LOCAL sources
/// instead of the Qobuz API (which the offline gate refuses, leaving the badge
/// empty). The offline index (`index.db` `cached_tracks` rows, surfaced as
/// `CachedTrackInfo`) carries `quality` / `bit_depth` / `sample_rate` per
/// track; the tier/detail rule is the SAME one `resolve_from_tracks` and the
/// LocalLibrary "Offline"-source rows use (24-bit+ = Hi-Res, detail via
/// `crate::quality::detail` — the library.db `qobuz_download` rows derive their
/// badge from these very columns at sync time), so the offline badge matches
/// every other surface. Returns `None` for non-Qobuz items, Qobuz playlists
/// (membership is API-side), and uncached/non-ready items — the caller then
/// falls through to the normal resolver path.
pub(in crate::myqbz_detail) async fn resolve_offline_cached(
    item: &MixtapeCollectionItem,
) -> Option<ResolvedItem> {
    use qbz_offline_cache::OfflineCacheStatus;

    if item.source != AlbumSource::Qobuz {
        return None;
    }
    let off = crate::offline::get().await?;
    let guard = off.db.lock().await;
    let db = guard.as_ref()?;

    let (info, ready_count) = match item.item_type {
        ItemType::Album => {
            let ready: Vec<qbz_offline_cache::CachedTrackInfo> = db
                .get_album_tracks(&item.source_item_id)
                .ok()?
                .into_iter()
                .filter(|cached| matches!(cached.status, OfflineCacheStatus::Ready))
                .collect();
            let count = ready.len();
            (ready.into_iter().next()?, count)
        }
        ItemType::Track => {
            let id = item.source_item_id.parse::<u64>().ok()?;
            let info = db.get_track(id).ok().flatten()?;
            if !matches!(info.status, OfflineCacheStatus::Ready) {
                return None;
            }
            (info, 1)
        }
        // Qobuz playlist membership lives in the API — not derivable locally.
        ItemType::Playlist => return None,
    };

    // Quality tier/detail — the resolve_from_tracks rule over the index
    // columns. The stored quality string ("UltraHiRes" etc.) stands in for the
    // missing hires flag when bit_depth is NULL.
    let quality_tier = match info.bit_depth {
        Some(d) if d >= 24 => "hires",
        Some(_) => "cd",
        None if info.quality.to_ascii_lowercase().contains("hires") => "hires",
        None => "",
    };
    let quality_detail = if quality_tier.is_empty() {
        String::new()
    } else {
        crate::quality::detail(info.bit_depth, info.sample_rate)
    };

    // Type label: albums classify by the stored track_count (the album's REAL
    // count, captured at add time); the cached READY count is the fallback for
    // older rows saved without one (a floor — partial caches undercount).
    let type_label_v = match item.item_type {
        ItemType::Album => {
            let count = item
                .track_count
                .filter(|n| *n > 0)
                .map(|n| n as u32)
                .unwrap_or(ready_count as u32);
            crate::album_map::classify_release_type(Some(count)).to_uppercase()
        }
        other => type_label(other),
    };

    Some(ResolvedItem {
        source_kind: "qobuz".to_string(),
        quality_tier: quality_tier.to_string(),
        quality_detail,
        type_label: type_label_v,
        // No artwork backfill offline: the empty-stored-art backfill is a
        // local-item concern (those still resolve through the normal path),
        // and a Qobuz CDN url could not be fetched offline anyway.
        artwork_url: String::new(),
        // The offline index carries no artist id — and the Qobuz artist page
        // is gate-blocked offline anyway, so the link stays a no-op here.
        artist_id: String::new(),
    })
}
