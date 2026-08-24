//! Navigation: open a My QBZ grid and load its rows.
use slint::ComponentHandle;

use qbz_models::mixtape::{CollectionKind, MixtapeCollection};

use crate::artwork::{self, ImageCache};
use crate::{AppWindow, ContentView, NavState};

use super::artwork_jobs::artwork_jobs;
use super::db::list_collections;
use super::offline::retain_available_offline;
use super::render::{apply, set_loading};
use super::Grid;

/// Open a My QBZ grid (Mixtapes or Collections) and load its rows from the
/// per-user library.db on a blocking worker, then render + spawn mosaic
/// artwork. `kind` selects the grid (Mixtape → Mixtapes; Collection → the
/// Collections grid, which displays collection + artist_collection).
pub fn navigate(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    kind: CollectionKind,
) {
    let grid = match kind {
        CollectionKind::Mixtape => Grid::Mixtapes,
        _ => Grid::Collections,
    };
    let view = match grid {
        Grid::Mixtapes => ContentView::Mixtapes,
        Grid::Collections => ContentView::Collections,
    };

    handle.clone().spawn(async move {
        let _ = weak.upgrade_in_event_loop(move |w| {
            set_loading(&w, true);
            w.global::<NavState>().set_view(view);
        });

        // The Mixtapes grid wants kind == mixtape; the Collections grid loads
        // ALL kinds and filters locally (collection | artist_collection) so the
        // kind-filter dropdown can switch between them without a refetch.
        let kind_arg = match grid {
            Grid::Mixtapes => Some(CollectionKind::Mixtape),
            Grid::Collections => None,
        };
        let rows = tokio::task::spawn_blocking(move || list_collections(kind_arg))
            .await
            .unwrap_or_default();

        // For the Collections grid, drop mixtapes (load returned all kinds).
        let rows: Vec<MixtapeCollection> = match grid {
            Grid::Mixtapes => rows,
            Grid::Collections => rows
                .into_iter()
                .filter(|c| c.kind != CollectionKind::Mixtape)
                .collect(),
        };

        // D11.c — OFFLINE: unavailable items hide and a collection whose
        // items are ALL unavailable leaves the grid. Online: untouched.
        let rows = if crate::offline_mode::engine().is_offline() {
            retain_available_offline(rows).await
        } else {
            rows
        };

        let _ = weak.upgrade_in_event_loop(move |w| {
            apply(&w, grid, rows);
            let jobs = artwork_jobs(&w, grid);
            artwork::spawn_loads(jobs, w.as_weak(), image_cache.clone());
        });
    });
}
