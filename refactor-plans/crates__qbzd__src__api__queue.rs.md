# crates/qbzd/src/api/queue.rs (755 lines)

## Summary
Queue HTTP routes: `GET /api/queue`, `POST /api/queue/{add,remove,clear,
move,jump,stop-after}`. Heavily documented (0-based index convention,
server-side track materialization). ~245 lines of logic, ~510 lines of
tests — the largest file in this gap-fill batch.

## Proposed split
- `mod.rs` (~15 lines) — module declarations only, re-exporting the public
  route functions.
- `list.rs` (~50 lines) — `list` (`GET /api/queue`).
- `add.rs` (~60 lines) — `add` (`POST /api/queue/add`), `parse_track_ids`,
  `parse_position`, `AddPosition`.
- `remove.rs` (~65 lines) — `remove`, `parse_remove_index`,
  `check_remove_index`, `RemoveCheck` (the remove-index gate logic — pure
  and already unit-tested standalone).
- `clear_reorder.rs` (~40 lines) — `clear`, `reorder`.
- `jump_stop_after.rs` (~65 lines) — `jump`, `stop_after`.
- `mapping.rs` (~55 lines) — `track_to_queue_track` (Qobuz `Track` ->
  `QueueTrack`), `repeat_str`.
- `shared.rs` (~20 lines) — `auth_gate`, `paginate`, `parse_offset_limit`
  (small pure/shared helpers used by more than one of the above).
- `tests/` directory, one test file per production module above, mirroring
  which functions they exercise (e.g. `tests_remove.rs` for the
  `check_remove_index`/`parse_remove_index` tests, `tests_mapping.rs` for
  `track_to_queue_track_maps_the_catalog_shape`, etc.) — this is the single
  biggest lever for getting this file under budget, since ~510 of the 755
  lines are tests.

## Re-export surface
`mod.rs` (`crate::api::queue`) re-exports `list`, `add`, `remove`, `clear`,
`reorder`, `jump`, `stop_after`, and `pub(crate) track_to_queue_track` (used
by other `api/*.rs` handlers, per its own doc comment cross-referencing
`qbz-mixtape/src/enqueue.rs`'s independent re-derivation) — verify with
`grep -rn "api::queue::" crates/qbzd/src` before finalizing which symbols
must stay `pub(crate)`.

## Coupling / watch-outs
- `track_to_queue_track` is explicitly documented as one of THREE
  independent re-derivations of the same Track->QueueTrack mapping
  (`qbz/src/playback.rs:2028-2073`, `qbz-mixtape/src/enqueue.rs:430-472`,
  and this one) — do not "fix" the duplication during this split; keep it
  as its own cohesive block exactly as documented.
- `RemoveCheck`/`check_remove_index` bounds-check-wins-over-playing-index
  ordering is a documented, deliberate invariant — keep the doc comment
  when moved to `remove.rs`.
- The narrow race documented in `remove()` (bounds check and the actual
  mutation are two separate core calls) must stay commented exactly as-is
  wherever `remove()` lands — it's load-bearing documentation, not filler.
- `auth_gate` here duplicates the same helper in `api/fav.rs`, `api/
  playback.rs`, `api/browse.rs` — same consolidation opportunity noted
  there, out of scope for this split.

## Verify after split
`cargo test -p qbzd api::queue::` (all ~30 existing tests green);
`cargo build -p qbzd`; smoke-test `qbzd queue add/remove/list/clear` CLI
round-trips.
