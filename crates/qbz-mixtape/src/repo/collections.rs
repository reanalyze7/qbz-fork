//! Collection reads + creation: create/list/get.

use rusqlite::{params, Connection, OptionalExtension, Result};
use uuid::Uuid;

use qbz_models::mixtape::{CollectionKind, CollectionPlayMode, CollectionSourceType, MixtapeCollection};

use super::items::list_items;
use super::{now_ms, row_to_collection, serialize_kind, serialize_play_mode, serialize_source_type};

pub fn create_collection(
    conn: &Connection,
    kind: CollectionKind,
    name: &str,
    description: Option<&str>,
    source_type: CollectionSourceType,
    source_ref: Option<&str>,
) -> Result<MixtapeCollection> {
    let id = Uuid::new_v4().to_string();
    let ts = now_ms();

    // New collections go to the top of their kind's navigation (position = 0;
    // shift others down). Manual drag-reorder can rearrange later.
    conn.execute(
        "UPDATE mixtape_collections SET position = position + 1 WHERE kind = ?1",
        params![serialize_kind(kind)],
    )?;

    let last_synced_at = match source_type {
        CollectionSourceType::ArtistDiscography => Some(ts),
        _ => None,
    };

    conn.execute(
        "INSERT INTO mixtape_collections (
            id, kind, name, description,
            source_type, source_ref,
            play_mode, custom_artwork_path,
            position, hidden, last_played_at, play_count,
            last_synced_at, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 0, 0, NULL, 0, ?8, ?9, ?10)",
        params![
            id,
            serialize_kind(kind),
            name,
            description,
            serialize_source_type(source_type),
            source_ref,
            serialize_play_mode(CollectionPlayMode::InOrder),
            last_synced_at,
            ts,
            ts,
        ],
    )?;

    get_collection(conn, &id).map(|o| o.expect("just inserted"))
}

pub fn list_collections(
    conn: &Connection,
    kind: Option<CollectionKind>,
) -> Result<Vec<MixtapeCollection>> {
    let mut out = Vec::new();
    match kind {
        Some(k) => {
            let mut stmt = conn.prepare(
                "SELECT id, kind, name, description, source_type, source_ref,
                        play_mode, custom_artwork_path, position, hidden,
                        last_played_at, play_count, last_synced_at,
                        created_at, updated_at
                   FROM mixtape_collections
                   WHERE kind = ?1
                   ORDER BY position ASC",
            )?;
            let rows = stmt.query_map(params![serialize_kind(k)], row_to_collection)?;
            for r in rows {
                out.push(r?);
            }
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, kind, name, description, source_type, source_ref,
                        play_mode, custom_artwork_path, position, hidden,
                        last_played_at, play_count, last_synced_at,
                        created_at, updated_at
                   FROM mixtape_collections
                   ORDER BY kind, position ASC",
            )?;
            let rows = stmt.query_map([], row_to_collection)?;
            for r in rows {
                out.push(r?);
            }
        }
    }
    // Hydrate items for every listed collection so the sidebar / AddToMixtape
    // modal can show accurate item counts + mosaic artwork. Without this the
    // listing returns `items: []` and every count reads as zero.
    for c in out.iter_mut() {
        c.items = list_items(conn, &c.id)?;
    }
    Ok(out)
}

pub fn get_collection(conn: &Connection, id: &str) -> Result<Option<MixtapeCollection>> {
    let maybe = conn
        .query_row(
            "SELECT id, kind, name, description, source_type, source_ref,
                    play_mode, custom_artwork_path, position, hidden,
                    last_played_at, play_count, last_synced_at,
                    created_at, updated_at
               FROM mixtape_collections
               WHERE id = ?1",
            params![id],
            row_to_collection,
        )
        .optional()?;
    if let Some(mut c) = maybe {
        c.items = list_items(conn, id)?;
        Ok(Some(c))
    } else {
        Ok(None)
    }
}
