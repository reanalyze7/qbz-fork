use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension, Result};

use super::types::SnapshotHeader;

fn row_to_header(r: &rusqlite::Row) -> Result<SnapshotHeader> {
    Ok(SnapshotHeader {
        qobuz_playlist_id: r.get::<_, i64>("qobuz_playlist_id")? as u64,
        name: r.get("name")?,
        owner: r.get("owner")?,
        track_count: r.get("track_count")?,
        snapped_at: r.get("snapped_at")?,
    })
}

/// One snapshot header, or None.
pub fn get_header(conn: &Connection, qobuz_playlist_id: u64) -> Result<Option<SnapshotHeader>> {
    conn.query_row(
        "SELECT qobuz_playlist_id, name, owner, track_count, snapped_at
           FROM qobuz_playlist_snapshot WHERE qobuz_playlist_id = ?1",
        params![qobuz_playlist_id as i64],
        row_to_header,
    )
    .optional()
}

/// All snapshot headers.
pub fn all_headers(conn: &Connection) -> Result<Vec<SnapshotHeader>> {
    let mut stmt = conn.prepare(
        "SELECT qobuz_playlist_id, name, owner, track_count, snapped_at
           FROM qobuz_playlist_snapshot",
    )?;
    let mut out = Vec::new();
    for r in stmt.query_map([], row_to_header)? {
        out.push(r?);
    }
    Ok(out)
}

/// One playlist's snapshot track ids in snapshot (position) order.
pub fn track_ids(conn: &Connection, qobuz_playlist_id: u64) -> Result<Vec<u64>> {
    let mut stmt = conn.prepare(
        "SELECT track_id FROM qobuz_playlist_snapshot_tracks
          WHERE qobuz_playlist_id = ?1 ORDER BY position",
    )?;
    let mut out = Vec::new();
    for r in stmt.query_map(params![qobuz_playlist_id as i64], |r| {
        r.get::<_, i64>(0)
    })? {
        out.push(r? as u64);
    }
    Ok(out)
}

/// playlist id -> snapshot track ids in position order, for every playlist
/// with membership rows (availability intersection, B8).
pub fn all_track_ids(conn: &Connection) -> Result<HashMap<u64, Vec<u64>>> {
    let mut stmt = conn.prepare(
        "SELECT qobuz_playlist_id, track_id FROM qobuz_playlist_snapshot_tracks
          ORDER BY qobuz_playlist_id, position",
    )?;
    let mut out: HashMap<u64, Vec<u64>> = HashMap::new();
    for r in stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)? as u64))
    })? {
        let (pid, tid) = r?;
        out.entry(pid).or_default().push(tid);
    }
    Ok(out)
}
