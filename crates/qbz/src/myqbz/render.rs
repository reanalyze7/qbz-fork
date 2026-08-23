//! Apply / rebuild — the Slint-model read/write layer. Owns the per-grid
//! caches (mirrors `playlist_manager::CACHE`) so toolbar changes rebuild
//! without a DB refetch.

use std::sync::{LazyLock, Mutex};

use qbz_models::mixtape::{CollectionKind, MixtapeCollection};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, MixtapeCardItem, MyQbzState};

use super::card::card_item;
use super::sort_filter::{passes_search, sort_collections};
use super::Grid;

/// Last-loaded data per kind-group (so toolbar changes rebuild from cache,
/// no DB refetch). Mirrors `playlist_manager::CACHE`.
pub(super) static MIXTAPES_CACHE: LazyLock<Mutex<Vec<MixtapeCollection>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
pub(super) static COLLECTIONS_CACHE: LazyLock<Mutex<Vec<MixtapeCollection>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub fn set_loading(window: &AppWindow, loading: bool) {
    window.global::<MyQbzState>().set_loading(loading);
}

/// Store freshly-loaded rows for `grid` and render them through the active
/// toolbar state.
pub fn apply(window: &AppWindow, grid: Grid, rows: Vec<MixtapeCollection>) {
    match grid {
        Grid::Mixtapes => {
            if let Ok(mut c) = MIXTAPES_CACHE.lock() {
                *c = rows;
            }
        }
        Grid::Collections => {
            if let Ok(mut c) = COLLECTIONS_CACHE.lock() {
                *c = rows;
            }
        }
    }
    rebuild(window, grid);
}

/// Rebuild the visible card model for one grid from its cache, honoring the
/// active toolbar state (search / sort / kind-filter). UI thread only.
pub fn rebuild(window: &AppWindow, grid: Grid) {
    let state = window.global::<MyQbzState>();
    match grid {
        Grid::Mixtapes => {
            let data = MIXTAPES_CACHE.lock().map(|c| c.clone()).unwrap_or_default();
            let query = state.get_mix_search().trim().to_lowercase();
            let sort = state.get_mix_sort().to_string();
            let dir = state.get_mix_sort_dir().to_string();
            let mut filtered: Vec<MixtapeCollection> =
                data.into_iter().filter(|c| passes_search(c, &query)).collect();
            sort_collections(&mut filtered, &sort, &dir);
            let items: Vec<MixtapeCardItem> = filtered.iter().map(card_item).collect();
            state.set_mixtapes(ModelRc::new(VecModel::from(items)));
        }
        Grid::Collections => {
            let data = COLLECTIONS_CACHE.lock().map(|c| c.clone()).unwrap_or_default();
            let query = state.get_col_search().trim().to_lowercase();
            let sort = state.get_col_sort().to_string();
            let dir = state.get_col_sort_dir().to_string();
            let kind_filter = state.get_col_kind_filter().to_string();
            let mut filtered: Vec<MixtapeCollection> = data
                .into_iter()
                .filter(|c| match kind_filter.as_str() {
                    "collection" => c.kind == CollectionKind::Collection,
                    "artist_collection" => c.kind == CollectionKind::ArtistCollection,
                    _ => true,
                })
                .filter(|c| passes_search(c, &query))
                .collect();
            sort_collections(&mut filtered, &sort, &dir);
            let items: Vec<MixtapeCardItem> = filtered.iter().map(card_item).collect();
            state.set_collections(ModelRc::new(VecModel::from(items)));
        }
    }
    state.set_loading(false);
}

/// Re-clicking the active sort field flips direction; a new field resets to
/// asc. Mirrors `selectSort`.
pub fn set_sort(window: &AppWindow, grid: Grid, field: &str) {
    let state = window.global::<MyQbzState>();
    let (cur_sort, cur_dir) = match grid {
        Grid::Mixtapes => (state.get_mix_sort().to_string(), state.get_mix_sort_dir().to_string()),
        Grid::Collections => (state.get_col_sort().to_string(), state.get_col_sort_dir().to_string()),
    };
    let new_dir = if cur_sort == field {
        if cur_dir == "asc" { "desc" } else { "asc" }
    } else {
        "asc"
    };
    match grid {
        Grid::Mixtapes => {
            state.set_mix_sort(field.into());
            state.set_mix_sort_dir(new_dir.into());
        }
        Grid::Collections => {
            state.set_col_sort(field.into());
            state.set_col_sort_dir(new_dir.into());
        }
    }
    rebuild(window, grid);
}

/// Reset toolbar filters/sort (search too, like Tauri's `resetFilters`).
pub fn reset(window: &AppWindow, grid: Grid) {
    let state = window.global::<MyQbzState>();
    match grid {
        Grid::Mixtapes => {
            state.set_mix_sort("position".into());
            state.set_mix_sort_dir("asc".into());
            state.set_mix_search("".into());
        }
        Grid::Collections => {
            state.set_col_sort("position".into());
            state.set_col_sort_dir("asc".into());
            state.set_col_kind_filter("all".into());
            state.set_col_search("".into());
        }
    }
    rebuild(window, grid);
}
