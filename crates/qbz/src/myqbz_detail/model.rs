//! The render-row builder: `to_item` (MixtapeCollectionItem -> MixtapeDetailItem).

use qbz_models::mixtape::{AlbumSource, ItemType, MixtapeCollectionItem};
use slint::{ModelRc, VecModel};

use super::strings::{
    inline_cache_key, item_type_str, source_str, tracks_text, type_label, year_text,
};
use super::{INLINE_CACHE, RESOLVE_CACHE};
use crate::MixtapeDetailItem;

/// Build one ready-to-render row. The `_50` row-artwork downscale reuses the
/// grid controller's `small_qobuz_url`. Source kind defaults from the stored
/// `source` (the live local-vs-qobuz `resolveItems` resolution is
/// DEFERRED, so quality badge inputs stay empty here).
pub(super) fn to_item(item: &MixtapeCollectionItem) -> MixtapeDetailItem {
    let source = source_str(item.source);
    // `small_qobuz_url` only rewrites Qobuz CDN `_<size>.jpg` URLs; running it on
    // a LOCAL filesystem path corrupts/no-ops it.
    // Gate the rewrite to Qobuz items; local artwork passes through raw so
    // the source-aware artwork dispatch can read it as a file.
    let mut artwork_url = item
        .artwork_url
        .as_deref()
        .filter(|u| !u.is_empty())
        .map(|u| {
            if item.source == AlbumSource::Qobuz {
                crate::myqbz::small_qobuz_url(u, 50)
            } else {
                u.to_string()
            }
        })
        .unwrap_or_default();

    // Re-hydrate the live-resolved display values (source kind, quality
    // tier/detail, type label, backfilled artwork) from the resolveItems cache
    // so a filter/sort/search re-derive keeps the columns populated without
    // re-fetching. A miss falls back to the stored-source defaults (qobuz/local
    // + no quality), which the `resolve_items` pass then fills in. On a miss the
    // row is flagged `quality_resolving` so the quality cell shows a skeleton
    // until the async pass lands.
    let cache_key = inline_cache_key(source, &item.source_item_id);
    let resolved = RESOLVE_CACHE.with(|cell| cell.borrow().get(&cache_key).cloned());
    let (source_kind, quality_tier, quality_detail, type_label_v, artist_id, quality_resolving) =
        match resolved {
            Some(r) => {
                // Backfill the row cover from the resolved track when the stored
                // `artwork_url` was empty (disco-builder local items, older saves).
                if artwork_url.is_empty() && !r.artwork_url.is_empty() {
                    artwork_url = r.artwork_url.clone();
                }
                (r.source_kind, r.quality_tier, r.quality_detail, r.type_label, r.artist_id, false)
            }
            None => (
                source.to_string(),
                String::new(),
                String::new(),
                type_label(item.item_type),
                String::new(),
                true,
            ),
        };

    // Re-hydrate inline tracks from the controller-level cache (keyed
    // `source|source_item_id`) so a filter/sort/search re-derive does NOT lose
    // already-resolved tracks or trigger a re-fetch (spec 12 §8 — the cache
    // survives the re-derive). A cache hit marks the row loaded.
    let cache_key = inline_cache_key(source, &item.source_item_id);
    let (cached_tracks, tracks_loaded) = INLINE_CACHE.with(|cell| {
        match cell.borrow().get(&cache_key) {
            Some(tracks) => (tracks.clone(), true),
            None => (Vec::new(), false),
        }
    });

    MixtapeDetailItem {
        position: item.position,
        item_type: item_type_str(item.item_type).into(),
        source: source.into(),
        source_item_id: item.source_item_id.clone().into(),
        title: item.title.clone().into(),
        subtitle: item.subtitle.clone().unwrap_or_default().into(),
        // Only qobuz items get a clickable artist subtitle (spec 12 §6.3).
        subtitle_is_link: item.source == AlbumSource::Qobuz
            && item.subtitle.as_deref().map(|s| !s.is_empty()).unwrap_or(false),
        // Resolved Qobuz artist id ("" until resolveItems lands / for
        // local items) — routes the artist link to the Qobuz artist page.
        artist_id: artist_id.into(),
        // Resolved source kind / quality / type label — from the resolveItems
        // cache (above) when resolved; else the stored-source defaults.
        source_kind: source_kind.into(),
        type_label: type_label_v.into(),
        quality_tier: quality_tier.into(),
        quality_detail: quality_detail.into(),
        quality_resolving,
        tracks_text: tracks_text(item).into(),
        year_text: year_text(item).into(),
        artwork_url: artwork_url.into(),
        artwork: slint::Image::default(),
        selected: false,
        // Expanded-mode inline tracks (spec 12 §8): albums and playlists can
        // host inline tracks; a bare track item is itself (no expansion).
        can_expand: matches!(item.item_type, ItemType::Album | ItemType::Playlist),
        // Loaded/tracks come from the per-item cache (above) so the re-derive
        // keeps previously-resolved tracks instead of re-fetching.
        tracks_loaded,
        expand_loading: false,
        inline_tracks: ModelRc::new(VecModel::from(cached_tracks)),
    }
}
