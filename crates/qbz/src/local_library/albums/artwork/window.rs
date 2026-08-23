//! Windowed dispatch bookkeeping: band tracking, in-flight dedupe, and the
//! image-cache handle captured at load time.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Model};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, LocalLibraryState};

use crate::local_library::albums::load::ALBUMS_GEN;

/// Last row band reported by the windowed albums grid (item indices into
/// `albums-visible`, prefetch margin already included by the grid). Kept so
/// model rebuilds (load/derive) can re-dispatch — the grid only fires its
/// callback when the band CHANGES, not when the rows under it change.
static ALBUMS_WINDOW: std::sync::Mutex<(usize, usize)> = std::sync::Mutex::new((0, 59));

/// Cover ids currently in the artwork pipeline for the albums window
/// (dedupe during fast scroll). Freed on apply; cleared on reloads.
pub(crate) fn albums_inflight() -> &'static std::sync::Mutex<std::collections::HashSet<String>> {
    static S: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    S.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

/// Image-cache handle captured at load time so the window dispatcher can
/// spawn artwork jobs outside the load path.
pub(crate) fn albums_dispatch_ctx() -> &'static std::sync::Mutex<Option<ImageCache>> {
    static S: std::sync::OnceLock<std::sync::Mutex<Option<ImageCache>>> = std::sync::OnceLock::new();
    S.get_or_init(|| std::sync::Mutex::new(None))
}

/// A windowed artwork job finished (applied or dropped) — free its slot so
/// the dispatcher can request the id again after an eviction.
pub fn album_artwork_job_done(id: &str) {
    albums_inflight().lock().unwrap().remove(id);
}

/// Dispatch throttle for the grid's band reports (leading + trailing edge,
/// UI thread). During a fling the grid crosses a row boundary every ~270px
/// and each crossing used to spawn artwork jobs for rows flying straight
/// past the viewport; coalescing to one dispatch per interval keeps the
/// decode pipeline on rows the user can actually see.
const ALBUMS_DISPATCH_THROTTLE_MS: u64 = 180;
thread_local! {
    static ALBUMS_BAND: crate::viewport::BandDispatcher =
        crate::viewport::BandDispatcher::new(ALBUMS_DISPATCH_THROTTLE_MS);
}

/// The windowed grid reported a new visible row band. The band is stored
/// immediately (model rebuilds re-read it); the artwork dispatch is
/// throttled, and gen-guarded so a pass scheduled before a reload cannot
/// dispatch/evict against the replacement model — the reload's own
/// `derive_albums` dispatch is authoritative for the new generation.
pub fn albums_window_changed(window: &AppWindow, first: i32, last: i32) {
    let first = first.max(0) as usize;
    let last = last.max(first as i32) as usize;
    *ALBUMS_WINDOW.lock().unwrap() = (first, last);
    let gen = ALBUMS_GEN.load(Ordering::SeqCst);
    let weak = window.as_weak();
    ALBUMS_BAND.with(|d| {
        d.report(Box::new(move || {
            if !crate::local_library::albums_gen_current(gen) {
                return;
            }
            if let Some(w) = weak.upgrade() {
                dispatch_albums_window(&w);
            }
        }));
    });
}

/// Dispatch covers for the current albums window (over `albums-visible`) and
/// evict decoded covers far outside it back to the placeholder, so cover RAM
/// scales with the viewport instead of the library. Delivery is id-keyed
/// (`LocalAlbumById`), so a derive re-sort between dispatch and apply cannot
/// land a cover on the wrong card; a full reload bumps `ALBUMS_GEN` and the
/// apply arm drops the stale image.
pub fn dispatch_albums_window(window: &AppWindow) {
    let (first, last) = *ALBUMS_WINDOW.lock().unwrap();
    let Some(image_cache) = albums_dispatch_ctx().lock().unwrap().clone() else {
        return;
    };
    let gen = ALBUMS_GEN.load(Ordering::SeqCst);
    let s = window.global::<LocalLibraryState>();
    let visible = s.get_albums_visible();
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
    // decoded cache, else a bounded 264px re-decode from the source file).
    let span = last - first + 1;
    let keep_lo = first.saturating_sub(span);
    let keep_hi = (last + span).min(len - 1);
    let mut jobs = Vec::new();
    {
        let mut inflight = albums_inflight().lock().unwrap();
        for vi in first..=last {
            let Some(item) = visible.row_data(vi) else { continue };
            if item.artwork.size().width > 0 || item.artwork_url.is_empty() {
                continue;
            }
            let id = item.id.to_string();
            if inflight.insert(id.clone()) {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::LocalAlbumById { id, gen },
                    url: item.artwork_url.to_string(),
                });
            }
        }
    }
    for vi in (0..keep_lo).chain(keep_hi + 1..len) {
        let Some(item) = visible.row_data(vi) else { continue };
        if item.artwork.size().width > 0 {
            super::set_local_album_artwork(window, item.id.as_str(), slint::Image::default());
        }
    }
    if !jobs.is_empty() {
        crate::artwork::spawn_local_loads(jobs, window.as_weak(), image_cache);
    }
}
