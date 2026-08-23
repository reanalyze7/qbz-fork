//! Lazy load on first visit (re-entry keeps it) — the Artists tab entry point.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, Model};

use crate::adapter::SlintAdapter;
use crate::artwork::ImageCache;
use crate::{AppWindow, LocalLibraryState};

use crate::local_library::albums::current_group_mode;
use crate::local_library::artists::images::ARTISTS_IMG_GEN;
use crate::local_library::artists::select::select_local_artist;
use crate::local_library::artists::state::take_pending_artist;

use super::fetch::load_and_merge_artists;
use super::finish::finish_artists_load;

/// Load + merge the artists master list on first visit (re-entry keeps it).
/// Also caches the album set for the right-pane filter, seeds decode jobs for
/// rows that already have an image (custom or previously-cached Qobuz), and
/// kicks the capped background fetch for the rest.
pub fn ensure_artists_loaded(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<LocalLibraryState>();
        if s.get_artists().row_count() != 0 || s.get_artists_loading() {
            // Already loaded → satisfy a pending open-artist immediately.
            if s.get_artists().row_count() != 0 {
                if let Some(name) = take_pending_artist() {
                    select_local_artist(w.as_weak(), handle.clone(), image_cache.clone(), name);
                }
            }
            return;
        }
        s.set_artists_loading(true);
        s.set_artists_load_failed(false);
        let gen = ARTISTS_IMG_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        let weak2 = w.as_weak();
        let handle_inner = handle.clone();
        // Album-identity mode for the album cache below — the Artists tab
        // must group albums the same way as the Albums tab (a folder-mode
        // compilation cross-lists under every artist in all_artists).
        let _group_mode = current_group_mode(&w);
        handle.spawn(async move {
            let items = tokio::task::spawn_blocking(load_and_merge_artists)
                .await
                .unwrap_or_default();
            let _ = weak2.upgrade_in_event_loop(move |w| {
                finish_artists_load(&w, runtime, handle_inner, image_cache, gen, items);
            });
        });
    });
}
