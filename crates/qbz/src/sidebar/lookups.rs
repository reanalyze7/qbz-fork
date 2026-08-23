//! Small read-only accessors + trivial setters.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, SidebarPlaylistItem, SidebarState};

use super::{CACHE, NAME_DESC};

/// Name + description + offline_only for a LOCAL playlist id, from the
/// last loaded sidebar cache (the local twin of `playlist_name_desc`).
pub fn local_playlist_meta(id: &str) -> Option<(String, String, bool)> {
    CACHE.lock().ok().and_then(|c| {
        c.local_playlists
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.name.clone(), p.description.clone(), p.offline_only))
    })
}

/// Name + description for `id`, from the last loaded playlist payload.
/// Used by the sidebar context menu to prefill the edit-playlist modal
/// without a refetch. Returns None when the playlist is unknown.
pub fn playlist_name_desc(id: u64) -> Option<(String, String)> {
    NAME_DESC.lock().ok().and_then(|nd| nd.get(&id).cloned())
}

/// Total track count for `id`, from the last loaded playlist cache. Used by the
/// sidebar "Add to Mixtape/Collection" context action to populate the AddItem
/// `track_count` (the SidebarEntry struct doesn't carry it). Returns None when
/// the playlist is unknown.
pub fn playlist_track_count(id: u64) -> Option<u32> {
    CACHE
        .lock()
        .ok()
        .and_then(|c| c.playlists.iter().find(|p| p.id == id).map(|p| p.tracks_count))
}

/// Highlight the open playlist in the sidebar (or clear with "").
pub fn set_active(window: &AppWindow, id: &str) {
    window.global::<SidebarState>().set_active_id(id.into());
}

/// Filter the move-to-folder menu's folder list by a case-insensitive
/// substring of the search query (Slint strings have no `contains`, so
/// this lives in Rust). An empty query restores the full list.
pub fn search_menu_folders(window: &AppWindow, query: &str) {
    let q = query.trim().to_lowercase();
    let data = CACHE.lock().map(|c| c.clone()).unwrap_or_default();
    let filtered: Vec<SidebarPlaylistItem> = data
        .folders
        .iter()
        .filter(|f| q.is_empty() || f.name.to_lowercase().contains(&q))
        .map(|f| SidebarPlaylistItem {
            id: f.id.clone().into(),
            name: f.name.clone().into(),
        })
        .collect();
    window
        .global::<SidebarState>()
        .set_menu_folders(ModelRc::new(VecModel::from(filtered)));
}
