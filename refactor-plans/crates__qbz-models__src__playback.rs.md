# crates/qbz-models/src/playback.rs (156 lines)

## Summary
Playback-related shared types: `QueueTrack` (the big one, ~55 fields+docs),
`RepeatMode`, `QueueState`, `PlaybackState`, `PlaybackStatus` — all plain
serde data structs with no logic, used across qbzd/qbz-app/qbz-ui.

## Proposed split
Just over budget, driven almost entirely by `QueueTrack`'s doc comments.
Split by "what stage of playback owns this shape" — pure data, so a plain
by-domain split (no pure/IO axis applies, there is no IO here):

- `playback/mod.rs` (~10 lines) — module doc + `pub use` re-exports of every
  item below, so `qbz_models::playback::QueueTrack` etc. keep working.
- `playback/queue_track.rs` (~65 lines) — `QueueTrack` struct + its doc
  comments + `default_streamable()`.
- `playback/queue_state.rs` (~20 lines) — `RepeatMode` (+ `Default` impl) and
  `QueueState`.
- `playback/status.rs` (~50 lines) — `PlaybackState` (+ `Default`) and
  `PlaybackStatus` (+ `Default` impl).

## Re-export surface
`playback/mod.rs` re-exports `QueueTrack`, `RepeatMode`, `QueueState`,
`PlaybackState`, `PlaybackStatus` at `crate::playback::*` (or wherever
`qbz_models` currently re-exports from — check `qbz-models/src/lib.rs` for a
top-level `pub use playback::*;` and keep it unchanged).

## Coupling / watch out
- `QueueTrack` is serialized/deserialized across the qbzd HTTP API, MPRIS
  bridge, and the Slint UI models — field names and `#[serde(default)]`
  attributes must be preserved byte-for-byte; do not reorder in a way that
  changes anything observable (derive-based serde is order-independent for
  JSON, so this is low risk, but double check any bincode/postcard usage
  elsewhere in the workspace that might be order-sensitive).
- The trailing comment "Audio backend types ... defined in qbz-audio crate"
  is a cross-crate note — keep it in `mod.rs` since it's about the module's
  boundary, not any one struct.
- No shared mutable state; this is the easiest file in the batch.

## Verify after split
- `cargo check -p qbz-models` and `cargo check --workspace` (many crates
  depend on this).
- `cargo test -p qbz-models`.
- Grep for `qbz_models::playback::` / `qbz_models::QueueTrack` etc. across
  the workspace to confirm no import paths broke.
