//! Collection mutations: rename/describe/play_mode/kind/artwork/delete/touch.

use rusqlite::{params, Connection, Result};

use qbz_models::mixtape::{CollectionKind, CollectionPlayMode};

use super::collections::get_collection;
use super::{now_ms, serialize_kind, serialize_play_mode};

pub fn rename_collection(conn: &Connection, id: &str, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE mixtape_collections SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_name, now_ms(), id],
    )?;
    Ok(())
}

pub fn set_description(conn: &Connection, id: &str, desc: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE mixtape_collections SET description = ?1, updated_at = ?2 WHERE id = ?3",
        params![desc, now_ms(), id],
    )?;
    Ok(())
}

pub fn set_play_mode(conn: &Connection, id: &str, mode: CollectionPlayMode) -> Result<()> {
    conn.execute(
        "UPDATE mixtape_collections SET play_mode = ?1, updated_at = ?2 WHERE id = ?3",
        params![serialize_play_mode(mode), now_ms(), id],
    )?;
    Ok(())
}

/// Convert between Mixtape and Collection. Rejects any involvement of
/// ArtistCollection — that kind is anchored by `source_ref` (an artist id)
/// and cannot be freely renamed into something else.
pub fn set_kind(conn: &Connection, id: &str, new_kind: CollectionKind) -> Result<()> {
    let existing = get_collection(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    if matches!(existing.kind, CollectionKind::ArtistCollection)
        || matches!(new_kind, CollectionKind::ArtistCollection)
    {
        return Err(rusqlite::Error::InvalidParameterName(
            "cannot convert to/from artist_collection".into(),
        ));
    }
    conn.execute(
        "UPDATE mixtape_collections SET kind = ?1, updated_at = ?2 WHERE id = ?3",
        params![serialize_kind(new_kind), now_ms(), id],
    )?;
    Ok(())
}

pub fn set_custom_artwork(conn: &Connection, id: &str, path: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE mixtape_collections SET custom_artwork_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![path, now_ms(), id],
    )?;
    Ok(())
}

/// Returns the currently stored `custom_artwork_path` for a collection, if
/// any. Used before overwriting / clearing to find the previous file that
/// needs deleting from disk.
pub fn get_custom_artwork(conn: &Connection, id: &str) -> Result<Option<String>> {
    let path = conn
        .query_row(
            "SELECT custom_artwork_path FROM mixtape_collections WHERE id = ?1",
            params![id],
            |r| r.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten();
    Ok(path)
}

pub fn delete_collection(conn: &Connection, id: &str) -> Result<()> {
    // CASCADE removes items.
    conn.execute("DELETE FROM mixtape_collections WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn touch_play(conn: &Connection, id: &str) -> Result<()> {
    let ts = now_ms();
    conn.execute(
        "UPDATE mixtape_collections
            SET last_played_at = ?1, play_count = play_count + 1, updated_at = ?2
            WHERE id = ?3",
        params![ts, ts, id],
    )?;
    Ok(())
}
