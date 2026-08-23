//! The collapsed-sidebar folder flyout.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, SidebarEntry, SidebarFolderPopupState};

use super::entry_build::{local_playlist_entry, playlist_entry};
use super::offline_filter::offline_visible;
use super::sort_search::sort_playlists;
use super::{LocalSidebarPlaylist, CACHE};

/// Populate the collapsed-sidebar folder flyout with `folder_id`'s playlists,
/// built from the cache so it works even for collapsed folders (whose
/// children are absent from the flattened `entries`).
pub fn load_folder_popup(window: &AppWindow, folder_id: &str) {
    let data = CACHE.lock().map(|c| c.clone()).unwrap_or_default();
    let offline = crate::offline_mode::engine().is_offline();
    let sorted = sort_playlists(&data.playlists);
    let mut entries: Vec<SidebarEntry> = sorted
        .iter()
        .filter(|p| {
            data.folder_map
                .get(&p.id)
                .map(|f| f.as_str() == folder_id)
                .unwrap_or(false)
        })
        .filter(|p| !data.hidden_playlists.contains(&p.id))
        .filter(|p| offline_visible(&data, offline, p))
        .map(|p| playlist_entry(p, true, folder_id))
        .collect();
    // Local playlists assigned to this folder, name-sorted, appended after.
    let mut local_members: Vec<&LocalSidebarPlaylist> = data
        .local_playlists
        .iter()
        .filter(|p| p.folder_id.as_deref() == Some(folder_id))
        .collect();
    local_members.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    for p in local_members {
        entries.push(local_playlist_entry(p, true, folder_id));
    }
    window
        .global::<SidebarFolderPopupState>()
        .set_playlists(ModelRc::new(VecModel::from(entries)));
}
