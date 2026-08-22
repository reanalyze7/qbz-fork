//! CRUD repository for mixtape_collections and mixtape_collection_items.
//!
//! All functions take `&Connection` (or `&mut Connection` when a transaction
//! is needed). No Tauri state, no async runtime — testable with in-memory
//! SQLite. The Slint command layer wraps these with the app's library handle.

mod collections;
mod collections_edit;
mod items;
mod items_add;
mod items_reorder;
mod rows;
mod serde_helpers;

pub use collections::{create_collection, get_collection, list_collections};
pub use collections_edit::{
    delete_collection, get_custom_artwork, rename_collection, set_custom_artwork, set_description,
    set_kind, set_play_mode, touch_play,
};
pub use items::{item_exists, list_items, remove_item};
pub use items_add::{add_item, add_item_with};
pub use items_reorder::reorder_items;

use rows::{row_to_collection, row_to_item};
use serde_helpers::{
    parse_item_type, parse_kind, parse_play_mode, parse_source, parse_source_type,
    serialize_item_type, serialize_kind, serialize_play_mode, serialize_source,
    serialize_source_type,
};

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests;
