//! Page-1 load + reload/ensure entry points for the Tracks tab.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Model};

use crate::{AppWindow, LocalLibraryState};

use super::apply::apply_tracks;
use super::state::{fetch_tracks_page, TRACKS_GEN};

pub(crate) fn spawn_tracks_page_load(window: &AppWindow, handle: tokio::runtime::Handle, gen: u64) {
    let s = window.global::<LocalLibraryState>();
    let query = s.get_tracks_search().to_string();
    let sort = s.get_tracks_sort().to_string();
    let weak = window.as_weak();
    handle.spawn(async move {
        let result = tokio::task::spawn_blocking(move || fetch_tracks_page(query, 0, sort))
            .await
            .ok()
            .flatten();
        let _ = weak.upgrade_in_event_loop(move |w| {
            if TRACKS_GEN.load(Ordering::SeqCst) != gen {
                return;
            }
            match result {
                Some((rows, has_more)) => apply_tracks(&w, rows, has_more),
                None => {
                    let s = w.global::<LocalLibraryState>();
                    s.set_tracks_loading(false);
                    s.set_tracks_load_failed(true);
                }
            }
        });
    });
}

/// (Re)load page 1 of the tracks list with the current search.
pub fn reload_tracks(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let gen = TRACKS_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        let s = w.global::<LocalLibraryState>();
        s.set_tracks_loading(true);
        s.set_tracks_load_failed(false);
        spawn_tracks_page_load(&w, handle, gen);
    });
}

/// Lazy load on first visit (re-entry keeps the loaded set + scroll).
pub fn ensure_tracks_loaded(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<LocalLibraryState>();
        if s.get_tracks().row_count() == 0 && !s.get_tracks_loading() {
            let gen = TRACKS_GEN.fetch_add(1, Ordering::SeqCst) + 1;
            s.set_tracks_loading(true);
            s.set_tracks_load_failed(false);
            spawn_tracks_page_load(&w, handle, gen);
        }
    });
}
