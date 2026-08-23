//! Single-item resolve wrappers: the per-row/context-menu resolve, and the
//! display-only inline-track-expansion fetch.

use qbz_models::mixtape::MixtapeCollectionItem;
use qbz_models::QueueTrack;
use qbz_mixtape::enqueue::ProdItemResolver;

use super::{resolve_local, Runtime};

/// Resolve a SINGLE item (per-row actions). Mirrors `v2_enqueue_collection_item`
/// (spec 40 §6): resolve the one item directly, then **stamp
/// `source_item_id_hint = item.source_item_id` INLINE** (this path bypasses
/// `resolve_collection_tracks`, so the central stamp does not run). Failed
/// resolution logs + returns empty (the caller toasts "0 playable tracks").
pub(super) async fn resolve_single_item(
    runtime: &Runtime,
    item: &MixtapeCollectionItem,
) -> Vec<QueueTrack> {
    use qbz_mixtape::enqueue::ItemResolver;

    let client_lock = runtime.core().client();
    let client = {
        let guard = client_lock.read().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => {
                log::warn!("[qbz-slint] myqbz_play: no Qobuz client; cannot resolve item");
                return Vec::new();
            }
        }
    };

    let resolver = ProdItemResolver::new(&client, resolve_local);
    match resolver.resolve(item).await {
        Ok(mut tracks) => {
            // Inline boundary stamp (resolve_collection_tracks isn't used here).
            let hint = item.source_item_id.clone();
            for track in &mut tracks {
                track.source_item_id_hint = Some(hint.clone());
            }
            tracks
        }
        Err(e) => {
            log::warn!(
                "[qbz-slint] myqbz_play: item {}/{} resolve failed: {}",
                item.source_item_id,
                item.title,
                e
            );
            Vec::new()
        }
    }
}

/// Resolve a single item's tracks for the **expanded view-mode inline track
/// expansion** (spec 12 §8). Same resolver path as `resolve_single_item`
/// (Qobuz album/track/playlist + local via `resolve_local`), but used for
/// DISPLAY only — no queue mutation, no `source_item_id_hint` stamping. The
/// per-(item_type, source) routing the spec's `fetchTracksForItem` matrix
/// describes already lives inside the shared `ProdItemResolver::resolve` /
/// `resolve_local_item` (qobuz album->tracks, local album->tracks;
/// a local playlist returns its own resolver error → []),
/// so this stays a thin wrapper. Returns the resolved tracks (empty on any
/// resolver error, so the caller shows the per-item "no results" state).
pub(crate) async fn fetch_item_tracks(
    runtime: &Runtime,
    item: &MixtapeCollectionItem,
) -> Vec<QueueTrack> {
    use qbz_mixtape::enqueue::ItemResolver;

    let client_lock = runtime.core().client();
    let client = {
        let guard = client_lock.read().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => {
                log::warn!("[qbz-slint] myqbz_play: no Qobuz client; cannot fetch item tracks");
                // Local items resolve without the client, but the resolver
                // needs a client ref to build, so bail empty (the caller shows
                // the per-item empty state).
                return Vec::new();
            }
        }
    };

    let resolver = ProdItemResolver::new(&client, resolve_local);
    match resolver.resolve(item).await {
        Ok(tracks) => tracks,
        Err(e) => {
            // B10 — while OFFLINE the Qobuz arms are gate-refused by design
            // (cached items derive their badges locally in myqbz_detail), so a
            // per-item failure here is expected noise, not a fault — log it at
            // debug. Online failures stay warn.
            if crate::offline_mode::engine().is_offline() {
                log::debug!(
                    "[qbz-slint] myqbz_play: fetch_item_tracks {}/{} failed offline: {}",
                    item.source_item_id,
                    item.title,
                    e
                );
            } else {
                log::warn!(
                    "[qbz-slint] myqbz_play: fetch_item_tracks {}/{} failed: {}",
                    item.source_item_id,
                    item.title,
                    e
                );
            }
            Vec::new()
        }
    }
}
