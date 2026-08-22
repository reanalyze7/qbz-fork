# crates/qbz-app/src/settings/pinned_items.rs (323 lines)

## Summary
Headless, frontend-agnostic `PinnedItemsService`: SQLite-backed store for the Home
"Pinned" section (albums/artists/playlists), with an in-memory `HashSet<(kind, id)>`
mirror for O(1) `is_pinned` checks; mirrors `artist_blacklist.rs`'s conventions.

## Proposed split
Clean pure-struct-and-impl file with one big `#[cfg(test)]` block (~75 lines) at the
end. Split by responsibility: schema/lifecycle vs. read/write operations vs. tests.

- `pinned_items/mod.rs` (~50 lines) — module doc, `DB_FILE_NAME` const,
  `PinnedItem` struct, `pub use` of `PinnedItemsService` from `service.rs`.
- `pinned_items/service.rs` (~90 lines) — `PinnedItemsService` struct definition +
  `new`, `new_in_memory`, `init_schema`, `load_from_db` (construction/lifecycle
  concerns).
- `pinned_items/ops.rs` (~95 lines) — the `impl PinnedItemsService` block continued:
  `is_pinned`, `pin`, `unpin`, `list`, `count`, `keys_snapshot` (the read/write API
  surface). Use a second `impl PinnedItemsService { ... }` block in this file —
  Rust allows multiple `impl` blocks for the same type across files as long as
  they're in the same crate.
- `pinned_items/tests.rs` (~80 lines) — the entire `#[cfg(test)] mod tests` block
  (the `item()` helper + the single combined `lifecycle` test), declared via
  `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`pinned_items/mod.rs` re-exports `PinnedItem`, `PinnedItemsService`, `DB_FILE_NAME` at
`crate::settings::pinned_items::*` — the per-user lifecycle wrapper in the `qbz`
crate (mentioned in the module doc) continues to import from the same path
unchanged.

## Coupling / watch out
- `pinned_keys: RwLock<HashSet<(String, String)>>` is shared mutable state read/written
  by nearly every method in `ops.rs` but only initialized in `service.rs`'s
  constructors — keep the field itself defined once in `service.rs`'s struct
  definition; `ops.rs` just accesses `self.pinned_keys`.
- `conn: Connection` (rusqlite) is not `Send`-shared beyond the struct itself; no
  special handling needed for the split, just don't duplicate the field.
- The module doc explicitly cross-references `artist_blacklist.rs` as the sibling
  file with "same pragmas, error style" — check whether `artist_blacklist.rs` is
  also oversized and, if another agent is splitting it, keep the same directory
  convention (`service.rs`/`ops.rs`/`tests.rs` naming) for consistency between the two.

## Verify after split
- `cargo test -p qbz-app pinned_items::tests::lifecycle` — the one combined
  lifecycle test must stay green (it exercises pin/unpin/list/count/keys_snapshot
  end-to-end).
- `cargo check -p qbz-app` and grep for `pinned_items::` importers in the `qbz` crate
  (the per-user lifecycle wrapper) to confirm the public path is unchanged.
