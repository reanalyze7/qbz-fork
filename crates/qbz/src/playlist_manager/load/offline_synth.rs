//! D11.b — OFFLINE: the Qobuz fetch is gate-refused (empty), so the
//! reachable playlists are synthesized locally: the MIXED ones (>= 1 local
//! sidecar row) plus — B8 — the snapshot-available ones (>= 1 cached
//! snapshot track).

use std::collections::{HashMap, HashSet};

use crate::folders::PlaylistSettingsLite;

use super::super::types::PmPlaylist;

#[allow(clippy::too_many_arguments)]
pub(super) fn synthesize_offline_playlists(
    playlists: &mut Vec<PmPlaylist>,
    folder_ids: &HashSet<&String>,
    settings: &HashMap<u64, PlaylistSettingsLite>,
    play_counts: &HashMap<u64, u32>,
    local_counts: &HashMap<u64, u32>,
    snapshot_names: &HashMap<u64, (String, Option<u32>)>,
    snapshot_available: &HashSet<u64>,
) {
    if !crate::offline_mode::engine().is_offline() {
        return;
    }
    let known: HashSet<u64> = playlists.iter().map(|p| p.id).collect();
    let mut ids: Vec<u64> = local_counts
        .iter()
        .filter(|&(_, &count)| count > 0)
        .map(|(&id, _)| id)
        .collect();
    for &id in snapshot_available {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    for id in ids {
        if known.contains(&id) {
            continue;
        }
        let count = local_counts.get(&id).copied().unwrap_or(0);
        let s = settings.get(&id).cloned().unwrap_or_default();
        let snapshot = snapshot_names.get(&id);
        // Names: the sidebar's session cache (loaded while online), else the
        // persisted snapshot (B7 — survives a cold offline start), else the
        // "Playlist (N local)" fallback.
        let name = crate::sidebar::playlist_name_desc(id)
            .map(|(name, _)| name)
            .or_else(|| snapshot.map(|(name, _)| name.clone()))
            .unwrap_or_else(|| qbz_i18n::t_args("Playlist ({} local)", &[&count.to_string()]));
        playlists.push(PmPlaylist {
            id,
            name,
            // The snapshot's point-in-time Qobuz total when known
            // ("# of tracks" sort + the card count line).
            tracks_count: snapshot.and_then(|(_, tc)| *tc).unwrap_or(0),
            duration: 0,
            local_count: count,
            play_count: play_counts.get(&id).copied().unwrap_or(0),
            is_favorite: s.is_favorite,
            is_hidden: s.hidden,
            folder_id: s.folder_id.filter(|fid| folder_ids.contains(fid)),
            position: s.position,
            cover_urls: Vec::new(),
            offline_available: snapshot_available.contains(&id),
        });
    }
}
