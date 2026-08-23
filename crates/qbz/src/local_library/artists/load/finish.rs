//! UI-thread completion of an artists load: apply rows, satisfy a pending
//! open-artist, seed portrait decode jobs, kick the capped Qobuz fetch.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, Model};

use crate::adapter::SlintAdapter;
use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, LocalLibraryState};

use crate::local_library::artists::derive::apply_artists;
use crate::local_library::artists::images::fetch_missing_artist_images;
use crate::local_library::artists::merge::ArtistRow;
use crate::local_library::artists::normalize::normalize_artist;
use crate::local_library::artists::select::select_local_artist;
use crate::local_library::artists::state::take_pending_artist;

/// Apply the merged rows, satisfy a pending open-artist, and seed/kick the
/// portrait pipeline. Runs on the UI thread (called from an event-loop hop).
pub(crate) fn finish_artists_load(
    w: &AppWindow,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    gen: u64,
    items: Vec<ArtistRow>,
) {
    apply_artists(w, items);
    // Satisfy a pending open-artist now that the set + ARTIST_ALBUMS
    // cache are loaded (navigated here from a "Go to artist" link).
    if let Some(name) = take_pending_artist() {
        select_local_artist(w.as_weak(), handle.clone(), image_cache.clone(), name);
    }
    // Seed decode jobs for rows that already carry an image-path.
    // Non-http paths are local files: routed through the
    // source-aware dispatcher.
    let s = w.global::<LocalLibraryState>();
    let artists = s.get_artists();
    let mut local_jobs = Vec::new();
    let mut http_jobs = Vec::new();
    for i in 0..artists.row_count() {
        if let Some(a) = artists.row_data(i) {
            let p = a.image_path.to_string();
            if p.is_empty() {
                continue;
            }
            let job = ArtworkJob {
                target: ArtworkTarget::LocalArtistRowImage { index: i, gen },
                url: p.clone(),
            };
            if p.starts_with("http") {
                http_jobs.push(job);
            } else {
                local_jobs.push(job);
            }
        }
    }
    crate::artwork::spawn_local_loads(local_jobs, w.as_weak(), image_cache.clone());
    crate::artwork::spawn_loads(http_jobs, w.as_weak(), image_cache.clone());
    // Kick the capped Qobuz portrait fetch for missing rows. Snapshot
    // the names HERE (UI thread, sync) — fetch_missing_artist_images
    // must NOT block the event loop to read the model.
    if s.get_artists_fetch_images() {
        let mut names = Vec::new();
        for i in 0..artists.row_count() {
            if let Some(a) = artists.row_data(i) {
                if a.image_path.is_empty()
                    && normalize_artist(&a.name.to_string()) != "various artists"
                {
                    names.push(a.name.to_string());
                }
            }
        }
        s.set_artists_images_fetching(true);
        s.set_artists_images_fetched(0);
        fetch_missing_artist_images(runtime, w.as_weak(), handle, image_cache, gen, names);
    }
}
