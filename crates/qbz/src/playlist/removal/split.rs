//! Split a removal row set by id namespace (Seam D).

use super::SelectedRow;

/// The removal split of a row set, by id namespace.
#[derive(Default)]
pub struct RemovalSplit {
    /// Qobuz rows resolved to `playlist_track_id`s — what the
    /// `playlist/deleteTracks` API actually takes. ALL instances of a
    /// selected catalog id resolve (duplicates removed together — Tauri
    /// behavior).
    pub playlist_track_ids: Vec<u64>,
    /// Local sidecar rows: `local_tracks.id` (the row's display id).
    pub local_track_ids: Vec<i64>,
}

/// Split rows for removal by id namespace (Seam D). Qobuz catalog ids
/// resolve to `playlist_track_id` through the `CURRENT` Track cache (the
/// loaded detail keeps it there; `TrackItem` drops it) — never ship a
/// TRACK id to `remove_tracks_from_playlist` (its parameter is
/// playlist_track_ids; the old bulk path did exactly that and silently
/// failed). Call on the UI thread while the detail is open.
pub fn split_for_removal(rows: &[SelectedRow]) -> RemovalSplit {
    let mut split = RemovalSplit::default();
    let mut qobuz_ids: Vec<u64> = Vec::new();
    for row in rows {
        match row.source.as_str() {
            "local" => match row.id.parse::<i64>() {
                Ok(rid) => split.local_track_ids.push(rid),
                Err(_) => {
                    log::warn!("[qbz-slint] remove: unresolvable local row id {}", row.id)
                }
            },
            _ => match row.id.parse::<u64>() {
                Ok(tid) => qobuz_ids.push(tid),
                Err(_) => {
                    log::warn!("[qbz-slint] remove: unresolvable row id {}", row.id)
                }
            },
        }
    }
    if !qobuz_ids.is_empty() {
        let id_set: std::collections::HashSet<u64> = qobuz_ids.iter().copied().collect();
        let mut resolved: std::collections::HashSet<u64> = std::collections::HashSet::new();
        if let Ok(cur) = super::super::apply::CURRENT.lock() {
            for track in cur.iter().filter(|t| id_set.contains(&t.id)) {
                match track.playlist_track_id {
                    Some(ptid) => {
                        split.playlist_track_ids.push(ptid);
                        resolved.insert(track.id);
                    }
                    None => {
                        log::warn!(
                            "[qbz-slint] remove: track {} has no playlist_track_id",
                            track.id
                        );
                    }
                }
            }
        }
        for tid in qobuz_ids {
            if !resolved.contains(&tid) {
                log::warn!(
                    "[qbz-slint] remove: track {tid} not resolvable to a playlist_track_id — skipped"
                );
            }
        }
    }
    split
}
