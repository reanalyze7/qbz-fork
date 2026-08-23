//! Multi-select bulk enqueue (`bulk_ids.rs` has the Qobuz-track-id resolve
//! for "Add to playlist").

use qbz_models::mixtape::MixtapeCollectionItem;
use qbz_models::QueueTrack;
use qbz_mixtape::enqueue::ProdItemResolver;

use crate::playback::refresh_sidebar;
use crate::AppWindow;

use super::{resolve_local, Runtime};

/// **Bulk** enqueue for the detail select-mode bulk bar (spec 12 §13.1 Add to
/// queue / Play next). Resolves EACH selected `MixtapeCollectionItem` through
/// the same `ProdItemResolver` the per-row path uses (so Qobuz albums/tracks/
/// playlists + local all resolve), flattens them in selection order, then:
/// - **play_next = true**: insert the whole batch immediately after the current
///   track, in REVERSE so the first resolved track lands first (same rule as the
///   per-row `PlayNext`).
/// - **play_next = false**: append the batch at the end of the queue.
///
/// Never replaces the queue and never stamps the queue-source collection
/// (append/play-next preserve context, mirroring the per-row contract). Items
/// that resolve to nothing are logged + skipped; an all-empty batch toasts.
pub fn bulk_enqueue(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    items: Vec<MixtapeCollectionItem>,
    play_next: bool,
) {
    if items.is_empty() {
        return;
    }
    handle.spawn(async move {
        let client_lock = runtime.core().client();
        let client = {
            let guard = client_lock.read().await;
            match guard.as_ref() {
                Some(c) => c.clone(),
                None => {
                    log::warn!("[qbz-slint] myqbz_play: no Qobuz client; cannot bulk-enqueue");
                    crate::toast::error_weak(&weak, qbz_i18n::t("These items resolved to 0 playable tracks"));
                    return;
                }
            }
        };
        let resolver = ProdItemResolver::new(&client, resolve_local);

        // Resolve each item in selection order, stamping the per-item boundary
        // hint inline (this path bypasses resolve_collection_tracks).
        use qbz_mixtape::enqueue::ItemResolver;
        let mut tracks: Vec<QueueTrack> = Vec::new();
        for item in &items {
            match resolver.resolve(item).await {
                Ok(mut resolved) => {
                    let hint = item.source_item_id.clone();
                    for t in &mut resolved {
                        t.source_item_id_hint = Some(hint.clone());
                    }
                    tracks.extend(resolved);
                }
                Err(e) => {
                    log::warn!(
                        "[qbz-slint] myqbz_play: bulk item {}/{} resolve failed: {}",
                        item.source_item_id,
                        item.title,
                        e
                    );
                }
            }
        }

        if tracks.is_empty() {
            crate::toast::error_weak(&weak, qbz_i18n::t("These items resolved to 0 playable tracks"));
            return;
        }

        if play_next {
            // REVERSE so the first resolved track lands immediately after the
            // current track (spec §9.8).
            for track in tracks.into_iter().rev() {
                runtime.core().add_track_next(track).await;
            }
            refresh_sidebar(false);
            crate::toast::success_weak(&weak, qbz_i18n::t("Playing next"));
        } else {
            runtime.core().add_tracks(tracks).await;
            refresh_sidebar(false);
            crate::toast::success_weak(&weak, qbz_i18n::t("Added to queue"));
        }
    });
}
