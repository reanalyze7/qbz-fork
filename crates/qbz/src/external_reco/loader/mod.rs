//! Loader entry points: `ensure_loaded` / `force_reload`, and the shared
//! loading-flag helpers the run loop latches.
use slint::ComponentHandle;

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::artwork::ImageCache;
use crate::{AppWindow, ExternalRecoState};

mod build;
mod cache_paint;
mod cache_write;
mod run;

pub fn ensure_loaded(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &ImageCache,
) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    if w.global::<ExternalRecoState>().get_loaded() {
        return;
    }
    w.global::<ExternalRecoState>().set_loading(true);
    run::spawn(runtime.clone(), weak.clone(), handle, image_cache.clone(), false);
}

/// Force a full rebuild of the Recommendations tab, bypassing the instant
/// results-cache paint (the "Refresh now" action). Resets the loaded/loading
/// latches and runs `spawn` with `force = true`, which skips the cache-read
/// early-return so every row is rebuilt and the results blob is overwritten.
pub fn force_reload(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &ImageCache,
) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let s = w.global::<ExternalRecoState>();
    s.set_loaded(false);
    s.set_loading(true);
    run::spawn(runtime.clone(), weak.clone(), handle, image_cache.clone(), true);
}

pub(super) fn latch_loaded(weak: &slint::Weak<AppWindow>) {
    let _ = weak.upgrade_in_event_loop(|w| {
        let s = w.global::<ExternalRecoState>();
        s.set_loading(false);
        s.set_loaded(true);
        // Defensive: every builder clears its own pending flag as it resolves;
        // this guarantees no skeleton can stick after the whole build settles.
        clear_all_pending(&w);
    });
}

/// Mark the rows the controller is about to build as pending, so their per-row
/// skeletons show immediately while the builders run.
pub(super) fn set_pending(w: &AppWindow, cold_start: bool) {
    let s = w.global::<ExternalRecoState>();
    if cold_start {
        s.set_pending_top_albums(true);
        s.set_pending_top_artists(true);
    } else {
        s.set_pending_rec_artists_common(true);
        s.set_pending_rec_artists_recent(true);
        s.set_pending_rec_albums(true);
        s.set_pending_fresh_releases(true);
        s.set_pending_weekly_exploration(true);
        s.set_pending_weekly_jams(true);
        s.set_pending_deep_cut_albums(true);
    }
}

fn clear_all_pending(w: &AppWindow) {
    let s = w.global::<ExternalRecoState>();
    s.set_pending_rec_artists_common(false);
    s.set_pending_rec_artists_recent(false);
    s.set_pending_rec_albums(false);
    s.set_pending_fresh_releases(false);
    s.set_pending_weekly_exploration(false);
    s.set_pending_weekly_jams(false);
    s.set_pending_deep_cut_albums(false);
    s.set_pending_top_albums(false);
    s.set_pending_top_artists(false);
}
