use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

#[cfg(test)]
mod tests;

impl LibraryDatabase {
    /// Next append position for a local sidecar add to a Qobuz playlist.
    ///
    /// Tauri's convention is `qobuz_count + sidecar_count` — append after the
    /// whole merged list — but that formula re-issues positions after a
    /// removal (stored positions keep their gaps while counts shrink; Tauri
    /// bug T3), which collides in the absolute-slot interleave and silently
    /// loses rows. The fix-forward rule is the MAX of both worlds:
    ///
    /// `max(qobuz_count + local_count, MAX(position) + 1)`
    ///
    /// so an add always lands after the merged end AND past every stored
    /// position. Batch adds take this once and assign `next + i` per row.
    pub fn next_playlist_sidecar_position(
        &self,
        qobuz_playlist_id: u64,
        qobuz_track_count: u32,
    ) -> Result<i32, LibraryError> {
        let local_count = self.get_playlist_local_track_count(qobuz_playlist_id)?;
        let max_pos: Option<i32> = self
            .conn
            .query_row(
                "SELECT MAX(position) FROM playlist_local_tracks WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to read max sidecar position: {}", e))
            })?;
        let count_based = (qobuz_track_count + local_count) as i32;
        Ok(count_based.max(max_pos.map(|p| p + 1).unwrap_or(0)))
    }

    /// One-shot healing for sidecar position collisions (mixed playlists).
    ///
    /// Positions are absolute slots in the merged interleave and have no
    /// UNIQUE constraint; the legacy Slint picker/drag wrote them 0-based per
    /// batch, which can produce duplicate positions that a Map-based merge
    /// collapses (silent row loss, edges E1/E2). This walks the local table
    /// in stable order (position ASC, added_at ASC, rowid ASC — the first
    /// claimant of a contested slot keeps it, matching the merge's emit) and
    /// renumbers every LATER claimant into the append region:
    /// `max(qobuz_track_count + sidecar_count, MAX(position) + 1)` onward.
    ///
    /// Non-colliding rows are NEVER touched — drift is normal (edge E7);
    /// this is collision repair, not renormalization. Returns one
    /// "kind ref: old -> new" description per moved row for the caller to
    /// log; empty = nothing healed. Idempotent.
    pub fn heal_playlist_sidecar_positions(
        &self,
        qobuz_playlist_id: u64,
        qobuz_track_count: u32,
    ) -> Result<Vec<String>, LibraryError> {
        // (kind, rowid, ref-description, position) in stable claim order.
        let mut rows: Vec<(&'static str, i64, String, i32)> = Vec::new();
        {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT id, local_track_id, position FROM playlist_local_tracks
                     WHERE qobuz_playlist_id = ?1
                     ORDER BY position ASC, added_at ASC, id ASC",
                )
                .map_err(|e| {
                    LibraryError::Database(format!("Failed to prepare heal query: {}", e))
                })?;
            let mapped = stmt
                .query_map(params![qobuz_playlist_id as i64], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i32>(2)?,
                    ))
                })
                .map_err(|e| LibraryError::Database(format!("Failed to query heal rows: {}", e)))?;
            for r in mapped {
                let (rowid, track, pos) = r.map_err(|e| {
                    LibraryError::Database(format!("Failed to read heal row: {}", e))
                })?;
                rows.push(("local", rowid, track.to_string(), pos));
            }
        }
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let max_pos = rows.iter().map(|r| r.3).max().unwrap_or(-1);
        let sidecar_total = rows.len();
        let mut seen = std::collections::HashSet::new();
        let mut moves: Vec<(&'static str, i64, String, i32)> = Vec::new();
        for row in rows {
            if !seen.insert(row.3) {
                moves.push(row);
            }
        }
        if moves.is_empty() {
            return Ok(Vec::new());
        }
        let mut next =
            ((qobuz_track_count as i32) + sidecar_total as i32).max(max_pos + 1);
        let mut healed = Vec::with_capacity(moves.len());
        for (kind, rowid, reference, old) in moves {
            let sql = "UPDATE playlist_local_tracks SET position = ?1 WHERE id = ?2";
            self.conn.execute(sql, params![next, rowid]).map_err(|e| {
                LibraryError::Database(format!("Failed to heal sidecar position: {}", e))
            })?;
            healed.push(format!("{kind} {reference}: {old} -> {next}"));
            next += 1;
        }
        Ok(healed)
    }
}
