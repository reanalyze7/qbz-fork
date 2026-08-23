use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Model};

use super::{
    albums_gen_current, fav_albums_dispatch_ctx, fav_albums_inflight, FAV_ALBUMS_GEN,
};
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, FavoritesState};

/// Last row band reported by the windowed albums grid (item indices into
/// `albums-visible`, prefetch margin already included by the grid). Kept so
/// model rebuilds (load/derive) can re-dispatch — the grid only fires its
/// callback when the band CHANGES, not when the rows under it change.
static FAV_ALBUMS_WINDOW: std::sync::Mutex<(usize, usize)> = std::sync::Mutex::new((0, 59));

/// Dispatch throttle for the grid's band reports (leading + trailing edge,
/// UI thread). During a fling the grid crosses a row boundary every ~270px
/// and each crossing used to spawn artwork jobs for rows flying straight
/// past the viewport; coalescing to one dispatch per interval keeps the
/// decode pipeline on rows the user can actually see.
const FAV_ALBUMS_DISPATCH_THROTTLE_MS: u64 = 180;
thread_local! {
    static FAV_ALBUMS_BAND: crate::viewport::BandDispatcher =
        crate::viewport::BandDispatcher::new(FAV_ALBUMS_DISPATCH_THROTTLE_MS);
}

/// The windowed grid reported a new visible row band. The band is stored
/// immediately (model rebuilds re-read it); the artwork dispatch is
/// throttled, and gen-guarded so a pass scheduled before a reload cannot
/// dispatch/evict against the replacement model — the reload's own
/// `derive_albums` dispatch is authoritative for the new generation.
pub fn albums_window_changed(window: &AppWindow, first: i32, last: i32) {
    let first = first.max(0) as usize;
    let last = last.max(first as i32) as usize;
    *FAV_ALBUMS_WINDOW.lock().unwrap() = (first, last);
    let gen = FAV_ALBUMS_GEN.load(Ordering::SeqCst);
    let weak = window.as_weak();
    FAV_ALBUMS_BAND.with(|d| {
        d.report(Box::new(move || {
            if !albums_gen_current(gen) {
                return;
            }
            if let Some(w) = weak.upgrade() {
                dispatch_fav_albums_window(&w);
            }
        }));
    });
}

/// Dispatch covers for the current albums window (over `albums-visible`) and
/// evict decoded covers far outside it back to the placeholder, so cover RAM
/// scales with the viewport instead of the library. Delivery is id-keyed
/// (`FavoriteAlbumById`), so a derive re-sort between dispatch and apply
/// cannot land a cover on the wrong card; a tab reload bumps
/// `FAV_ALBUMS_GEN` and the apply arm drops the stale image.
pub fn dispatch_fav_albums_window(window: &AppWindow) {
    let (first, last) = *FAV_ALBUMS_WINDOW.lock().unwrap();
    let Some(image_cache) = fav_albums_dispatch_ctx().lock().unwrap().clone() else {
        return;
    };
    let gen = FAV_ALBUMS_GEN.load(Ordering::SeqCst);
    let state = window.global::<FavoritesState>();
    let visible = state.get_albums_visible();
    let len = visible.row_count();
    if len == 0 {
        return;
    }
    let last = last.min(len - 1);
    if first > last {
        return;
    }
    // Retention = the window plus one window-span on each side. Beyond it,
    // covers return to the placeholder; re-entry is cheap (byte-budgeted
    // decoded cache, else a bounded re-decode through the disk cache).
    let span = last - first + 1;
    let keep_lo = first.saturating_sub(span);
    let keep_hi = (last + span).min(len - 1);
    let mut jobs = Vec::new();
    {
        let mut inflight = fav_albums_inflight().lock().unwrap();
        for vi in first..=last {
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
    for vi in (0..keep_lo).chain(keep_hi + 1..len) {
        let Some(item) = visible.row_data(vi) else { continue };
        if item.artwork.size().width > 0 {
            crate::favorites::artwork_apply::set_album_artwork(
                window,
                item.id.as_str(),
                slint::Image::default(),
            );
        }
    }
    if !jobs.is_empty() {
        crate::artwork::spawn_loads(jobs, window.as_weak(), image_cache);
    }
}
