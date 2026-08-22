# crates/qbz/src/offline_cache.rs (597 lines)

## Summary
Slint offline-cache controller: the thin orchestration layer over
`qbz-offline-cache` that triggers single/batch/album/playlist caching and
removal, tracks a session-wide "which track ids are cached" set, and drives
per-row cache-status/unlock-padlock updates via a `CacheEventSink`.

## Proposed split
By domain — track the cached-ids cache separately from the
trigger/action functions, and split the actions by "cache" vs "remove":

- `offline_cache/mod.rs` (~25 lines) — module doc, `pub use` re-exports of
  every currently-public item (`is_cached`, `cached_ids_set`,
  `load_cached_ids`, `row_sink`, `cache_track`, `cache_tracks`,
  `cache_album`, `cache_playlist`, `remove_album`, `redownload_track`,
  `redownload_album`, `open_folder`, `clear_all`, `remove_cached`,
  `refresh_cached`), plus the `Runtime` type alias.
- `offline_cache/ids.rs` (~70 lines) — the `CACHED_IDS` static, `cached_ids`,
  `is_cached`, `cached_ids_set`, `mark_cached`, `load_cached_ids`. The
  session-wide ready-set is genuinely separable state used from every other
  submodule below via `mark_cached`/`is_cached`.
- `offline_cache/sink.rs` (~60 lines) — `row_sink`, `push_status`,
  `push_unlocking`. The `CacheEventSink` builder that reflects events onto
  visible rows.
- `offline_cache/info.rs` (~20 lines) — `track_cache_info`. Tiny, but a
  clean single-purpose "catalog Track -> DB row" mapper.
- `offline_cache/cache_actions.rs` (~200 lines) — `cache_track`,
  `cache_tracks`, `cache_album`, `cache_playlist`. All the "add to offline
  cache" triggers, which share the pre-flight-limit-check + insert +
  `spawn_track_cache_download` pattern.
- `offline_cache/remove_actions.rs` (~180 lines) — `remove_album`,
  `redownload_track`, `redownload_album`, `open_folder`, `clear_all`,
  `remove_cached`, `refresh_cached`, `remove_cached_inner`. All the
  "remove / re-download / clear" triggers.

## Re-export surface
`offline_cache/mod.rs` becomes the `mod offline_cache;` target; every item
listed above stays reachable at its current `crate::offline_cache::X` path
via `pub use ids::*; pub use sink::*; pub use cache_actions::*; pub use
remove_actions::*;` (the `info.rs` helper is internal-only, no re-export
needed unless another module already reaches for it directly).

## Coupling / watch out
- `CACHED_IDS` (in `ids.rs`) is mutated from `cache_actions.rs` (indirectly,
  via the sink's `Completed` event calling `mark_cached(id, true)`) and from
  `remove_actions.rs` (`mark_cached(id, false)` in several places) — all
  three submodules need `use super::ids::mark_cached;` (or make it `pub(super)`).
- `row_sink` (sink.rs) is passed into every `spawn_track_cache_download` call
  in BOTH `cache_actions.rs` and `remove_actions.rs` (redownload flows) —
  keep it `pub(crate)` and importable from both.
- `remove_actions.rs`'s `redownload_track`/`redownload_album`/`remove_cached`/
  `refresh_cached`/`clear_all` all end by calling
  `crate::offline_manager::rebuild(weak.clone())` — an outbound coupling to
  the OTHER file in this batch (`offline_manager.rs`); don't lose that call
  when moving code, and note for other agents that `offline_manager.rs`'s
  plan (same batch) is the other half of this dependency.
- `refresh_cached` deliberately sequences `remove_cached_inner(..., toast:
  false)` then `cache_track(...)` in ONE spawned task ("the delete must land
  first" — see its doc comment) — keep this ordering guarantee intact,
  don't let the split tempt a "run these on separate tasks" refactor.

## What to verify after the real split
- `cargo build -p qbz` (main.rs and any UI callback wiring that calls
  `offline_cache::cache_track` etc. must resolve unchanged).
- `cargo test -p qbz` (no dedicated unit tests in this file today, but
  ensure the crate-level test suite still passes).
- Manual smoke: cache a single track, cache an album, remove a cached
  track, clear the whole offline cache, and confirm row cache-status badges
  update in the running Slint UI.
