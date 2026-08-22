# crates/qbz-app/src/settings/local_favorites.rs (307 lines)

## Summary
Headless SQLite-backed service for favoriting LOCAL library items (albums/
artists/tracks — never Qobuz-offline downloads): `LocalFavoritesService`
holds a `Connection` plus an in-memory `(kind, id)` `HashSet` for O(1) heart
lookups, with schema init, load-from-db, favorite/unfavorite, list, and
per-artist counts.

## Proposed split
Mirrors the sibling `pinned_items.rs` file (same pragmas/error style per
ADR-006) — split by pure-data-type vs I/O-service vs tests, matching the
convention that file already established (check `refactor-plans/
crates__qbz-app__src__settings__pinned_items.rs.md` for the sibling's exact
split shape before implementing, so both land consistently).

- `local_favorites/mod.rs` (~20 lines) — module declarations + re-exports of
  `DB_FILE_NAME`, `LocalFavItem`, `LocalFavoritesService`.
- `local_favorites/model.rs` (~30 lines) — `DB_FILE_NAME` const,
  `LocalFavItem` struct (+ derives). Pure data, no logic.
- `local_favorites/service.rs` (~195 lines) — `LocalFavoritesService` struct
  + full impl: `new`, `new_in_memory`, `init_schema`, `load_from_db`,
  `is_favorite`, `favorite`, `unfavorite`, `list`, `count_by_artist`,
  `count`, `keys_snapshot`. This is the file's actual I/O core (SQLite reads/
  writes) — if it's still too close to 130 after extracting `model.rs` and
  `tests.rs`, split further into `service.rs` (schema/load/lifecycle: `new`,
  `new_in_memory`, `init_schema`, `load_from_db`, ~90 lines) and
  `queries.rs` (`is_favorite`, `favorite`, `unfavorite`, `list`,
  `count_by_artist`, `count`, `keys_snapshot`, ~105 lines, as a second
  `impl LocalFavoritesService` block).
- `local_favorites/tests.rs` (~55 lines) — the existing `#[cfg(test)] mod
  tests` block (`item` helper, `lifecycle`, `source_check_rejects_offline`),
  included via `#[cfg(test)] mod tests;` from `mod.rs`.

## Re-export surface
`local_favorites/mod.rs` is the public-API surface — re-export
`DB_FILE_NAME`, `LocalFavItem`, `LocalFavoritesService` at the same path so
`crate::settings::local_favorites::LocalFavoritesService` (and the
per-user lifecycle wrapper in the `qbz` crate mentioned in the file's own
doc comment, `crate::local_favorites`) is unaffected.

## Coupling / watch out
- The doc comment explicitly says this file "Mirrors `pinned_items.rs`" —
  since `pinned_items.rs` already has a plan in `refactor-plans/` (it's
  listed among the already-written plans), read that plan first and match
  its module-naming/line-count shape so a future reader sees the two
  siblings split identically, per the project's own stated convention.
- `RwLock<HashSet<(String, String)>>` is the in-memory index kept in sync
  with every write (`favorite`/`unfavorite` both update the SQLite row AND
  the in-memory set) — if `service.rs` splits further into `service.rs` +
  `queries.rs`, the `keys: RwLock<...>` field lives on the struct (defined
  once) but both impl blocks touch it; that's fine in Rust (multiple `impl`
  blocks for one struct across files), just don't accidentally duplicate
  the field.
- The `source TEXT NOT NULL CHECK (source IN ('local'))` SQL constraint is
  the mechanism that keeps this store free of Qobuz-offline duplicates
  (tested by `source_check_rejects_offline`) — this SQL string lives in
  `init_schema` and must not be reformatted/altered when moved.
- Per-user lifecycle (open/close per logged-in user) lives OUTSIDE this
  file in the `qbz` crate wrapper per the doc comment — no coupling to
  fix here, just don't assume this file owns that lifecycle.

## Verify after split
- `cargo build -p qbz-app`
- `cargo test -p qbz-app local_favorites` (both existing tests green,
  unchanged assertions)
- Grep for `LocalFavoritesService::`/`LocalFavItem` call sites (the mixed-
  library feed, per-artist favorite counts UI) to confirm no import broke.
