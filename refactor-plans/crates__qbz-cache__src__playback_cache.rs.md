# crates/qbz-cache/src/playback_cache.rs (318 lines)

## Summary
L2 (disk-based) playback cache: LRU-evicted `{track_id}.audio` files under a
cache directory, with a `PlaybackCache` struct (get/insert/contains/clear/
stats) backed by an in-memory `HashMap<u64, CacheEntry>` index rebuilt from
disk on startup.

## Proposed split
This file is data-struct + I/O mixed together with no separate test module
(no `#[cfg(test)]` present), so the natural split is by responsibility
within the single `PlaybackCache` type — construction/rebuild vs. read/write
vs. eviction/stats:

- `playback_cache/mod.rs` (~55 lines) — module doc, `CacheEntry` (private),
  `PlaybackCacheState`, the `PlaybackCache` struct definition, and
  `PlaybackCacheStats` (lines 1-40, 312-318) — re-exports everything.
- `playback_cache/init.rs` (~85 lines) — `PlaybackCache::new`,
  `with_path`, `rebuild_state`, `track_path` (lines 42-127) — construction
  and the on-disk-scan-to-rebuild-index logic, as an `impl PlaybackCache`
  block in its own file (Rust allows splitting `impl` blocks for the same
  type across files/modules).
- `playback_cache/access.rs` (~120 lines) — `contains`, `get`, `insert`
  (lines 129-245) — the hot read/write path.
- `playback_cache/eviction.rs` (~65 lines) — `evict_if_needed`, `clear`,
  `stats`, `cache_dir` (lines 247-310) — LRU eviction + introspection.

## Re-export surface
`playback_cache/mod.rs` re-exports `PlaybackCache` and `PlaybackCacheStats`
(the two public types) at `crate::playback_cache::{PlaybackCache,
PlaybackCacheStats}`, matching current callers unchanged. The split `impl
PlaybackCache` blocks in `init.rs`/`access.rs`/`eviction.rs` each do
`use super::{PlaybackCache, PlaybackCacheState, CacheEntry};` and add
methods to the same type — no trait needed, Rust supports multiple `impl`
blocks per type across files as long as they're in the same crate.

## Coupling / watch out
- `CacheEntry` and `PlaybackCacheState` are used by every impl block
  (rebuild reads/writes `state.entries`; access reads/writes; eviction
  reads/writes) — keep both types in `mod.rs` so all three impl-block
  files can `use super::*`.
- The `state: Mutex<PlaybackCacheState>` locking pattern
  (`self.state.lock().unwrap()`) is repeated in nearly every method across
  what will become 3 files — no shared helper currently exists; consider
  (but don't require) adding a small private `fn lock(&self) -> MutexGuard`
  helper in `mod.rs` to DRY this up while splitting, since every method
  already does the exact same `.lock().unwrap()` boilerplate.
- `track_path` (in `init.rs` per this plan) is also called from
  `access.rs::get`/`insert` and `eviction.rs::evict_if_needed`/`clear` (the
  latter two currently reconstruct the path manually with
  `format!("{}.audio", track_id)` instead of calling `track_path` — note
  this existing minor duplication, worth fixing to call the shared helper
  during the split, but not required for the split itself). Since it's a
  cross-file call, it must be `pub(crate)` or unprefixed on `impl
  PlaybackCache`, not module-private — fine since it's an inherent method,
  not a free fn.

## Verify after split
- `cargo build -p qbz-cache` (no existing tests in this file to preserve,
  per the read; check the crate's integration tests directory for
  playback-cache coverage and run those, e.g. `cargo test -p qbz-cache`).
- Smoke-test: construct a `PlaybackCache`, insert + get a track, restart
  (drop and reconstruct via `with_path` on the same dir) to confirm
  `rebuild_state` still finds the file, then `clear()`.
