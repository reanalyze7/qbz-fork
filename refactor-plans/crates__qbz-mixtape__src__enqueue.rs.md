# crates/qbz-mixtape/src/enqueue.rs (691 lines)

## Summary
Resolves a `MixtapeCollection`'s items into a flat `Vec<QueueTrack>`: the
`ItemResolver` trait (mockable for tests), item-order shuffling, prev/next
item-boundary detection for skip navigation, the production resolver
(`ProdItemResolver`) dispatching to async Qobuz calls or a caller-supplied
sync local closure, the four Qobuz/local resolver free functions, and two
shared `Track`/`LocalTrack` -> `CoreQueueTrack` mapping helpers.

## Proposed split
561 lines over budget. The file already has clear `// ── section ──`
banners marking natural boundaries — the split follows them directly.

- `enqueue/mod.rs` (~60 lines) — `ItemResolver` trait, `resolve_collection_
  tracks` (the orchestrator: shuffle-if-needed + resolve-each + stamp hint +
  flatten), and `shuffle_items`. This is the small public-facing orchestration
  entry point most callers actually use.
- `enqueue/boundary.rs` (~65 lines) — `next_item_index`, `previous_item_index`:
  the skip-to-item-boundary pure logic (no I/O, easy to unit test alone).
- `enqueue/prod_resolver.rs` (~65 lines) — `ProdItemResolver<'a, L>` struct +
  its `new` + the `ItemResolver` impl (the `match (item_type, source)`
  dispatch table).
- `enqueue/qobuz.rs` (~120 lines) — `resolve_qobuz_album`,
  `resolve_qobuz_track`, `resolve_qobuz_playlist` (the three async Qobuz
  resolver free fns).
- `enqueue/local.rs` (~75 lines) — `resolve_local_album`,
  `resolve_local_album_tracks`, `resolve_local_track`, `resolve_local_item`
  (the synchronous local-DB resolver family + the full item-type dispatch
  matrix documented as "centralized so frontends do NOT re-implement it").
- `enqueue/mapping.rs` (~90 lines) — `track_to_queue_track_from_api`,
  `local_track_to_queue_track` (the two shared Track->CoreQueueTrack mappers
  used by both the Qobuz and local resolvers).
- `enqueue/tests.rs` (~195 lines, `#[cfg(test)] mod tests`) — unchanged,
  included via `#[cfg(test)] mod tests;` from `mod.rs`; large but this is
  test code, not counted against the same pressure as production files by
  most teams — still, if the 130-line rule applies uniformly, further split
  into `tests/mock_resolver.rs` + `tests/boundary_tests.rs` + `tests/
  collection_tests.rs`.

## Re-export surface
`enqueue/mod.rs` re-exports everything currently `pub`: `ItemResolver`,
`resolve_collection_tracks`, `shuffle_items`, `next_item_index`,
`previous_item_index`, `ProdItemResolver`, `resolve_qobuz_album`,
`resolve_qobuz_track`, `resolve_qobuz_playlist`, `resolve_local_album`,
`resolve_local_album_tracks`, `resolve_local_track`, `resolve_local_item`,
`track_to_queue_track_from_api`, `local_track_to_queue_track` — via `pub use
{boundary::*, prod_resolver::*, qobuz::*, local::*, mapping::*};` so
`crate::enqueue::{...}` / `qbz_mixtape::enqueue::{...}` call sites in
Slint's `playback.rs` (queue-building) and elsewhere are unaffected.

## Coupling / watch out
- `ProdItemResolver`'s `local: L` closure bound
  (`Fn(&MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String> + Send
  + Sync`) is the crate's key frontend-agnostic seam — the long doc comment
  explaining why `&LibraryDatabase` can never cross an `.await` is
  load-bearing context; keep it attached to the struct in `prod_resolver.rs`,
  don't shorten it during the move.
- `resolve_local_item`'s Playlist arm returns a hard error string ("local
  playlists not supported in this release") that is pinned by a test
  (`resolve_local_item_playlist_is_unsupported`) — do not "fix" or reword
  this during the split.
- `next_item_index`/`previous_item_index` both fall back from
  `source_item_id_hint` to `album_id` when the hint is absent — this
  fallback rule is duplicated in both functions; note it as a candidate for
  a shared `boundary_of` helper extracted to `boundary.rs`'s top, but don't
  change behavior, just de-duplicate the closure if convenient.
- `MockResolver` in the test module is used across several tests — keep it
  in one place (`tests/mock_resolver.rs` if the test file itself is split)
  rather than duplicating it.

## Verify after split
- `cargo test -p qbz-mixtape enqueue` (all 8 existing tests green: the two
  `resolve_local_item` error-path tests, hint-stamping/flattening,
  album-shuffle contiguity, and the four boundary-navigation tests).
- `cargo check -p qbz-mixtape`
- Grep for `qbz_mixtape::enqueue::` importers (the Slint queue/playback
  controller building mixtape queues) to confirm nothing broke.
