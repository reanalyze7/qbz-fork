//! The blocking-DB half of `load()`: folders, folder membership,
//! custom-sort positions, hidden-playlist set, local playlists, local
//! sidecar counts, and (offline only) the playlist-snapshot names +
//! availability set.

use std::collections::{HashMap, HashSet};

use crate::folders::FolderInfo;

use super::LocalSidebarPlaylist;

#[allow(clippy::type_complexity)]
pub(super) type MetaTuple = (
    Vec<FolderInfo>,
    HashMap<u64, String>,
    HashMap<u64, i32>,
    HashSet<u64>,
    Vec<LocalSidebarPlaylist>,
    HashMap<u64, u32>,
    HashMap<u64, (String, Option<u32>)>,
    HashSet<u64>,
);

/// Runs on a blocking worker (`tokio::task::spawn_blocking`) — every read
/// here is a synchronous `library.db` / settings-store access.
pub(super) fn load_folders_and_locals() -> MetaTuple {
    let folders: Vec<FolderInfo> = crate::folders::load_folders_full()
        .into_iter()
        .filter(|f| !f.is_hidden)
        .map(|f| FolderInfo {
            id: f.id,
            name: f.name,
        })
        .collect();
    let hidden_playlists: HashSet<u64> = crate::folders::playlist_settings_map()
        .into_iter()
        .filter(|(_, s)| s.hidden)
        .map(|(id, _)| id)
        .collect();
    // Hidden locals drop from the sidebar (B3) the way hidden Qobuz
    // playlists do — they stay reachable via the manager's "hidden"
    // filter, which reads the repo list directly.
    let local_playlists: Vec<LocalSidebarPlaylist> = crate::local_playlist::list_blocking()
        .into_iter()
        .filter(|p| !p.hidden)
        .map(|p| LocalSidebarPlaylist {
            id: p.id,
            name: p.name,
            description: p.description.unwrap_or_default(),
            offline_only: p.offline_only,
            folder_id: p.folder_id,
            // Resolved by the caller (async, off this blocking closure).
            cover_urls: Vec::new(),
        })
        .collect();
    // B7/B8 (offline only): the snapshot names replace the synthesized
    // "Playlist (N local)" fallback, and the snapshot-available set extends
    // the D11.b visibility filter.
    let (snapshot_names, snapshot_available) = if crate::offline_mode::engine().is_offline() {
        (
            crate::playlist_snapshot::headers_blocking(),
            crate::playlist_snapshot::available_offline_blocking(),
        )
    } else {
        (HashMap::new(), HashSet::new())
    };
    (
        folders,
        crate::folders::playlist_folder_map(),
        crate::folders::playlist_positions(),
        hidden_playlists,
        local_playlists,
        crate::folders::playlist_local_counts(),
        snapshot_names,
        snapshot_available,
    )
}
