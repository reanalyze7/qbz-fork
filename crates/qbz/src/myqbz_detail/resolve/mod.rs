//! resolveItems pass (spec §17): the async orchestration that resolves every
//! rendered row's tracks and hydrates the source/quality/type columns.

mod from_tracks;
mod offline;

use slint::Weak;

use from_tracks::resolve_from_tracks;
use offline::resolve_offline_cached;

use super::artwork::{dispatch_artwork, ArtworkJobSplit};
use super::strings::{inline_cache_key, source_str};
use super::{inline_tracks::with_row_by_source_id, FULL_ITEMS, RESOLVE_CACHE};
use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::AppWindow;

/// resolveItems pass (spec §17): resolve every rendered row's tracks via the
/// shared enqueue resolver (`myqbz_play::fetch_item_tracks` — the SAME
/// qobuz/local backends), derive the row's source kind + album-level
/// quality + type label from the first resolved track, push the values into the
/// row, and cache them (keyed `source|source_item_id`) so a later filter/sort/
/// search re-derive re-hydrates instead of re-fetching. Spawned once after
/// `apply` (alongside the artwork jobs); already-cached rows are skipped, so a
/// re-derive is instant. Fire-and-forget: failures leave the stored-source
/// defaults in place.
///
/// OFFLINE (B10): a cached Qobuz item's badges resolve from the LOCAL offline
/// index via `resolve_offline_cached` — the API path would be gate-refused and
/// the badge would stay empty. Online the branch is never taken, so that path
/// is byte-identical.
pub fn resolve_items(
    runtime: std::sync::Arc<qbz_app::shell::AppRuntime<crate::adapter::SlintAdapter>>,
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    let Some(window) = weak.upgrade() else { return };

    // Snapshot the items needing resolution (every full item not already
    // cached). Sourced from FULL_ITEMS so the resolver gets the typed
    // item_type/source + numeric fields.
    let pending: Vec<qbz_models::mixtape::MixtapeCollectionItem> = FULL_ITEMS.with(|cell| {
        cell.borrow()
            .iter()
            .filter(|it| {
                let key = inline_cache_key(source_str(it.source), &it.source_item_id);
                !RESOLVE_CACHE.with(|c| c.borrow().contains_key(&key))
            })
            .cloned()
            .collect()
    });
    drop(window);

    for full_item in pending {
        let runtime = runtime.clone();
        let weak = weak.clone();
        let image_cache = image_cache.clone();
        handle.spawn(async move {
            // B10 — OFFLINE: cached Qobuz items resolve their badges locally
            // (offline index); everything else (online, local, uncached)
            // takes the existing resolver path unchanged.
            let offline_resolved = if crate::offline_mode::engine().is_offline() {
                resolve_offline_cached(&full_item).await
            } else {
                None
            };
            let resolved = match offline_resolved {
                Some(r) => r,
                None => {
                    let tracks =
                        crate::myqbz_play::fetch_item_tracks(&runtime, &full_item).await;
                    resolve_from_tracks(&full_item, &tracks)
                }
            };
            let source = source_str(full_item.source).to_string();
            let source_item_id = full_item.source_item_id.clone();
            let stored_artwork_empty = full_item
                .artwork_url
                .as_deref()
                .map(|u| u.is_empty())
                .unwrap_or(true);
            let _ = weak.upgrade_in_event_loop(move |w| {
                let key = inline_cache_key(&source, &source_item_id);
                RESOLVE_CACHE.with(|cell| {
                    cell.borrow_mut().insert(key, resolved.clone());
                });
                // Push the resolved values into the currently-rendered row (if
                // still present after any re-derive). Clear `quality_resolving`
                // (the skeleton) and backfill the row cover when the stored
                // `artwork_url` was empty (disco-builder local items, older
                // saves) so the disc placeholder is replaced by the real art —
                // the album-view pattern applied to the detail rows.
                let mut backfilled_pos: Option<i32> = None;
                with_row_by_source_id(&w, &source_item_id, |it| {
                    it.source_kind = resolved.source_kind.clone().into();
                    it.quality_tier = resolved.quality_tier.clone().into();
                    it.quality_detail = resolved.quality_detail.clone().into();
                    it.type_label = resolved.type_label.clone().into();
                    it.artist_id = resolved.artist_id.clone().into();
                    it.quality_resolving = false;
                    if it.artwork_url.is_empty() && !resolved.artwork_url.is_empty() {
                        it.artwork_url = resolved.artwork_url.clone().into();
                        backfilled_pos = Some(it.position);
                    }
                });
                // Dispatch the one backfilled cover through the source-aware
                // path (qobuz CDN -> HTTP; local -> source-aware decode).
                // Only when the stored art was empty AND a row was actually
                // backfilled (skips the common already-had-art case).
                if stored_artwork_empty {
                    if let Some(pos) = backfilled_pos {
                        let job = ArtworkJob {
                            target: ArtworkTarget::MyQbzDetailRow { position: pos },
                            url: resolved.artwork_url.clone(),
                        };
                        let split = if resolved.source_kind == "qobuz" {
                            ArtworkJobSplit { remote: vec![job], ..Default::default() }
                        } else {
                            ArtworkJobSplit { local_or_plex: vec![job], ..Default::default() }
                        };
                        dispatch_artwork(split, w.as_weak(), image_cache.clone());
                    }
                }
            });
        });
    }
}
