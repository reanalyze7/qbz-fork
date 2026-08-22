# crates/qbzd/src/api/playback.rs (546 lines)

## Summary
The daemon's HTTP handlers for `GET /api/now-playing` and the 8
`POST /api/playback/*` transport routes (play/pause/toggle/stop/next/
previous/seek/volume) plus shuffle/repeat, including the mute/volume
nominal-value bookkeeping and the DSD-direct guards.

## Proposed split
By responsibility: read route vs. transport routes vs. mute/volume math vs.
shared error/gate internals vs. tests.

- `playback/mod.rs` (~15 lines) — module doc (lines 1-27, kept intact — it
  documents route-to-spec mapping and the DSD-direct/mute design decisions
  future readers need), `pub use` of the route fns from submodules.
- `playback/now_playing.rs` (~50 lines) — `now_playing()`.
- `playback/transport.rs` (~100 lines) — `play`, `pause`, `toggle`, `stop`,
  `next`, `previous` (the simple state-transition routes) + `advance` +
  `cold_start` (their shared internals — these two are only called from
  `next`/`previous`/`play`/`toggle`, so keep them in the same file as their
  callers).
- `playback/seek.rs` (~35 lines) — `seek()`.
- `playback/volume.rs` (~90 lines) — `volume()`, `apply_mute()`,
  `nominal_volume()` (the mute/nominal-volume bookkeeping is tightly coupled
  — keep together).
- `playback/queue_modes.rs` (~40 lines) — `shuffle()`, `repeat()`,
  `repeat_str()`.
- `playback/errors.rs` (~30 lines) — `auth_gate`, `device_error`,
  `runtime_error` (shared response-builder internals used across every
  route file).
- `playback/tests.rs` (~65 lines) — the `#[cfg(test)] mod tests` block
  (`repeat_str_matches_contract_lowercase`, `sample_event`,
  `now_playing_map_path_serializes_canonical_volume`,
  `volume_post_response_serializes_canonical_volume`), declared via
  `#[cfg(test)] mod tests;` in `mod.rs`, importing from the relevant
  submodules (`super::queue_modes::repeat_str`, `super::now_playing`, etc.).

## Re-export surface
`playback/mod.rs` re-exports `now_playing`, `play`, `pause`, `toggle`,
`stop`, `next`, `previous`, `seek`, `volume`, `shuffle`, `repeat`, and the
`pub(crate) fn resolve_quality` at `crate::api::playback::*` — the
`crates/qbzd/src/api/mod.rs` router (which dispatches on path) keeps
importing from the same `playback::` path unchanged.

## Coupling / watch out
- `resolve_quality` is `pub(crate)` and used by `transport.rs`'s
  `advance`/`cold_start` — keep it exported from `mod.rs` (or
  `transport.rs` directly) since it may also be used by sibling API files
  outside this chunk (check other `api/*.rs` files before finalizing which
  submodule owns it).
- `auth_gate` is called from `now_playing.rs` AND `transport.rs`'s `advance`
  — both need `use super::errors::auth_gate;`. Same shape as `browse.rs`'s
  own private `auth_gate` (a near-duplicate in a different file/module —
  NOT shared code today; do not accidentally merge them into one shared
  helper during this split unless a separate refactor decides to).
- `canon_volume` and `err_json`/`json`/`ApiState` are imported from the
  parent `super::{...}` (the `api` module) — every new submodule needs the
  same `use super::{canon_volume, err_json, json, ApiState};` import line.
- The DSD-direct guard (`is_dsd_direct_active`) appears in both `seek.rs`
  and `volume.rs` independently — no shared helper today, so splitting is
  safe, but note it for consistency if a future refactor wants to dedupe.
- `state.shared.lock()` (the `DaemonShared` mutex holding `muted` /
  `premute_volume` / `last_errors`) is touched by `volume.rs` (mute state),
  `transport.rs` (`last_errors.stream` on advance/cold-start failure), and
  `errors.rs`'s `auth_gate` — this is a cross-file shared-state pattern to
  watch during the actual split (lock scoping must not change).

## Verify after split
- `cargo test -p qbzd api::playback::` — all 4 tests green.
- `cargo check -p qbzd` and confirm `crate::api::playback::{now_playing,
  play, pause, toggle, stop, next, previous, seek, volume, shuffle,
  repeat}` still resolve from the router in `crates/qbzd/src/api/mod.rs`.
- Manually smoke-test via `qbzd` CLI (`qbzd play`, `qbzd volume --set 0.5`,
  `qbzd mute toggle`) against a running daemon if feasible.
