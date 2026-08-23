//! The blocking library.db + snapshot reads gathered alongside the Qobuz
//! fetch: folders, per-playlist settings/play-counts/local-counts, the
//! local-playlist list, and (offline only) the snapshot name/availability
//! maps.

use std::collections::{HashMap, HashSet};

use crate::folders::{FolderFull, PlaylistSettingsLite};

use super::super::types::PmLocalPlaylist;

pub(super) type Gathered = (
    Vec<FolderFull>,
    HashMap<u64, PlaylistSettingsLite>,
    HashMap<u64, u32>,
    HashMap<u64, u32>,
    Vec<PmLocalPlaylist>,
    HashMap<u64, (String, Option<u32>)>,
    HashSet<u64>,
);

/// BLOCKING — call inside `spawn_blocking`.
pub(super) fn gather_blocking() -> Gathered {
    let locals: Vec<PmLocalPlaylist> = crate::local_playlist::list_blocking()
        .into_iter()
        .map(|p| PmLocalPlaylist {
            id: p.id,
            name: p.name,
            offline_only: p.offline_only,
            track_count: p.track_count,
            is_favorite: p.favorite,
            is_hidden: p.hidden,
        })
        .collect();
    // B7/B8 (offline only): snapshot names for the synthesized entries + the
    // snapshot-available visibility set.
    let (snapshot_names, snapshot_available) = if crate::offline_mode::engine().is_offline() {
        (
            crate::playlist_snapshot::headers_blocking(),
            crate::playlist_snapshot::available_offline_blocking(),
        )
    } else {
        (HashMap::new(), HashSet::new())
    };
    (
        crate::folders::load_folders_full(),
        crate::folders::playlist_settings_map(),
        crate::folders::playlist_play_counts(),
        crate::folders::playlist_local_counts(),
        locals,
        snapshot_names,
        snapshot_available,
    )
}
