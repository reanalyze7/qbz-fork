# crates/qbzd/src/api/status.rs (387 lines)

## Summary
Implements `GET /api/info` and `GET /api/status` (the daemon's composite
status contract, 02-cli-and-api.md §3.3.3): the `StatusDoc`/`AuthStatus`/
`AudioStatus`/`PlaybackStatus`/`NetworkStatus` serde shapes, the live
assembler (`assemble_live`) that composes them from `DaemonShared` + the
player + queue + audio store + a TTL device-name cache, and helper
labellers for bit-perfect mode / backend / device presence.

## Proposed split
257 lines over budget.

- `status/mod.rs` (~90 lines) — the five `#[derive(Serialize)]` structs
  (`StatusDoc`, `AuthStatus`, `AudioStatus`, `PlaybackStatus`,
  `NetworkStatus`) — pure data shapes, the contract's serde surface.
- `status/handlers.rs` (~60 lines) — `info()` and `status()`, the two
  `tiny_http`-facing entry points (HTTP glue only, delegates assembly).
- `status/assemble.rs` (~110 lines) — `assemble_live`: the core composition
  function (DaemonShared snapshot -> player snapshot -> queue -> audio
  config -> playback state), unchanged logic, `use super::{StatusDoc, ...}`.
- `status/device_cache.rs` (~50 lines) — `device_is_present`,
  `cached_device_names`: the 5s TTL CPAL-enumeration cache, isolated since
  it's the one piece with its own mutable cache state (`state.devices`).
- `status/labels.rs` (~35 lines) — `bitperfect_label`, `backend_label`: pure
  enum-to-string mappers.
- `status/tests.rs` (~115 lines, `#[cfg(test)] mod tests`) — unchanged,
  included from `mod.rs` via `#[cfg(test)] mod tests;` (or kept as separate
  per-concern test modules next to their functions if that reads better —
  either is fine as long as all 4 tests stay green).

## Re-export surface
`status/mod.rs` re-exports `StatusDoc`, `AuthStatus`, `AudioStatus`,
`PlaybackStatus`, `NetworkStatus`, and `pub fn info`/`pub fn status` (via
`pub use handlers::{info, status};` or by keeping `info`/`status` directly
in `mod.rs` and only extracting the private helpers) — the `crate::api::
status::{info, status}` route-registration call sites in the daemon's HTTP
router are unaffected.

## Coupling / watch out
- `super::ApiState`, `super::json`, `super::canon_volume` are referenced
  throughout — every new file needs `use super::...` (the parent `api`
  module), keep those imports intact per file.
- The `f32` → JSON `Number::from_f32` widening bug workaround
  (`super::canon_volume` pointer-overwrite in `status()`) is load-bearing —
  don't "simplify" it away when moving `status()` into `handlers.rs`; the
  pinned test (`status_doc_playback_volume_serializes_canonically`) will
  catch a regression but preserve the comment explaining why.
- `assemble_live` explicitly drops the `state.shared.lock()` guard BEFORE
  any `.await` (documented in its doc comment) — this ordering constraint
  must survive the move into `assemble.rs` verbatim.
- `device_is_present`/`cached_device_names` take `&super::ApiState` and
  reach into `state.devices` (a mutex-guarded cache struct defined
  elsewhere in `api/mod.rs` presumably) — confirm that struct's field
  visibility (`.at`, `.names`) stays `pub(crate)` or equivalent across the
  module boundary.

## Verify after split
- `cargo check -p qbzd` and `cargo test -p qbzd status` (all 4 existing
  tests green, including the two contract-shape pinning tests).
- Smoke-test: `curl localhost:<port>/api/status` against a running `qbzd`
  and diff the JSON shape against the 02-cli-and-api.md §3.3.3 examples.
- Grep for `api::status::` importers (the HTTP router in `qbzd`) to confirm
  route registration still resolves.
