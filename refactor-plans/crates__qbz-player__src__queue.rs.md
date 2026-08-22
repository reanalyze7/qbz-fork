# crates/qbz-player/src/queue.rs (2306 lines)

## Summary
`QueueManager`: the thread-safe (single `Mutex<InternalState>`) playback queue
— add/remove/move tracks, shuffle order management, repeat modes, history for
"previous", the "stop after track" marker, and state snapshots (`get_state`,
`get_state_full`) for the frontend. Production code is lines 1-1195; the rest
(1197-2306, ~1110 lines, 57 `#[test]` fns) is a single `#[cfg(test)] mod tests`.

## Proposed split
By responsibility, all still behind the single `QueueManager` type (methods can
live in multiple `impl QueueManager` blocks across files in the same module dir):

- `queue/mod.rs` (~60 lines) — `InternalState`, `QueueMoveDirection`, the
  `QueueManager` struct + `new`/`Default`/`current`, module wiring
  (`mod mutate; mod playback; mod shuffle; mod state_view; mod internal;`
  `#[cfg(test)] mod tests;`), re-exports `pub use` nothing extra needed since
  everything is inherent methods on `QueueManager` (already reachable via the
  type itself — mod.rs just needs to keep `QueueManager` and `InternalState`
  public/crate-visible as today).
- `queue/mutate.rs` (~330 lines, still needs a further split — see below) —
  `impl QueueManager` block with `add_track`, `add_tracks`, `add_track_next`,
  `set_queue`, `set_queue_with_order`, `clear`, `remove_track`,
  `remove_upcoming_track`, `remove_upcoming_after`, `remove_after`,
  `move_track`.
  - Given 130-line budget, split further into:
    - `queue/mutate/add.rs` (~90 lines): `add_track`, `add_tracks`,
      `add_track_next`.
    - `queue/mutate/replace.rs` (~90 lines): `set_queue`,
      `set_queue_with_order`, `clear`.
    - `queue/mutate/remove.rs` (~180 lines, still over — split into
      `remove.rs` for `remove_track`/`remove_upcoming_track` (~110 lines) and
      `remove_range.rs` for `upcoming_len`, `remove_upcoming_after`,
      `remove_after` (~90 lines)).
    - `queue/mutate/move_track.rs` (~95 lines): `move_track` alone (it's a
      single large method with two branches — shuffle vs linear).
- `queue/playback.rs` (~250 lines, split further) — `current_track`,
  `peek_next`, `peek_upcoming`, `next`, `previous`, `sync_current_to_id`,
  `play_upcoming_at`, `play_index`.
  - Split into `queue/playback/peek.rs` (~110 lines: `current_track`,
    `peek_next`, `peek_upcoming`) and `queue/playback/advance.rs` (~140 lines:
    `next`, `previous`, `sync_current_to_id`, `play_upcoming_at`,
    `play_index` — these are the most bug-history-laden methods, see coupling
    notes below; may need one more split if it runs long).
- `queue/shuffle.rs` (~120 lines) — `set_shuffle`, `set_shuffle_with_order`,
  `is_shuffle`, plus the private internal helpers
  `regenerate_shuffle_order_internal`, `set_identity_shuffle_order_internal`,
  `remove_index_from_shuffle_internal`, `is_valid_shuffle_order`. (These
  internals are used across `mutate` and `playback` too — see coupling.)
- `queue/repeat_and_marker.rs` (~60 lines) — `set_repeat`, `get_repeat`,
  `set_stop_after`, `clear_stop_after`, `get_stop_after`,
  `consume_stop_after_if`.
- `queue/state_view.rs` (~110 lines) — `get_state`, `get_all_tracks`,
  `get_state_full` (the two near-duplicate snapshot builders — a good future
  dedup candidate, but out of scope for a line-count-only split).
- `queue/internal.rs` (~90 lines) — the free `remap_history_by_track_id_internal`,
  `remap_index_after_move` associated functions (already `fn` on
  `QueueManager`, take `&mut InternalState`) that don't fit neatly under one
  of the above (used by both `mutate` and `playback`).
- `queue/tests/mod.rs` + split test files (~1110 lines total, split by theme
  into e.g. `tests/clear.rs`, `tests/history_sync.rs` (the LANE C diagnostic
  block), `tests/move_and_shuffle.rs`, `tests/play_upcoming.rs` — each a
  `#[cfg(test)] mod ...` included from `queue/mod.rs`'s
  `#[cfg(test)] mod tests { mod clear; mod history_sync; ... }`, all using
  `use super::super::*;` to reach `QueueManager`/`create_test_track`). Keep
  `create_test_track` in one shared `tests/mod.rs` or a `tests/common.rs`.

## Re-export surface
`queue/mod.rs` — it already defines `pub struct QueueManager` and is the file
`crates/qbz-player/src/queue.rs` becomes `crates/qbz-player/src/queue/mod.rs`.
Nothing under `crate::queue::` changes shape: `qbz_player::queue::QueueManager`
stays the only public symbol callers use (`InternalState` stays private/
crate-internal, only reachable via `&mut InternalState` params inside the
module tree). No caller-visible API changes at all — this is a pure
in-crate-module reorganization.

## Coupling / watch out
- `InternalState` is NOT `pub`, but every submodule's associated functions
  take `&mut InternalState` / `&InternalState` directly (bypassing the
  `Mutex`) — these must stay `pub(super)` or `pub(crate)` visible from
  sibling submodules once split out of one file. Easiest: keep
  `InternalState` and its fields `pub(crate)` (or `pub(super)` at the
  `queue` module root) so `mutate`, `playback`, `shuffle`, `internal` can all
  reach into it.
- Shuffle bookkeeping is threaded through nearly every mutation (`add_track`,
  `add_track_next`, `remove_track`, `remove_upcoming_track`, `remove_after`,
  `move_track`, `set_queue*`, `set_shuffle*`) — the `shuffle.rs` internal
  helpers (`remove_index_from_shuffle_internal`,
  `regenerate_shuffle_order_internal`, `set_identity_shuffle_order_internal`,
  `is_valid_shuffle_order`) are called from `mutate/*` and must be
  `pub(super)` on `QueueManager` (associated fns), reachable across the split.
- History remapping (`remap_history_by_track_id_internal`) is called from
  `set_queue`, `set_queue_with_order`, and `clear` (all in different proposed
  files) — keep it in `internal.rs` as a `pub(super)` associated fn.
- `remap_index_after_move` is only used by `move_track` — could move into
  `mutate/move_track.rs` directly instead of `internal.rs` if that keeps
  cross-file calls simpler; a judgment call at split time.
- `sync_current_to_id`, `play_index`, and `play_upcoming_at` all touch history
  push/cap logic (the "push to history, cap at 50" pattern is duplicated 3x
  today — `next`, `sync_current_to_id`, `play_index` — a good place to extract
  a shared `push_history(state, idx)` helper into `internal.rs` while
  splitting, so the split doesn't just relocate the duplication).
- `remove_after` is explicitly documented as NOT shuffle-order-aware and kept
  only for existing unit test coverage vs. `remove_upcoming_after` which IS
  the wired UI action — preserve both and their doc comments verbatim across
  the split so a future reader doesn't "fix" the wrong one.
- Test module has bug-regression tests referencing internal `state.lock()`
  directly (`queue.state.lock().unwrap()` in a couple of shuffle tests) —
  these need `QueueManager.state` to stay visible to `tests` (it already is,
  same crate, but once split into `tests/mod.rs` + children ensure the field
  visibility (`pub(crate)` or module-private with `pub(super)`) still reaches
  them through the new nesting).

## Verify after split
- `cargo test -p qbz-player queue::` — all 57 existing tests green, especially
  the "LANE C diagnostic" (`sync_current_to_id`) and shuffle/move tests which
  encode real historical bug fixes (#316, #327) — regressions here are silent
  UI bugs, not compile errors.
- `cargo check -p qbz-player` and any crate depending on
  `qbz_player::queue::QueueManager` (search for `queue::QueueManager` /
  `use qbz_player::queue`).
- Grep for `.state.lock()` direct usages outside the module (should be none —
  confirms `InternalState`/`state` field encapsulation wasn't accidentally
  widened beyond crate-visibility).
