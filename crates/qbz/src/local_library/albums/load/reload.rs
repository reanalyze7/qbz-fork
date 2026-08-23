//! Reload / lazy-load entry points for the Albums tab.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Model};

use crate::artwork::ImageCache;
use crate::{AppWindow, LocalLibraryState};

use crate::local_library::albums::artwork::albums_inflight;

use super::spawn::spawn_albums_load;
use super::state::ALBUMS_GEN;

/// (Re)load the full album set, bumping the generation guard.
pub fn reload_albums(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let gen = ALBUMS_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        // The gen bump orphans every in-flight windowed job (dropped on
        // apply) — free their dedupe slots so the new set can re-request.
        albums_inflight().lock().unwrap().clear();
        let s = w.global::<LocalLibraryState>();
        s.set_albums_loading(true);
        s.set_albums_load_failed(false);
        spawn_albums_load(&w, handle, image_cache, gen);
    });
}

/// Load on first visit only (re-entry keeps the set + derived views).
pub fn ensure_albums_loaded(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<LocalLibraryState>();
        if s.get_albums().row_count() == 0 && !s.get_albums_loading() {
            let gen = ALBUMS_GEN.fetch_add(1, Ordering::SeqCst) + 1;
            s.set_albums_loading(true);
            s.set_albums_load_failed(false);
            spawn_albums_load(&w, handle, image_cache, gen);
        }
    });
}
