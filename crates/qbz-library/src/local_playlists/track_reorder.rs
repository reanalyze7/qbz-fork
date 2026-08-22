//! Track membership reorder/remove (`reorder`/`remove_track`).

use rusqlite::{params, Connection, Result};

use super::model::now_ms;

/// Move the row at `from` to `to` (both repo positions), shifting the rows
/// in between by one — remove-then-insert list semantics, so the moved row
/// lands at exactly position `to` (B2: the local detail's custom reorder).
/// `from == to`, or either position not naming an existing row, is a no-op.
pub fn reorder(conn: &Connection, playlist_id: &str, from: i32, to: i32) -> Result<()> {
    if from == to {
        return Ok(());
    }
    // Both endpoints must name existing rows, or the shift below would
    // corrupt the order (and -1 is the in-flight parking slot).
    let mut exists_stmt = conn.prepare(
        "SELECT 1 FROM local_playlist_tracks
          WHERE playlist_id = ?1 AND position = ?2 LIMIT 1",
    )?;
    if !exists_stmt.exists(params![playlist_id, from])?
        || !exists_stmt.exists(params![playlist_id, to])?
    {
        return Ok(());
    }
    conn.execute(
        "UPDATE local_playlist_tracks SET position = -1
          WHERE playlist_id = ?1 AND position = ?2",
        params![playlist_id, from],
    )?;
    if from < to {
        conn.execute(
            "UPDATE local_playlist_tracks SET position = position - 1
              WHERE playlist_id = ?1 AND position > ?2 AND position <= ?3",
            params![playlist_id, from, to],
        )?;
    } else {
        conn.execute(
            "UPDATE local_playlist_tracks SET position = position + 1
              WHERE playlist_id = ?1 AND position >= ?2 AND position < ?3",
            params![playlist_id, to, from],
        )?;
    }
    conn.execute(
        "UPDATE local_playlist_tracks SET position = ?2
          WHERE playlist_id = ?1 AND position = -1",
        params![playlist_id, to],
    )?;
    conn.execute(
        "UPDATE local_playlists SET updated_at = ?1 WHERE id = ?2",
        params![now_ms(), playlist_id],
    )?;
    Ok(())
}

/// Remove the row at `position` and compact the positions above it.
pub fn remove_track(conn: &Connection, playlist_id: &str, position: i32) -> Result<()> {
    conn.execute(
        "DELETE FROM local_playlist_tracks WHERE playlist_id = ?1 AND position = ?2",
        params![playlist_id, position],
    )?;
    conn.execute(
        "UPDATE local_playlist_tracks SET position = position - 1
          WHERE playlist_id = ?1 AND position > ?2",
        params![playlist_id, position],
    )?;
    conn.execute(
        "UPDATE local_playlists SET updated_at = ?1 WHERE id = ?2",
        params![now_ms(), playlist_id],
    )?;
    Ok(())
}
