//! The render/rebuild pipeline: cache -> filtered/sorted Slint models,
//! pushed onto `PlaylistManagerState`.

mod folder_counts;
mod tree;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, PlaylistManagerState, PmFolderItem, PmPlaylistItem};

use super::build::folder_item;
use super::sort_filter::{local_entries, passes, sort_entries, PmEntry};
use super::types::{PmData, CACHE, EXPANDED, TREE_INIT};
use folder_counts::{folder_counts, folder_items};
use tree::build_tree;

pub fn set_loading(window: &AppWindow, loading: bool) {
    window.global::<PlaylistManagerState>().set_loading(loading);
}

/// Store freshly-loaded data and render it.
pub fn apply(window: &AppWindow, data: PmData) {
    if let Ok(mut c) = CACHE.lock() {
        *c = data;
    }
    rebuild(window);
}

/// Reset the per-session tree-expand init so a fresh navigation
/// re-expands folders on first tree open.
pub fn reset_session(_window: &AppWindow) {
    if let Ok(mut init) = TREE_INIT.lock() {
        *init = false;
    }
}

/// Rebuild the visible grid/list model + the folder model + the tree
/// from the cache, honoring the active toolbar state. UI thread only.
pub fn rebuild(window: &AppWindow) {
    let data = CACHE.lock().map(|c| c.clone()).unwrap_or_default();
    let state = window.global::<PlaylistManagerState>();
    let query = state.get_search_query().trim().to_lowercase();
    let filter = state.get_filter().to_string();
    let sort = state.get_sort().to_string();
    let view_mode = state.get_view_mode().to_string();
    let folder_mode = state.get_folder_mode();

    let counts = folder_counts(&data);
    let folders = folder_items(&data, &counts);

    // Filtered + sorted playlists for the grid / list. While OFFLINE only
    // the MIXED (>= 1 local sidecar track) and snapshot-available (>= 1
    // cached snapshot track, B8) playlists stay (D11.b).
    let offline = crate::offline_mode::engine().is_offline();
    let filtered: Vec<_> = data
        .playlists
        .iter()
        .filter(|p| !offline || p.local_count > 0 || p.offline_available)
        .filter(|p| passes(p, &query, &filter, folder_mode, &view_mode))
        .cloned()
        .collect();
    // LOCAL playlists (library.db, D7) interleave into the SAME sort as the
    // Qobuz set (B4) — see `PmEntry` for the missing-stat sort rules.
    let mut entries: Vec<PmEntry> = filtered.iter().map(PmEntry::Qobuz).collect();
    entries.extend(local_entries(&data, &query, &filter).into_iter().map(PmEntry::Local));
    sort_entries(&mut entries, &sort);
    let playlist_items: Vec<PmPlaylistItem> = entries.iter().map(|e| e.item()).collect();
    let visible_count = playlist_items.len();

    // Tree rows (folder headers + nested + root playlists). Built only
    // when the tree view is active; otherwise an empty model.
    let tree = if folder_mode && view_mode == "tree" {
        build_tree(&data, &query, &filter, &sort)
    } else {
        Vec::new()
    };

    // The list-row move-to-folder menu starts unfiltered (= the full set);
    // its search box narrows it via `search_menu_folders`.
    let menu_folders = folder_items(&data, &counts);

    state.set_folders(ModelRc::new(VecModel::from(folders)));
    state.set_menu_folders(ModelRc::new(VecModel::from(menu_folders)));
    state.set_playlists(ModelRc::new(VecModel::from(playlist_items)));
    state.set_tree(ModelRc::new(VecModel::from(tree)));
    state.set_folder_count(data.folders.len() as i32);
    state.set_playlist_count(visible_count as i32);
    state.set_can_reorder(sort == "custom" && query.is_empty());
    state.set_loading(false);
}

/// Filter the list-row move-to-folder menu's folder list by a
/// case-insensitive substring (Slint strings have no `contains`, so this
/// lives in Rust). An empty query restores the full list. Counts mirror the
/// `rebuild` computation. UI thread only.
pub fn search_menu_folders(window: &AppWindow, query: &str) {
    let q = query.trim().to_lowercase();
    let data = CACHE.lock().map(|c| c.clone()).unwrap_or_default();
    let counts = folder_counts(&data);

    let filtered: Vec<PmFolderItem> = data
        .folders
        .iter()
        .filter(|f| q.is_empty() || f.name.to_lowercase().contains(&q))
        .map(|f| folder_item(f, counts.get(&f.id).copied().unwrap_or(0)))
        .collect();

    window
        .global::<PlaylistManagerState>()
        .set_menu_folders(ModelRc::new(VecModel::from(filtered)));
}

/// Toggle a tree folder's expand state, then rebuild (cheap, from cache).
pub fn toggle_tree_folder(window: &AppWindow, folder_id: &str) {
    if let Ok(mut exp) = EXPANDED.lock() {
        if !exp.remove(folder_id) {
            exp.insert(folder_id.to_string());
        }
    }
    rebuild(window);
}
