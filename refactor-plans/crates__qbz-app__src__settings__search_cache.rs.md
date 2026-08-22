# crates/qbz-app/src/settings/search_cache.rs (412 lines)

## Summary
Search results cache (Capa A of Intelligent Search, ADR-006):
`normalize_query`, a `VolatileSlice` marker struct, a JSON-file-backed
`ArtistCacheStore` (persistent artist sub-cache), and the main
`SearchCache` (in-memory LRU-ish bound + eviction) with `get`/`put`. Free
helper `page<T>()`. Has a sizeable `#[cfg(test)]` module.

## Proposed split
- `search_cache/mod.rs` (~15 lines) — re-exports.
- `search_cache/normalize.rs` (~15 lines) — `normalize_query` (lines
  56-68), a small pure helper with its own whitespace/case-collapse rule.
- `search_cache/artist_store.rs` (~65 lines) — `VolatileSlice`,
  `ArtistCacheStore` struct + impl: `open_at`, `load_from`, `get`, `put`,
  `persist` (lines 69-140).
- `search_cache/cache.rs` (~110 lines) — `SearchCache` struct + impl:
  `new`, `get`, `put`, `evict_to_bound` (lines 141-240), plus the free
  `page<T>()` helper (lines 241-255) it depends on.
- `search_cache/tests.rs` (~155 lines) — the `#[cfg(test)] mod tests`
  block (lines 256-412), including its fixture builders.

## Re-export surface
`search_cache/mod.rs` re-exports `normalize_query`, `SearchCache` (and
`ArtistCacheStore` if it's `pub` and used outside this file — verify) at
the current `qbz_app::settings::search_cache::X` path.
`search_service.rs` (sibling file) does `use super::search_cache::
SearchCache;` — that relative-module path must keep resolving once
`search_cache.rs` becomes a directory module.

## Coupling / watch out
- `SearchCache::get`/`put` compose both the in-memory bound/eviction AND
  the persistent `ArtistCacheStore` — keep both pieces working together
  (don't let `cache.rs` lose track of when it needs to call into
  `artist_store.rs`).
- `normalize_query`'s whitespace/case rules are the single canonicity
  point for cache keys — every lookup/store path must route through it;
  don't let it drift or duplicate once split into its own file.
- `page<T>()` (used to build `SearchResultsPage<T>` values, per the tests)
  is a small generic helper shared by `get`/`put` — keep it visible to
  `cache.rs` (same file, or `pub(super)` if genuinely reused elsewhere).
- LRU eviction (`evict_to_bound`) semantics are pinned by the
  `lru_evicts_oldest_beyond_bound` test — don't change eviction order
  while splitting.

## Verify after split
- `cargo test -p qbz-app settings::search_cache` green (roundtrip, LRU
  eviction, persisted-artists-survive-reopen, normalize whitespace/case).
- `cargo build -p qbz-app` (and `search_service.rs`'s `use` of
  `SearchCache`).
