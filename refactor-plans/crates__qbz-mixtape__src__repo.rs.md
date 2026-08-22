# crates/qbz-mixtape/src/repo.rs (737 lines)

CRUD repository for mixtape_collections + mixtape_collection_items over
`rusqlite::Connection`. No Tauri/async — testable with in-memory SQLite.

## Proposed split

- `repo/mod.rs` (~40 lines) — re-export surface + `now_ms()` helper.
- `repo/collections.rs` (~200 lines) — Collection CRUD: `create_collection`,
  `list_collections`, `get_collection`, `rename_collection`,
  `set_description`, `set_play_mode`, `set_kind`, `set_custom_artwork`,
  `get_custom_artwork`, `delete_collection`, `touch_play`.
- `repo/items.rs` (~220 lines) — Item CRUD: `list_items`, `add_item`,
  `add_item_with`, `item_exists`, `remove_item`, `reorder_items`,
  `list_items_tx`.
- `repo/serde_helpers.rs` (~70 lines) — the serialize/parse fns for
  `CollectionKind`/`CollectionSourceType`/`CollectionPlayMode`/`ItemType`/
  `AlbumSource`.
- `repo/rows.rs` (~40 lines) — `row_to_collection`, `row_to_item`.
- `repo/tests.rs` (~190 lines) — existing test module.

## Tricky coupling

- `collections.rs`'s `get_collection`/`list_collections` call
  `items::list_items` to hydrate `.items` — needs `use super::items;`.
- `items.rs`'s row mappers use `super::serde_helpers::{parse_item_type,
  parse_source}` and `super::rows::row_to_item`.
- `reorder_items` uses a transaction (`&mut Connection`) — keep its
  `list_items_tx` helper alongside it in `items.rs`, not shared with the
  plain `&Connection` `list_items`.

## Verify after split

`cargo build -p qbz-mixtape`, `cargo test -p qbz-mixtape repo::` (11
existing tests must stay green — CRUD, dedup, reorder, cascade, kind
conversion guard).
