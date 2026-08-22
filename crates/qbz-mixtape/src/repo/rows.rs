//! Row -> model mappers for mixtape_collections / mixtape_collection_items.

use rusqlite::Result;

use qbz_models::mixtape::{MixtapeCollection, MixtapeCollectionItem};

use super::{parse_item_type, parse_kind, parse_play_mode, parse_source, parse_source_type};

pub fn row_to_collection(r: &rusqlite::Row) -> Result<MixtapeCollection> {
    Ok(MixtapeCollection {
        id: r.get("id")?,
        kind: parse_kind(&r.get::<_, String>("kind")?),
        name: r.get("name")?,
        description: r.get("description")?,
        source_type: parse_source_type(&r.get::<_, String>("source_type")?),
        source_ref: r.get("source_ref")?,
        play_mode: parse_play_mode(&r.get::<_, String>("play_mode")?),
        custom_artwork_path: r.get("custom_artwork_path")?,
        position: r.get("position")?,
        hidden: r.get::<_, i64>("hidden")? != 0,
        last_played_at: r.get("last_played_at")?,
        play_count: r.get("play_count")?,
        last_synced_at: r.get("last_synced_at")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
        items: Vec::new(),
    })
}

pub fn row_to_item(r: &rusqlite::Row) -> Result<MixtapeCollectionItem> {
    Ok(MixtapeCollectionItem {
        collection_id: r.get("collection_id")?,
        position: r.get("position")?,
        item_type: parse_item_type(&r.get::<_, String>("item_type")?),
        source: parse_source(&r.get::<_, String>("source")?),
        source_item_id: r.get("source_item_id")?,
        title: r.get("title")?,
        subtitle: r.get("subtitle")?,
        artwork_url: r.get("artwork_url")?,
        year: r.get("year")?,
        track_count: r.get("track_count")?,
        added_at: r.get("added_at")?,
    })
}
