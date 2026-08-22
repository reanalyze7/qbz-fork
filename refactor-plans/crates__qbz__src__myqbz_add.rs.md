# crates/qbz/src/myqbz_add.rs (379 lines)

## Summary
"Add to Mixtape/Collection" controller: holds pending items to add in a
process-global `Mutex`, opens/closes the picker modal, loads+filters+sorts the
target-collection rows from the `qbz_mixtape` repo (kind-restricted,
recency-sorted, `item_exists`-resolved), and performs the actual add/create
mutation with a toast summarizing the outcome.

## Proposed split
By responsibility — payload/open-close state ↔ row loading (DB/IO) ↔ row
rendering (pure mapping) ↔ mutation (DB/IO):

- `myqbz_add/mod.rs` (~40 lines) — module doc, `AddItem` struct, `PENDING`
  static + `pending_snapshot`, `item_type_from_str`/`source_from_str` enum
  mapping helpers, `mod` declarations, `pub use` re-exports.
- `myqbz_add/open_close.rs` (~55 lines) — `open`, `close` (the modal
  open/close + header-string computation; pure UI-state plus one lock).
- `myqbz_add/rows.rs` (~100 lines) — `LoadedRow` struct, `load_rows` (the
  blocking DB read + kind-restriction + sort + `item_exists` resolution —
  clearly the "IO" module here).
- `myqbz_add/render.rs` (~80 lines) — `kind_icon`, `kind_label`,
  `album_count_label`, `apply_rows`, `ROWS_CACHE` static, `rebuild` (pure
  mapping from `LoadedRow` to `MyQbzAddRow` plus the search-filter rebuild).
- `myqbz_add/mutate.rs` (~110 lines) — `AddOutcome` struct, `add_items`,
  `toast_outcome`, `take_pending`, `track_items_from_local`,
  `create_collection` (the DB-writing / mutation half).

## Re-export surface
`myqbz_add/mod.rs` re-exports `AddItem`, `open`, `close`, `LoadedRow`,
`load_rows`, `apply_rows`, `rebuild`, `AddOutcome`, `add_items`,
`toast_outcome`, `take_pending`, `track_items_from_local`, `create_collection`
— i.e. everything currently `pub` in the file — so `main.rs`'s wiring
(`myqbz_add::open(...)`, spawn `load_rows` → `apply_rows`, etc., per the file's
own doc comment) needs no changes.

## Coupling / watch out
- `PENDING` (mod.rs) is read by `open`/`close` (open_close.rs), `load_rows`
  (rows.rs, via the `items: &[AddItem]` parameter — actually passed in, not
  read directly), and `take_pending` (mutate.rs) — keep it in `mod.rs` and
  reference as `super::PENDING` from the other files.
- `ROWS_CACHE` (render.rs) is read by both `apply_rows` and `rebuild`, both in
  the same proposed file — no cross-file coupling concern there.
- `item_type_from_str`/`source_from_str` are used by both `rows.rs`
  (`item_exists` call) and `mutate.rs` (`add_item_with` call) — keep them in
  `mod.rs` as small shared enum-mapping helpers, `pub(super)`.
- `create_collection` in this file delegates to `crate::myqbz::create_collection`
  (a differently-named function in a sibling module) — do not confuse the two
  during the split; the wrapper here just adapts kind-string + return shape.

## Verify after split
- `cargo check -p qbz` / `cargo build`.
- No existing unit tests in this file; none to keep green.
- Smoke-test: trigger "Add to Mixtape/Collection" from an album/track/playlist,
  confirm the picker lists collections correctly filtered/sorted, search
  filters client-side, adding an already-present item shows "Already in X", and
  "Create new" still works.
