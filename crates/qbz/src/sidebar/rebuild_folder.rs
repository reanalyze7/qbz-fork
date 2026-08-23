//! The per-folder entry-building step of `rebuild()`: the folder header row
//! plus its Qobuz + local member rows (when expanded).

use std::collections::HashSet;

use crate::{folders::FolderInfo, SidebarEntry};

use super::entry_build::{local_playlist_entry, playlist_entry};
use super::offline_filter::offline_visible;
use super::{LocalSidebarPlaylist, SidebarData, SidebarPlaylist};

/// Build one folder's entries (header + members, appended to `entries`).
/// Skips the folder entirely (no header emitted) when searching/offline and
/// it has no matching members — mirrors Tauri's
/// `if (isSearching && folderPlaylists.length === 0) continue`.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_folder_entries(
    entries: &mut Vec<SidebarEntry>,
    folder: &FolderInfo,
    sorted: &[SidebarPlaylist],
    data: &SidebarData,
    offline: bool,
    searching: bool,
    query: &str,
    expanded: &HashSet<String>,
) {
    let matches = |p: &SidebarPlaylist| !searching || p.name.to_lowercase().contains(query);
    let local_matches = |p: &LocalSidebarPlaylist| !searching || p.name.to_lowercase().contains(query);

    let members: Vec<&SidebarPlaylist> = sorted
        .iter()
        .filter(|p| data.folder_map.get(&p.id).map(|f| f == &folder.id).unwrap_or(false))
        .filter(|p| matches(p))
        .filter(|p| !data.hidden_playlists.contains(&p.id))
        .filter(|p| offline_visible(data, offline, p))
        .collect();
    // Local playlists assigned to THIS folder, name-sorted.
    let mut local_members: Vec<&LocalSidebarPlaylist> = data
        .local_playlists
        .iter()
        .filter(|p| p.folder_id.as_deref() == Some(folder.id.as_str()))
        .filter(|p| local_matches(p))
        .collect();
    local_members.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    // While searching, skip folders with no matching playlists (mirrors
    // Tauri's `if (isSearching && folderPlaylists.length === 0) continue`).
    // Offline the same rule hides folders whose members are all filtered out
    // (D11.b) — an empty folder header carries no information there. Locals
    // count as members for both gates.
    if (searching || offline) && members.is_empty() && local_members.is_empty() {
        return;
    }
    // When searching, force-expand so matches inside are visible.
    let is_exp = searching || expanded.contains(&folder.id);
    entries.push(SidebarEntry {
        kind: "folder".into(),
        id: folder.id.clone().into(),
        name: folder.name.clone().into(),
        expanded: is_exp,
        count: (members.len() + local_members.len()) as i32,
        indent: false,
        folder_id: "".into(),
        local_kind: "".into(),
        cover_count: 0,
        url1: Default::default(),
        url2: Default::default(),
        url3: Default::default(),
        url4: Default::default(),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
    });
    if is_exp {
        for p in members {
            entries.push(playlist_entry(p, true, &folder.id));
        }
        for p in local_members {
            entries.push(local_playlist_entry(p, true, &folder.id));
        }
    }
}
