# crates/qbz-cache/src/audio_cache.rs (273 lines)

## Summary
L1 in-memory LRU audio cache: a `Mutex<CacheState>` holding cached track bytes,
LRU access order, in-flight-fetch tracking, and a per-track "recently failed"
back-off map, with optional L2 disk-cache spillover on eviction.

## Proposed split
Just over budget (273 lines); split by concern — state/eviction core ↔
fetch-tracking/back-off ↔ stats:

- `audio_cache/mod.rs` (~150 lines) — `CachedTrack`, `CacheState`, `AudioCache`
  struct + `Default`/`new`/`with_playback_cache`/`set_playback_cache`/
  `get_playback_cache`, plus the core `get`/`contains`/`insert`/`clear` methods
  (the LRU + disk-spillover heart of the cache).
- `audio_cache/fetch_tracking.rs` (~60 lines) — `is_fetching`, `mark_fetching`,
  `unmark_fetching`, `mark_failed`, `recently_failed`, `clear_failed` (an
  `impl AudioCache` block covering only the in-flight/back-off bookkeeping tied
  to issue #637).
- `audio_cache/stats.rs` (~30 lines) — `CacheStats` struct, `AudioCache::stats`.

## Re-export surface
`audio_cache/mod.rs` stays the public surface: `AudioCache`, `CachedTrack`, and
`CacheStats` (re-exported via `pub use stats::CacheStats;`) — all under
`qbz_cache::audio_cache::*`, so callers elsewhere in `qbz-cache`/`qbz` that use
`AudioCache::new(...)`, `.get(...)`, `.stats()`, etc. see no path change (the
methods stay inherent on `AudioCache` regardless of which file's `impl` block
defines them).

## Coupling / watch out
- All three files' methods operate on the same `Mutex<CacheState>` field
  (`self.state`) — since `CacheState`'s fields (`tracks`, `access_order`,
  `fetching`, `failed`, `current_size`) are only accessed through `AudioCache`'s
  own methods (never leaked to callers), splitting the `impl` blocks across
  files is purely organizational; no visibility changes needed as long as
  `CacheState`'s fields stay `pub(crate)`-or-private within the same module
  path (keep `CacheState` itself in `mod.rs`, not a separate file, so its
  private fields stay reachable from every `impl AudioCache` block in this
  module).
- `insert()` (mod.rs) is the one method that touches `playback_cache` (L2
  spillover) — keep it with the struct definition since it's the most complex
  method and the one most likely to need `CacheState` field access alongside
  the `Option<Arc<PlaybackCache>>`.
- `crate::PlaybackCache` import needed wherever `playback_cache` is touched
  (`mod.rs` only, per this split).

## Verify after split
- `cargo check -p qbz-cache` / `cargo build -p qbz-cache`.
- `cargo test -p qbz-cache` — check for existing tests under
  `qbz-cache/src/` or a `tests/` dir exercising `AudioCache` (none were visible
  in this 273-line file itself, but check sibling files/integration tests).
- Smoke-test: run the app, play a few tracks back-to-back beyond the 400MB
  default cache size and confirm LRU eviction + L2 spillover still function
  (via logs: "Evicting track ... from memory cache").
