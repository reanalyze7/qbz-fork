//! D11.b/B7/B8 offline-playlist synthesis: while offline the Qobuz fetch
//! above is gate-refused (empty), so the reachable playlists are
//! synthesized locally: the MIXED ones (>= 1 local sidecar row) plus — B8 —
//! the snapshot-available ones (>= 1 cached snapshot track). Names come from
//! the previous load's session cache, else the persisted snapshot (B7 —
//! survives a cold offline start), else the synthesized "Playlist (N local)"
//! fallback.

use std::collections::{HashMap, HashSet};

use super::{SidebarPlaylist, NAME_DESC};

pub(super) fn synthesize_offline_playlists(
    playlists: &mut Vec<SidebarPlaylist>,
    local_counts: &HashMap<u64, u32>,
    snapshot_available: &HashSet<u64>,
    snapshot_names: &HashMap<u64, (String, Option<u32>)>,
) {
    let known: HashSet<u64> = playlists.iter().map(|p| p.id).collect();
    let prior: HashMap<u64, (String, String)> =
        NAME_DESC.lock().map(|nd| nd.clone()).unwrap_or_default();
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
        let snapshot = snapshot_names.get(&id);
        let (name, description) = prior
            .get(&id)
            .cloned()
            .or_else(|| snapshot.map(|(name, _)| (name.clone(), String::new())))
            .unwrap_or_else(|| {
                (
                    qbz_i18n::t_args("Playlist ({} local)", &[&count.to_string()]),
                    String::new(),
                )
            });
        // Track count: the snapshot's point-in-time Qobuz total when
        // known (the "# of tracks" sort key), else the local count.
        let tracks_count = snapshot.and_then(|(_, tc)| *tc).unwrap_or(count);
        playlists.push(SidebarPlaylist {
            id,
            name,
            description,
            tracks_count,
            cover_urls: Vec::new(),
            position: 0,
        });
    }
}
