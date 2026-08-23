//! Windowed albums artwork (mirrors local_library's albums grid). Dispatches
//! covers only for the visible row band instead of the whole favorited-album
//! set, so cover RAM scales with the viewport instead of the library.

mod dispatch_all;
mod window;

pub use dispatch_all::albums_view_mode_changed;
pub use window::{albums_window_changed, dispatch_fav_albums_window};
pub(crate) use dispatch_all::{dispatch_fav_albums_all_grouped, dispatch_fav_albums_all_visible};

use std::sync::atomic::{AtomicU64, Ordering};

use crate::artwork::ImageCache;

/// Generation guard, bumped on every Albums-tab (re)load. A stale in-flight
/// cover fetch (an older load's job) is discarded on apply so it can't land
/// on the replacement set.
pub(crate) static FAV_ALBUMS_GEN: AtomicU64 = AtomicU64::new(0);

/// True if `gen` is still the current favorites-albums generation. The
/// artwork pipeline calls this before applying a decoded cover so an
/// in-flight job from a superseded load doesn't paint the new model.
pub fn albums_gen_current(gen: u64) -> bool {
    FAV_ALBUMS_GEN.load(Ordering::SeqCst) == gen
}

/// Cover ids currently in the artwork pipeline for the albums window
/// (dedupe during fast scroll). Freed on apply; cleared on reloads.
pub(crate) fn fav_albums_inflight() -> &'static std::sync::Mutex<std::collections::HashSet<String>> {
    static S: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    S.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

/// Image-cache handle captured at load time so the window dispatcher can
/// spawn artwork jobs outside the load path. Favorite covers are Qobuz CDN
/// URLs (plain `spawn_loads`).
pub(crate) fn fav_albums_dispatch_ctx() -> &'static std::sync::Mutex<Option<ImageCache>> {
    static S: std::sync::OnceLock<std::sync::Mutex<Option<ImageCache>>> = std::sync::OnceLock::new();
    S.get_or_init(|| std::sync::Mutex::new(None))
}

/// A windowed artwork job finished (applied or dropped) — free its slot so
/// the dispatcher can request the id again after an eviction.
pub fn album_artwork_job_done(id: &str) {
    fav_albums_inflight().lock().unwrap().remove(id);
}

/// Reset the windowed-albums artwork pipeline for a fresh Albums-tab load:
/// bump the generation (orphans every in-flight job — dropped on apply),
/// free their dedupe slots and stash the image-cache handle the dispatchers
/// spawn jobs with. Runs on the UI thread BEFORE `apply_favorites`, whose
/// `derive_albums` dispatches the covers against the new generation.
pub fn begin_albums_artwork(image_cache: ImageCache) {
    FAV_ALBUMS_GEN.fetch_add(1, Ordering::SeqCst);
    fav_albums_inflight().lock().unwrap().clear();
    *fav_albums_dispatch_ctx().lock().unwrap() = Some(image_cache);
}
