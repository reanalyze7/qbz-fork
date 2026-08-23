use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Model};

use super::{fav_albums_dispatch_ctx, fav_albums_inflight, FAV_ALBUMS_GEN};
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, FavoritesState};

/// Flat LIST-view artwork: same `albums-visible` model as the grid, but the
/// list is not windowed (only `AlbumGrid` fires `window-changed`) — dispatch
/// every missing cover, no eviction. Phase 1 limit.
pub(crate) fn dispatch_fav_albums_all_visible(window: &AppWindow) {
    let Some(image_cache) = fav_albums_dispatch_ctx().lock().unwrap().clone() else {
        return;
    };
    let gen = FAV_ALBUMS_GEN.load(Ordering::SeqCst);
    let state = window.global::<FavoritesState>();
    let visible = state.get_albums_visible();
    let mut jobs = Vec::new();
    {
        let mut inflight = fav_albums_inflight().lock().unwrap();
        for vi in 0..visible.row_count() {
            let Some(item) = visible.row_data(vi) else { continue };
            if item.artwork.size().width > 0 || item.artwork_url.is_empty() {
                continue;
            }
            let id = item.id.to_string();
            if inflight.insert(id.clone()) {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::FavoriteAlbumById { id, gen },
                    url: item.artwork_url.to_string(),
                });
            }
        }
    }
    if !jobs.is_empty() {
        crate::artwork::spawn_loads(jobs, window.as_weak(), image_cache);
    }
}

/// The Albums grid/list view-mode toggled. Switching TO the list needs a full
/// dispatch (the list is not windowed and the grid's window may have evicted
/// covers the list now shows); switching to the grid is handled by AlbumGrid's
/// own post-layout seed Timer on mount (it re-fires `window-changed`).
pub fn albums_view_mode_changed(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    if state.get_albums_group_mode() == "off" && state.get_albums_view_mode() == "list" {
        dispatch_fav_albums_all_visible(window);
    }
}

/// Grouped-mode artwork: `albums-visible` is EMPTY there (the sections render
/// from `albums-grouped`), so the viewport window doesn't apply — keep the
/// pre-windowing behavior and dispatch every missing cover. Phase 1 limit;
/// per-section windowing needs the sections' content offsets.
pub(crate) fn dispatch_fav_albums_all_grouped(window: &AppWindow) {
    let Some(image_cache) = fav_albums_dispatch_ctx().lock().unwrap().clone() else {
        return;
    };
    let gen = FAV_ALBUMS_GEN.load(Ordering::SeqCst);
    let state = window.global::<FavoritesState>();
    let grouped = state.get_albums_grouped();
    let mut jobs = Vec::new();
    {
        let mut inflight = fav_albums_inflight().lock().unwrap();
        for gi in 0..grouped.row_count() {
            let Some(sec) = grouped.row_data(gi) else { continue };
            for i in 0..sec.albums.row_count() {
                let Some(item) = sec.albums.row_data(i) else { continue };
                if item.artwork.size().width > 0 || item.artwork_url.is_empty() {
                    continue;
                }
                let id = item.id.to_string();
                if inflight.insert(id.clone()) {
                    jobs.push(ArtworkJob {
                        target: ArtworkTarget::FavoriteAlbumById { id, gen },
                        url: item.artwork_url.to_string(),
                    });
                }
            }
        }
    }
    if !jobs.is_empty() {
        crate::artwork::spawn_loads(jobs, window.as_weak(), image_cache);
    }
}
