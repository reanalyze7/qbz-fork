//! Resolve a whole collection's items into a flat queue.

use qbz_models::mixtape::{CollectionPlayMode, MixtapeCollection};
use qbz_models::QueueTrack;
use qbz_mixtape::enqueue::{resolve_collection_tracks, ProdItemResolver};

use super::{resolve_local, Runtime};

/// Resolve a whole collection's items into a flat queue.
///
/// Builds a `ProdItemResolver` over the shared Qobuz client (a clone taken
/// under the client `RwLock` so the value lives for the whole resolve — its
/// `&` reference must outlive the `.await`s the Qobuz arms perform) + the
/// `Send + Sync` `resolve_local` closure, then runs
/// `resolve_collection_tracks`. `force_shuffle` overrides the persisted mode
/// with `AlbumShuffle` (time-seeded whole-item shuffle) for the hero Shuffle
/// CTA; otherwise the collection's persisted `play_mode` is used.
pub(crate) async fn resolve_collection(
    runtime: &Runtime,
    collection: &MixtapeCollection,
    force_shuffle: bool,
) -> Vec<QueueTrack> {
    let play_mode = if force_shuffle {
        CollectionPlayMode::AlbumShuffle
    } else {
        collection.play_mode
    };

    // Snapshot the Qobuz client (mirrors v2_enqueue_collection step 3 /
    // playback.rs's prefetch path). The clone lives in `client`, so the `&`
    // handed to ProdItemResolver outlives every Qobuz `.await` in resolve.
    let client_lock = runtime.core().client();
    let client = {
        let guard = client_lock.read().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => {
                log::warn!("[qbz-slint] myqbz_play: no Qobuz client; resolving local items only");
                // Still build a resolver — local items resolve without the
                // client; Qobuz items will error+skip inside the resolver.
                // Cloning a missing client is impossible, so bail early with the
                // local-only subset is not feasible (the resolver needs a client
                // ref). Return empty: the caller toasts "0 playable tracks".
                return Vec::new();
            }
        }
    };

    let resolver = ProdItemResolver::new(&client, resolve_local);
    resolve_collection_tracks(collection.items.clone(), play_mode, &resolver).await
}
