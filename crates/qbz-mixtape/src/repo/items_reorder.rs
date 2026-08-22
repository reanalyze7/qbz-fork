//! Whole-collection item reordering, done inside a single transaction.

use rusqlite::{params, Result};

use qbz_models::mixtape::MixtapeCollectionItem;

use super::{now_ms, row_to_item, serialize_item_type, serialize_source};

/// Rewrite an entire collection's item order in a single transaction.
/// `new_order_positions` is a permutation of current positions (0..N).
pub fn reorder_items(
    conn: &mut rusqlite::Connection,
    collection_id: &str,
    new_order_positions: &[i32],
) -> Result<()> {
    let tx = conn.transaction()?;
    let current = list_items_tx(&tx, collection_id)?;
    if current.len() != new_order_positions.len() {
        return Err(rusqlite::Error::InvalidParameterName(
            "reorder length mismatch".into(),
        ));
    }

    tx.execute(
        "DELETE FROM mixtape_collection_items WHERE collection_id = ?1",
        params![collection_id],
    )?;

    for (new_pos, old_pos) in new_order_positions.iter().enumerate() {
        let item = current
            .iter()
            .find(|i| i.position == *old_pos)
            .ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(format!(
                    "unknown position {} in reorder",
                    old_pos
                ))
            })?;
        tx.execute(
            "INSERT INTO mixtape_collection_items (
                collection_id, position, item_type, source, source_item_id,
                title, subtitle, artwork_url, year, track_count, added_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &item.collection_id,
                new_pos as i32,
                serialize_item_type(item.item_type),
                serialize_source(item.source),
                &item.source_item_id,
                &item.title,
                &item.subtitle,
                &item.artwork_url,
                &item.year,
                &item.track_count,
                &item.added_at,
            ],
        )?;
    }
    tx.execute(
        "UPDATE mixtape_collections SET updated_at = ?1 WHERE id = ?2",
        params![now_ms(), collection_id],
    )?;
    tx.commit()
}

fn list_items_tx(
    tx: &rusqlite::Transaction,
    collection_id: &str,
) -> Result<Vec<MixtapeCollectionItem>> {
    let mut stmt = tx.prepare(
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
