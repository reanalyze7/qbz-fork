use rusqlite::{params, Connection, Result};

use super::schema::now_ms;
use super::types::SnapshotNameEntry;

/// NAMES producer: upsert a header row for every listed playlist. Never
/// touches the membership table. Stamps `snapped_at` = now on each row.
pub fn upsert_names(conn: &Connection, entries: &[SnapshotNameEntry]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let ts = now_ms();
    let mut stmt = conn.prepare(
        "INSERT INTO qobuz_playlist_snapshot
             (qobuz_playlist_id, name, owner, track_count, snapped_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(qobuz_playlist_id) DO UPDATE SET
             name = excluded.name,
             owner = excluded.owner,
             track_count = excluded.track_count,
             snapped_at = excluded.snapped_at",
    )?;
    for e in entries {
        stmt.execute(params![
            e.qobuz_playlist_id as i64,
            e.name,
            e.owner,
            e.track_count,
            ts
        ])?;
    }
    Ok(())
}

/// MEMBERSHIP producer: full-replace the snapshot track ids of ONE playlist
/// (detail load) and refresh its header (name / owner / track_count /
/// snapped_at). Returns `false` (writing NOTHING) when the playlist has no
/// header row — i.e. it was never captured by the names producer, so it is
/// not one of the user's listed playlists.
pub fn replace_tracks(
    conn: &Connection,
    qobuz_playlist_id: u64,
    name: &str,
    owner: Option<&str>,
    track_ids: &[u64],
) -> Result<bool> {
    let updated = conn.execute(
        "UPDATE qobuz_playlist_snapshot
            SET name = ?2, owner = ?3, track_count = ?4, snapped_at = ?5
          WHERE qobuz_playlist_id = ?1",
        params![
            qobuz_playlist_id as i64,
            name,
            owner,
            track_ids.len() as u32,
            now_ms()
        ],
    )?;
    if updated == 0 {
        return Ok(false);
    }
    conn.execute(
        "DELETE FROM qobuz_playlist_snapshot_tracks WHERE qobuz_playlist_id = ?1",
        params![qobuz_playlist_id as i64],
    )?;
    let mut stmt = conn.prepare(
        "INSERT INTO qobuz_playlist_snapshot_tracks (qobuz_playlist_id, position, track_id)
         VALUES (?1, ?2, ?3)",
    )?;
    for (pos, tid) in track_ids.iter().enumerate() {
        stmt.execute(params![qobuz_playlist_id as i64, pos as i64, *tid as i64])?;
    }
    Ok(true)
}
