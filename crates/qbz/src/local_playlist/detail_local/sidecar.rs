use crate::local_playlist::row::{LoadedRow, RowItem};

/// Read + resolve a QOBUZ playlist's SIDECAR rows (`playlist_local_tracks`)
/// with their stored absolute positions — the shared reader behind the
/// offline mixed detail
/// ([`crate::local_playlist::detail_offline_mixed::navigate_qobuz_offline`])
/// and the ONLINE mixed detail (`playlist::load`). Runs the one-shot position
/// healing first (Seam C: collided slots — the legacy 0-based picker/drag
/// writes renumber stably into the append region; drift alone is never
/// touched, E7). Blocking — run on a worker thread.
pub fn read_sidecar_rows_blocking(playlist_id: u64, qobuz_track_count: u32) -> Vec<LoadedRow> {
    crate::library_db::with_db(|db| {
        match db.heal_playlist_sidecar_positions(playlist_id, qobuz_track_count) {
            Ok(healed) => {
                for entry in &healed {
                    log::warn!(
                        "[qbz-slint] playlist {playlist_id}: healed sidecar position collision — {entry}"
                    );
                }
            }
            Err(e) => {
                // Healing is best-effort; the merge tolerates collisions
                // (same-slot rows all emit) so reading still proceeds.
                log::warn!("[qbz-slint] playlist {playlist_id}: sidecar healing failed: {e}");
            }
        }
        let rows: Vec<LoadedRow> = db
            .get_playlist_local_tracks_with_position(playlist_id)?
            .into_iter()
            .map(|r| LoadedRow {
                position: r.playlist_position,
                item: RowItem::Local(Box::new(r.track)),
            })
            .collect();
        Ok(rows)
    })
    .unwrap_or_default()
}
