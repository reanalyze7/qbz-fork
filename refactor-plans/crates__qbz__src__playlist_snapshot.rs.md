# crates/qbz/src/playlist_snapshot.rs (142 lines)

## Summary
Slint-side glue over `qbz_library::qobuz_playlist_snapshot`: two
fire-and-forget "producer" fns (record playlist names, replace one
playlist's membership) and four blocking "consumer" fns (read headers/name,
compute which playlists have offline-playable tracks) for the offline-mode
port's sidebar/playlist-manager/mixed-playlist-detail views.

## Proposed split
Only 12 lines over budget — a light two-way split by producer/consumer
role is enough, no need for a directory:

- `playlist_snapshot.rs` (~75 lines) — keep the module doc, the `pub use
  repo::SnapshotNameEntry;` re-export, and the two PRODUCER fns
  (`record_names_detached`, `record_detail_detached`, lines 24-76) — these
  share the same "spawn on a blocking thread or `spawn_blocking`, log on
  error" pattern and are the smaller half.
- `playlist_snapshot/consumers.rs` (~65 lines) — lines 78-142: the four
  blocking CONSUMER fns (`headers_blocking`, `name_blocking`,
  `available_offline_blocking`, `playable_track_ids_blocking`) — grouped
  since all four share the "call from `spawn_blocking`" contract and read
  (never write) via `crate::library_db::with_db`.
  (If a `mod.rs` is preferred over a bare `playlist_snapshot.rs` +
  `playlist_snapshot/consumers.rs` sibling, restructure as
  `playlist_snapshot/mod.rs` (producers) + `playlist_snapshot/consumers.rs`
  — either layout works in Rust; pick whichever matches this crate's
  existing convention for other small multi-file modules.)

## Re-export surface
`playlist_snapshot.rs` (or `playlist_snapshot/mod.rs`) stays the
`crate::playlist_snapshot` path. It must `pub use consumers::*;` so
`playlist_snapshot::headers_blocking`, `::name_blocking`,
`::available_offline_blocking`, `::playable_track_ids_blocking` remain
reachable at their current paths for the sidebar/playlist-manager/detail
view callers, alongside the producer fns staying directly in the top file
and the existing `pub use repo::SnapshotNameEntry;` re-export.

## Coupling / watch out
- Both halves depend on `crate::library_db::with_db` and
  `qbz_library::qobuz_playlist_snapshot as repo` — keep the `use` statements
  in both files (cheap, no real coupling risk).
- `available_offline_blocking` and `playable_track_ids_blocking` (the
  consumer half) both call `crate::offline_mode::offline_playback_allowed()`
  and `crate::offline_cache::cached_ids_set()` — an early-return-empty guard
  shared by both; if consumers.rs is split out, no shared helper needs
  extracting (the guard is only 2 lines, duplicated fine) but note both
  fns encode the same "B8: cache may not serve full tracks past the grace
  window" business rule (D4) — keep both doc comments referencing it.
- The producer fns' "run on blocking thread if not already in a tokio
  context, else `spawn_blocking`" branch (`tokio::runtime::Handle::
  try_current().is_ok()`) is repeated identically in both
  `record_names_detached` and `record_detail_detached` — small enough that
  extracting a shared `fn detach(f: impl FnOnce() + Send + 'static)` helper
  is optional polish, not required for the 130-line split, but worth
  flagging as an easy simplify-pass follow-up.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz`.
- Smoke-test: view the sidebar/playlist manager while offline (or with the
  offline cache populated) and confirm playlist names still resolve and the
  offline-availability badge/filter still reflects the correct subset of
  playlists.
