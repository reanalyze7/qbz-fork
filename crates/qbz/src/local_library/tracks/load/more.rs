//! Fetch + append the next tracks page (scroll-near-bottom).

use std::sync::atomic::Ordering;

use slint::ComponentHandle;

use crate::{AppWindow, LocalLibraryState};

use super::apply::append_tracks;
use super::state::{fetch_tracks_page, TRACKS_GEN};

pub fn load_more_tracks(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<LocalLibraryState>();
        if !s.get_tracks_has_more() || s.get_tracks_loading_more() || s.get_tracks_loading() {
            return;
        }
        let offset = s.get_tracks_next_offset().max(0) as u64;
        let query = s.get_tracks_search().to_string();
        let sort = s.get_tracks_sort().to_string();
        let gen = TRACKS_GEN.load(Ordering::SeqCst);
        s.set_tracks_loading_more(true);
        let weak2 = w.as_weak();
        handle.spawn(async move {
            let result =
                tokio::task::spawn_blocking(move || fetch_tracks_page(query, offset, sort))
                    .await
                    .ok()
                    .flatten();
            let _ = weak2.upgrade_in_event_loop(move |w| {
                let s = w.global::<LocalLibraryState>();
                if TRACKS_GEN.load(Ordering::SeqCst) != gen {
                    s.set_tracks_loading_more(false);
                    return;
                }
                match result {
                    Some((rows, has_more)) => append_tracks(&w, rows, has_more),
                    None => s.set_tracks_loading_more(false),
                }
            });
        });
    });
}
