//! Item insertion, with and without dedup, for mixtape_collection_items.

use rusqlite::{params, Connection, Result};

use qbz_models::mixtape::{AlbumSource, ItemType};

use super::{now_ms, serialize_item_type, serialize_source};

/// Insert a new item at the end of the collection. Returns `true` if inserted,
/// `false` if an exact (source, source_item_id) duplicate already exists in
/// this collection. Different variants of the same album — e.g. Qobuz vs a
/// Local copy — are NOT deduped (they may differ in mastering or quality).
pub fn add_item(
    conn: &Connection,
    collection_id: &str,
    item_type: ItemType,
    source: AlbumSource,
    source_item_id: &str,
    title: &str,
    subtitle: Option<&str>,
    artwork_url: Option<&str>,
    year: Option<i32>,
    track_count: Option<i32>,
) -> Result<bool> {
    add_item_with(
        conn, collection_id, item_type, source, source_item_id,
        title, subtitle, artwork_url, year, track_count,
        false,
    )
}

/// Same as `add_item` but when `allow_duplicate` is true the
/// `(collection_id, source, source_item_id)` dedup check is skipped and the
/// row is always inserted. Used by the confirmation-backed add flow so a
/// user that explicitly says "yes, add it again" gets their duplicate.
#[allow(clippy::too_many_arguments)]
pub fn add_item_with(
    conn: &Connection,
    collection_id: &str,
    item_type: ItemType,
    source: AlbumSource,
    source_item_id: &str,
    title: &str,
    subtitle: Option<&str>,
    artwork_url: Option<&str>,
    year: Option<i32>,
    track_count: Option<i32>,
    allow_duplicate: bool,
) -> Result<bool> {
    if !allow_duplicate {
        let exists: bool = conn
            .prepare(
                "SELECT 1 FROM mixtape_collection_items
                   WHERE collection_id = ?1 AND source = ?2 AND source_item_id = ?3
                   LIMIT 1",
            )?
            .exists(params![collection_id, serialize_source(source), source_item_id])?;
        if exists {
            return Ok(false);
        }
    }

    let next_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1
           FROM mixtape_collection_items WHERE collection_id = ?1",
        params![collection_id],
        |r| r.get(0),
    )?;

    let ts = now_ms();
    conn.execute(
        "INSERT INTO mixtape_collection_items (
            collection_id, position, item_type, source, source_item_id,
            title, subtitle, artwork_url, year, track_count, added_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            collection_id,
            next_pos,
            serialize_item_type(item_type),
            serialize_source(source),
            source_item_id,
            title,
            subtitle,
            artwork_url,
            year,
            track_count,
            ts,
        ],
    )?;
    conn.execute(
        "UPDATE mixtape_collections SET updated_at = ?1 WHERE id = ?2",
        params![ts, collection_id],
    )?;
    Ok(true)
}
