//! Capped, sequential background fetch of missing artist portraits from
//! Qobuz.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, Model};

use crate::adapter::SlintAdapter;
use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, LocalLibraryState};

use crate::local_library::artists::normalize::normalize_artist;

use super::state::artists_img_gen_current;

/// Capped, sequential background fetch of missing artist portraits from Qobuz
/// (max 50/session, 1s apart, exact-normalized match only). 1:1 with Tauri's
/// `fetchMissingArtistImages`, with per-image immediate paint + a generation
/// guard. Names with an image already are skipped (snapshotted on the UI
/// thread; the worker never touches the Slint model except via event-loop hops).
pub fn fetch_missing_artist_images(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    gen: u64,
    mut names: Vec<String>,
) {
    // `names` is snapshotted by the caller on the UI thread (NEVER block the
    // event loop here — this can be invoked from inside an event-loop closure).
    names.truncate(50);
    if names.is_empty() {
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<LocalLibraryState>().set_artists_images_fetching(false);
        });
        return;
    }

    handle.spawn(async move {
        let mut painted = 0i32;
        for name in names {
            if artists_img_gen_current() != gen {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            if artists_img_gen_current() != gen {
                break;
            }
            let page = match runtime.core().search_artists(&name, 3, 0, None).await {
                Ok(p) => p,
                Err(qbz_core::CoreError::NotInitialized) => break,
                Err(e) => {
                    log::debug!("[locallibrary] artist image search failed for {name}: {e}");
                    continue;
                }
            };
            let nsel = normalize_artist(&name);
            let matched = page
                .items
                .into_iter()
                .find(|a| normalize_artist(&a.name) == nsel);
            let Some(artist) = matched else {
                continue; // no exact match -> skip (no wrong-artist persist)
            };
            let Some(url) = artist.image.as_ref().and_then(|i| i.best().cloned()) else {
                continue;
            };

            // Persist the fetched portrait (best-effort; paints regardless).
            let name_c = name.clone();
            let url_c = url.clone();
            let canon = artist.name.clone();
            let _ = tokio::task::spawn_blocking(move || {
                crate::library_db::with_db(|db| {
                    db.cache_artist_image_with_canonical(
                        &name_c,
                        Some(&url_c),
                        "qobuz",
                        None,
                        Some(&canon),
                    )
                })
            })
            .await;

            // Paint now: resolve the current flat-master index on the UI thread.
            let url_p = url.clone();
            let name_p = name.clone();
            let cache = image_cache.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                let s = w.global::<LocalLibraryState>();
                let flat = s.get_artists();
                let mut idx = None;
                for i in 0..flat.row_count() {
                    if let Some(a) = flat.row_data(i) {
                        if a.name.as_str() == name_p {
                            idx = Some(i);
                            break;
                        }
                    }
                }
                if let Some(i) = idx {
                    crate::artwork::spawn_loads(
                        vec![ArtworkJob {
                            target: ArtworkTarget::LocalArtistRowImage { index: i, gen },
                            url: url_p,
                        }],
                        w.as_weak(),
                        cache,
                    );
                }
            });
            painted += 1;
        }
        let _ = weak.upgrade_in_event_loop(move |w| {
            let s = w.global::<LocalLibraryState>();
            s.set_artists_images_fetching(false);
            s.set_artists_images_fetched(painted);
        });
    });
}
