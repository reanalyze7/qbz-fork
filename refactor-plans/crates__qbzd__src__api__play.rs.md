# `crates/qbzd/src/api/play.rs` (296 lines)

POST `/api/play`: resolves a play selector (track/album/playlist/artist/url) to catalog
tracks, materializes them into the queue, and cold-starts playback.

## Proposed split

- `play.rs` (~90 lines) — public `pub fn play()` handler + `pub(crate) fn start_resolved()`
  (the queue-materialize + cold-start ritual). Stays the re-export/public surface: this is
  what `super::play` / the router imports.
- `play/selector.rs` (~90 lines) — `Selector` enum, `parse_selector`, `fetch_tracks`,
  `not_found`, `ARTIST_TOP_LIMIT`. Pure parsing/resolution logic, no HTTP response building
  beyond error envelopes.
- `play/util.rs` (~40 lines) — `clamp_index`, `summary`, `auth_gate` — small shared helpers.
- Move the `#[cfg(test)] mod tests` block to sit alongside whichever module it tests most
  (mostly `selector.rs` + `util.rs`); a thin integration-style test can stay in `play.rs`.

## Coupling to flag

- `start_resolved` is called from elsewhere too (it's `pub(crate)`) — check other callers
  under `crates/qbzd/src/api/` before moving it, so the re-export path still resolves.
- Depends on `super::queue::track_to_queue_track` and `super::{err_json, json, ApiState}` —
  keep those imports intact in whichever submodule keeps `start_resolved`.

## Verify after split

- `cargo test -p qbzd` (unit tests for `parse_selector`/`clamp_index` still pass).
- `cargo build -p qbzd` — confirm `super::play::start_resolved` callers still resolve.
