//! Playlist-header reads (`list`/`get`) and their count hydration.

use rusqlite::{params, Connection, OptionalExtension, Result};

use super::model::{LocalPlaylist, LocalPlaylistTrackSource};

fn row_to_playlist(r: &rusqlite::Row) -> Result<LocalPlaylist> {
    Ok(LocalPlaylist {
        id: r.get("id")?,
        name: r.get("name")?,
        description: r.get("description")?,
        offline_only: r.get::<_, i64>("offline_only")? != 0,
        favorite: r.get::<_, i64>("favorite")? != 0,
        hidden: r.get::<_, i64>("hidden")? != 0,
        custom_artwork_path: r.get("custom_artwork_path")?,
        folder_id: r.get("folder_id")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
        track_count: 0,
        qobuz_count: 0,
        local_count: 0,
    })
}

/// Fill the per-source counts on a loaded playlist header.
fn hydrate_counts(conn: &Connection, p: &mut LocalPlaylist) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT source, COUNT(*) FROM local_playlist_tracks
         WHERE playlist_id = ?1 GROUP BY source",
    )?;
    let rows = stmt.query_map(params![p.id], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?))
    })?;
    for row in rows {
        let (source, count) = row?;
        match LocalPlaylistTrackSource::parse(&source) {
            LocalPlaylistTrackSource::Qobuz => p.qobuz_count = count,
            LocalPlaylistTrackSource::Local => p.local_count = count,
        }
        p.track_count += count;
    }
    Ok(())
}

/// All local playlists (counts hydrated), newest first.
pub fn list(conn: &Connection) -> Result<Vec<LocalPlaylist>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, offline_only, favorite, hidden,
                custom_artwork_path, folder_id, created_at, updated_at
           FROM local_playlists
          ORDER BY created_at DESC",
    )?;
    let mut out: Vec<LocalPlaylist> = Vec::new();
    for r in stmt.query_map([], row_to_playlist)? {
        out.push(r?);
    }
    for p in out.iter_mut() {
        hydrate_counts(conn, p)?;
    }
    Ok(out)
}

/// One playlist header (counts hydrated), or None.
pub fn get(conn: &Connection, id: &str) -> Result<Option<LocalPlaylist>> {
    let maybe = conn
        .query_row(
            "SELECT id, name, description, offline_only, favorite, hidden,
                    custom_artwork_path, folder_id, created_at, updated_at
               FROM local_playlists WHERE id = ?1",
            params![id],
            row_to_playlist,
        )
        .optional()?;
    match maybe {
        Some(mut p) => {
            hydrate_counts(conn, &mut p)?;
            Ok(Some(p))
        }
        None => Ok(None),
    }
}
