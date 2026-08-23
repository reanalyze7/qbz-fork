//! Hero Shuffle: replace-play with forced `AlbumShuffle`, plus the
//! `play_mode='album_shuffle'` persist side effect.

use qbz_models::mixtape::CollectionPlayMode;

use crate::AppWindow;

use super::hero::play_all_tracks;
use super::load::load_collection;
use super::resolve::resolve_collection;
use super::Runtime;

/// Hero **Shuffle** (`on_shuffle`): resolve with forced `AlbumShuffle`
/// ordering, then replace-play (same queue-source stamp + touch_play as Play —
/// it is a replace). As a SIDE EFFECT (1:1 with Tauri `handleShuffle`, spec 12
/// §20), Shuffle also persists `play_mode='album_shuffle'` so the collection
/// remembers it; if THIS is the open collection, the detail is reloaded so the
/// overflow play-mode-toggle label flips to the other mode ("Play in order").
/// Persist runs ONLY on Shuffle — a normal hero Play never changes play_mode.
pub fn shuffle(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: crate::artwork::ImageCache,
    collection_id: String,
) {
    handle.clone().spawn(async move {
        let Some(collection) = load_collection(&collection_id).await else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load this collection"));
            return;
        };
        let tracks = resolve_collection(&runtime, &collection, true).await;
        play_all_tracks(&runtime, &weak, &collection_id, tracks).await;

        // Persist play_mode='album_shuffle' as a side effect (Tauri's
        // `setPlayMode` in handleShuffle). Best-effort: only persist when the
        // collection wasn't already album_shuffle, and reload the open detail so
        // the overflow toggle label reflects the new mode.
        if collection.play_mode != CollectionPlayMode::AlbumShuffle {
            persist_album_shuffle(&runtime, &weak, &handle, &image_cache, &collection_id);
        }
    });
}

/// Best-effort `repo::set_play_mode(id, AlbumShuffle)` (spec 12 §20 shuffle
/// side effect), then reload the detail IF `id` is the currently-open
/// collection (so the hero overflow play-mode-toggle label flips). DB work runs
/// on a blocking worker (no `&Connection` crosses an `.await`); the reload hops
/// back to the event loop. Failures are logged, not surfaced — the shuffle
/// already played, so a failed persist must not toast an error.
fn persist_album_shuffle(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &crate::artwork::ImageCache,
    collection_id: &str,
) {
    let write_id = collection_id.to_string();
    let persisted = crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            qbz_mixtape::repo::set_play_mode(conn, &write_id, CollectionPlayMode::AlbumShuffle)
        }))
    });
    match persisted {
        Some(Ok(())) => {}
        Some(Err(e)) => {
            log::warn!("[qbz-slint] myqbz_play: persist album_shuffle({collection_id}) failed: {e}");
            return;
        }
        None => {
            log::warn!("[qbz-slint] myqbz_play: persist album_shuffle: library db unavailable");
            return;
        }
    }

    // Reload the detail only when this collection is the one on screen, so the
    // overflow toggle label flips to the OTHER mode. Off-screen collections need
    // no reload — their next open restores from the DB.
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let runtime = runtime.clone();
    let id = collection_id.to_string();
    let _ = weak.upgrade_in_event_loop(move |w| {
        use slint::ComponentHandle;
        if w.global::<crate::MyQbzDetailState>().get_id().as_str() == id {
            crate::myqbz_detail::navigate(runtime, w.as_weak(), handle, image_cache, id);
        }
    });
}
