# crates/qbz-reco/src/store.rs (526 lines)

## Summary
SQLite-backed `ArtistVectorStore`: schema init, in-memory MBID<->idx index,
sparse-vector persistence (`set_vector`/`get_vector`), similarity/related
artist queries, and cleanup. Ported 1:1 from Tauri's `artist_vectors::store`.

## Proposed split
By responsibility, keeping `impl ArtistVectorStore` as inherent methods spread
across files via multiple `impl` blocks in the same module (Rust allows this)
or move to submodules with `mod.rs` re-exporting the type:

- `store/mod.rs` (~60 lines) — struct defs (`ArtistVectorStore`,
  `SimilarArtist`), `VECTOR_TTL_SECS`, `pub use` of helpers, `current_timestamp`.
- `store/init.rs` (~90 lines) — `open_at`, `init` (schema DDL), `load_artist_index`.
- `store/index.rs` (~60 lines) — `get_or_create_idx`, `get_idx`, `get_mbid`,
  `get_artist_name`.
- `store/vectors.rs` (~100 lines) — `set_vector`, `get_vector`,
  `has_fresh_vector`, `cleanup_expired`, `clear_all`.
- `store/related.rs` (~90 lines) — `get_related_artists`, `get_all_related_artists`.
- `store/tests.rs` (~95 lines) — the existing `#[cfg(test)] mod tests`.

All of the above are `impl ArtistVectorStore { ... }` blocks split across
files but the struct itself only lives in `mod.rs`; each file does
`use super::ArtistVectorStore;` and adds an `impl` block.

## Re-export surface
`store/mod.rs` stays the `mod store;` target; `pub use` isn't even needed
since callers already do `crate::store::ArtistVectorStore` — multiple `impl`
blocks in different files under the same module resolve to the same type
automatically. No public API path changes.

## Coupling / watch out
- `artist_to_idx`/`idx_to_artist`/`next_idx` are private fields only
  touched by `index.rs` and `init.rs` — keep them `pub(super)` or add
  accessor methods if splitting further than "same module, multiple impls".
- `current_timestamp()` is a free fn used by `vectors.rs` and `related.rs`
  is not — keep it in `mod.rs` and import via `use super::current_timestamp`.
- Byte-identical SQLite schema (3 tables) must not change — this is a pure
  code-organization split, no SQL changes.

## Verify after split
- `cargo test -p qbz-reco store::` — all 4 existing tests
  (`create_artist_index`, `store_and_retrieve_vector`,
  `related_artists_rank_by_summed_weight`, `fresh_vector_check`) green.
- `cargo check -p qbz-reco` and check callers in `builder.rs`/`suggestions.rs`
  still compile unchanged.
