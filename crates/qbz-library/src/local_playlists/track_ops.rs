//! Track membership reads and inserts (`get_tracks`/`add_tracks`).

use rusqlite::{params, Connection, Result};

use super::model::{now_ms, LocalPlaylistTrack, LocalPlaylistTrackInput, LocalPlaylistTrackSource};

/// Membership rows in position order.
pub fn get_tracks(conn: &Connection, playlist_id: &str) -> Result<Vec<LocalPlaylistTrack>> {
    let mut stmt = conn.prepare(
        "SELECT playlist_id, position, source, qobuz_track_id, local_path, added_at
           FROM local_playlist_tracks
          WHERE playlist_id = ?1
          ORDER BY position ASC",
    )?;
    let mut out = Vec::new();
    for r in stmt.query_map(params![playlist_id], |r| {
        Ok(LocalPlaylistTrack {
            playlist_id: r.get("playlist_id")?,
            position: r.get("position")?,
            source: LocalPlaylistTrackSource::parse(&r.get::<_, String>("source")?),
            qobuz_track_id: r.get::<_, Option<i64>>("qobuz_track_id")?.map(|v| v as u64),
            local_path: r.get("local_path")?,
            added_at: r.get("added_at")?,
        })
    })? {
        out.push(r?);
    }
    Ok(out)
}

/// Append tracks at the end (positions continue after the current max).
/// Exact duplicates (same source + same ref already in the playlist) are
/// skipped silently (the sidecar tables' UNIQUE idempotence, kept here by
/// an explicit existence check since position is part of every row).
/// Returns the number of rows actually inserted.
pub fn add_tracks(
    conn: &Connection,
    playlist_id: &str,
    entries: &[LocalPlaylistTrackInput],
) -> Result<usize> {
    let mut next_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1
           FROM local_playlist_tracks WHERE playlist_id = ?1",
        params![playlist_id],
        |r| r.get(0),
    )?;
    let ts = now_ms();
    let mut inserted = 0usize;
    for entry in entries {
        let (source, qobuz_id, local_path): (&str, Option<i64>, Option<&str>) = match entry {
            LocalPlaylistTrackInput::Qobuz(id) => ("qobuz", Some(*id as i64), None),
            LocalPlaylistTrackInput::Local(path) => ("local", None, Some(path.as_str())),
        };
        let exists: bool = conn
            .prepare(
                "SELECT 1 FROM local_playlist_tracks
                  WHERE playlist_id = ?1 AND source = ?2
                    AND COALESCE(qobuz_track_id, -1) = COALESCE(?3, -1)
                    AND COALESCE(local_path, '') = COALESCE(?4, '')
                  LIMIT 1",
            )?
            .exists(params![playlist_id, source, qobuz_id, local_path])?;
        if exists {
            continue;
        }
        conn.execute(
            "INSERT INTO local_playlist_tracks
                (playlist_id, position, source, qobuz_track_id, local_path, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![playlist_id, next_pos, source, qobuz_id, local_path, ts],
        )?;
        next_pos += 1;
        inserted += 1;
    }
    if inserted > 0 {
        conn.execute(
            "UPDATE local_playlists SET updated_at = ?1 WHERE id = ?2",
            params![ts, playlist_id],
        )?;
    }
    Ok(inserted)
}
