//! Item reads/removal: list/exists/remove for mixtape_collection_items.

use rusqlite::{params, Connection, Result};

use qbz_models::mixtape::{AlbumSource, MixtapeCollectionItem};

use super::{now_ms, row_to_item, serialize_source};

pub fn list_items(conn: &Connection, collection_id: &str) -> Result<Vec<MixtapeCollectionItem>> {
    let mut stmt = conn.prepare(
        "SELECT collection_id, position, item_type, source, source_item_id,
                title, subtitle, artwork_url, year, track_count, added_at
           FROM mixtape_collection_items
           WHERE collection_id = ?1
           ORDER BY position ASC",
    )?;
    let mut out = Vec::new();
    for r in stmt.query_map(params![collection_id], row_to_item)? {
        out.push(r?);
    }
    Ok(out)
}

/// Returns true if this `(collection_id, source, source_item_id)` tuple
/// already has at least one row in mixtape_collection_items. Used by the
/// bulk-add confirmation flow so the UI can ask before inserting duplicates.
pub fn item_exists(
    conn: &Connection,
    collection_id: &str,
    source: AlbumSource,
    source_item_id: &str,
) -> Result<bool> {
    let exists: bool = conn
        .prepare(
            "SELECT 1 FROM mixtape_collection_items
               WHERE collection_id = ?1 AND source = ?2 AND source_item_id = ?3
               LIMIT 1",
        )?
        .exists(params![collection_id, serialize_source(source), source_item_id])?;
    Ok(exists)
}

pub fn remove_item(conn: &Connection, collection_id: &str, position: i32) -> Result<()> {
    conn.execute(
        "DELETE FROM mixtape_collection_items
           WHERE collection_id = ?1 AND position = ?2",
        params![collection_id, position],
    )?;
    // Compact positions above the removed index.
    conn.execute(
        "UPDATE mixtape_collection_items
           SET position = position - 1
           WHERE collection_id = ?1 AND position > ?2",
        params![collection_id, position],
    )?;
    conn.execute(
        "UPDATE mixtape_collections SET updated_at = ?1 WHERE id = ?2",
        params![now_ms(), collection_id],
    )?;
    Ok(())
}
