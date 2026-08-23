//! The big flattened-list builder: folders + their members + root
//! playlists + root locals. The per-folder step lives in `rebuild_folder.rs`.

use std::collections::HashSet;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, SidebarEntry, SidebarPlaylistItem, SidebarState};

use super::entry_build::{local_playlist_entry, playlist_entry};
use super::offline_filter::offline_visible;
use super::rebuild_folder::push_folder_entries;
use super::sort_search::sort_playlists;
use super::{LocalSidebarPlaylist, SidebarPlaylist, CACHE, EXPANDED, SEARCH};

/// Rebuild the flattened entries (+ the folders list for the
/// move-to-folder menu) from the cache + expand state, applying the
/// active sort and the playlist-name search filter.
pub fn rebuild(window: &AppWindow) {
    let data = CACHE.lock().map(|c| c.clone()).unwrap_or_default();
    let expanded = EXPANDED.lock().map(|e| e.clone()).unwrap_or_default();
    let query = SEARCH.lock().map(|q| q.clone()).unwrap_or_default();
    let searching = !query.is_empty();
    let offline = crate::offline_mode::engine().is_offline();
    let folder_ids: HashSet<&String> = data.folders.iter().map(|f| &f.id).collect();

    // Sort then filter by the playlist-name query (recursive — the same
    // filter applies to playlists nested in folders).
    let sorted = sort_playlists(&data.playlists);
    let matches = |p: &SidebarPlaylist| !searching || p.name.to_lowercase().contains(&query);
    let local_matches = |p: &LocalSidebarPlaylist| !searching || p.name.to_lowercase().contains(&query);

    let mut entries: Vec<SidebarEntry> = Vec::new();
    for folder in &data.folders {
        push_folder_entries(&mut entries, folder, &sorted, &data, offline, searching, &query, &expanded);
    }
    // Root playlists — no folder, or a folder that no longer exists.
    for p in &sorted {
        let in_folder = data
            .folder_map
            .get(&p.id)
            .map(|f| folder_ids.contains(f))
            .unwrap_or(false);
        if !in_folder
            && matches(p)
            && !data.hidden_playlists.contains(&p.id)
            && offline_visible(&data, offline, p)
        {
            entries.push(playlist_entry(p, false, ""));
        }
    }
    // LOCAL playlists (library.db, D7) NOT in a folder (or in one that no
    // longer exists) — root rows after the Qobuz set, name-sorted, honoring
    // the same search filter. Folder-assigned locals were already emitted
    // under their folder header above. Always present, online or offline.
    {
        let mut locals: Vec<&LocalSidebarPlaylist> = data
            .local_playlists
            .iter()
            .filter(|p| {
                let in_folder = p
                    .folder_id
                    .as_ref()
                    .map(|f| folder_ids.contains(f))
                    .unwrap_or(false);
                !in_folder && local_matches(p)
            })
            .collect();
        locals.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        for p in locals {
            entries.push(local_playlist_entry(p, false, ""));
        }
    }

    let folders: Vec<SidebarPlaylistItem> = data
        .folders
        .iter()
        .map(|f| SidebarPlaylistItem {
            id: f.id.clone().into(),
            name: f.name.clone().into(),
        })
        .collect();

    let state = window.global::<SidebarState>();
    state.set_entries(ModelRc::new(VecModel::from(entries)));
    state.set_folders(ModelRc::new(VecModel::from(folders.clone())));
    // The move-to-folder menu starts unfiltered (= the full folder set);
    // the search box narrows it via `search_menu_folders`.
    state.set_menu_folders(ModelRc::new(VecModel::from(folders)));
    state.set_loading(false);
}
