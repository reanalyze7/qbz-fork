//! `navigate` — the top-level nav-in entry point.
use slint::ComponentHandle;

use super::reset_apply::{apply, apply_not_found, get_collection, reset};
use crate::artwork::ImageCache;
use crate::myqbz_detail::{artwork, resolve};
use crate::{AppWindow, ContentView, NavState};

/// Open the collection-detail view for `id`: switch the ContentView + loading
/// state immediately, fetch the collection on a blocking worker, then apply +
/// render + spawn (source-split) artwork + the resolveItems pass. Mirrors
/// `myqbz::navigate` (load/apply/render) and the album/playlist detail
/// navigators. The `runtime` drives the resolveItems backend resolution
/// (quality / source-kind / type per item).
pub fn navigate(
    runtime: std::sync::Arc<qbz_app::shell::AppRuntime<crate::adapter::SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
) {
    handle.clone().spawn(async move {
        {
            let weak = weak.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                reset(&w);
                w.global::<NavState>().set_view(ContentView::MixtapeDetail);
            });
        }

        let fetch_id = id.clone();
        let collection =
            tokio::task::spawn_blocking(move || get_collection(&fetch_id)).await.ok().flatten();

        // D11.c — OFFLINE: drop the items failing the availability rule
        // (qobuz not-cached / grace-expired) before the rows render. Online:
        // untouched.
        let collection = match collection {
            Some(mut c) if crate::offline_mode::engine().is_offline() => {
                let items: Vec<&qbz_models::mixtape::MixtapeCollectionItem> =
                    c.items.iter().collect();
                let avail = crate::myqbz::offline_availability(&items).await;
                drop(items);
                let before = c.items.len();
                c.items.retain(|it| avail.item_available(it));
                if c.items.len() < before {
                    log::info!(
                        "[qbz-slint] myqbz_detail {}: {} item(s) unavailable offline, hidden (D11)",
                        c.id,
                        before - c.items.len()
                    );
                }
                Some(c)
            }
            other => other,
        };

        let resolve_handle = handle.clone();
        let _ = weak.upgrade_in_event_loop(move |w| match collection {
            Some(c) => {
                apply(&w, c);
                let split = artwork::artwork_jobs(&w);
                artwork::dispatch_artwork(split, w.as_weak(), image_cache.clone());
                // resolveItems (spec §17): resolve each item's quality / source
                // kind / type from the backends and hydrate the rows (also
                // backfills + dispatches covers for rows stored with empty art).
                resolve::resolve_items(runtime, w.as_weak(), resolve_handle, image_cache.clone());
            }
            None => {
                log::warn!("[qbz-slint] myqbz_detail navigate({id}): collection not found");
                apply_not_found(&w);
            }
        });
    });
}
