//! Artwork jobs for the loaded collection: hero mosaic + row thumbnails,
//! split by source so each routes through the right decoder.

use qbz_models::mixtape::AlbumSource;
use slint::{ComponentHandle, Model};

use super::FULL_ITEMS;
use crate::artwork::{self, ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, MyQbzDetailState};

/// The artwork jobs for the loaded collection, SPLIT by source so each is
/// dispatched through the correct decoder (spec §17 fallback chain): Qobuz items
/// carry an HTTP CDN url → the Remote/HTTP path (`spawn_loads`); local
/// items carry a filesystem path → the source-aware path
/// (`spawn_local_loads`). Mixing them (the old single `spawn_loads`) broke
/// local covers — a filesystem path was fetched as an HTTP url and failed
/// silently, leaving the row/hero cell blank.
#[derive(Default)]
pub struct ArtworkJobSplit {
    /// Qobuz CDN urls — HTTP fetch via the disk cache.
    pub remote: Vec<ArtworkJob>,
    /// Local filesystem paths — source-aware decode.
    pub local_or_plex: Vec<ArtworkJob>,
}

/// Build the (remote, local) artwork jobs for the loaded collection: the
/// up-to-9 hero-mosaic cells (only when no custom cover) + one thumbnail per
/// visible item row. Each job is routed to the `remote` bucket for Qobuz items
/// and the `local_or_plex` bucket otherwise.
pub fn artwork_jobs(window: &AppWindow) -> ArtworkJobSplit {
    let state = window.global::<MyQbzDetailState>();
    let mut split = ArtworkJobSplit::default();

    // Hero mosaic cells: classify each cell by the corresponding FULL_ITEMS
    // item's source (the cells map 1:1 to the first N items in original order).
    if !state.get_has_custom_cover() {
        let urls = [
            state.get_url1(),
            state.get_url2(),
            state.get_url3(),
            state.get_url4(),
            state.get_url5(),
            state.get_url6(),
            state.get_url7(),
            state.get_url8(),
            state.get_url9(),
        ];
        let cell_sources: Vec<AlbumSource> =
            FULL_ITEMS.with(|cell| cell.borrow().iter().map(|it| it.source).collect());
        for (slot, url) in urls.iter().enumerate() {
            if url.is_empty() {
                continue;
            }
            let job = ArtworkJob {
                target: ArtworkTarget::MyQbzDetailCover { slot },
                url: url.to_string(),
            };
            match cell_sources.get(slot) {
                Some(AlbumSource::Qobuz) => split.remote.push(job),
                _ => split.local_or_plex.push(job),
            }
        }
    }

    // Row thumbnails (the rendered model — matched back by position on apply).
    // Route by the row's resolved source-kind (qobuz -> remote; local ->
    // source-aware). A not-yet-resolved row defaults to its stored kind.
    let model = state.get_items();
    for i in 0..model.row_count() {
        let Some(item) = model.row_data(i) else { continue };
        if item.artwork_url.is_empty() {
            continue;
        }
        let job = ArtworkJob {
            target: ArtworkTarget::MyQbzDetailRow { position: item.position },
            url: item.artwork_url.to_string(),
        };
        if item.source_kind == "qobuz" {
            split.remote.push(job);
        } else {
            split.local_or_plex.push(job);
        }
    }
    split
}

/// Dispatch a built `ArtworkJobSplit` through the correct decoders: Qobuz CDN
/// urls via the HTTP path (`spawn_loads`), local paths via the source-aware
/// path (`spawn_local_loads`). The single entry point both `navigate` (initial
/// load) and the toolbar re-derive (`refresh_row_covers`) use, so the
/// source-split routing lives in ONE place.
pub fn dispatch_artwork(split: ArtworkJobSplit, weak: slint::Weak<AppWindow>, image_cache: ImageCache) {
    if !split.remote.is_empty() {
        artwork::spawn_loads(split.remote, weak.clone(), image_cache.clone());
    }
    if !split.local_or_plex.is_empty() {
        artwork::spawn_local_loads(split.local_or_plex, weak, image_cache);
    }
}

/// Set a decoded row thumbnail by item position (the rendered model order may
/// differ from FULL_ITEMS after a sort, so match by the stable position).
pub fn set_row_artwork(window: &AppWindow, position: i32, image: slint::Image) {
    let model = window.global::<MyQbzDetailState>().get_items();
    for i in 0..model.row_count() {
        if let Some(mut it) = model.row_data(i) {
            if it.position == position {
                it.artwork = image;
                model.set_row_data(i, it);
                break;
            }
        }
    }
}
