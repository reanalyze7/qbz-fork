# crates/qbzd/src/api/mod.rs (671 lines)

## Summary
The qbzd HTTP control-plane core: route tables (P0/P1 constants), the shared
`ApiState`, socket bind/serve lifecycle, the origin-shield + bearer-token
access gate, the big method+path `route()` match dispatching into the
`browse`/`discover`/`fav`/... submodules, and JSON response helpers.

## Proposed split
By responsibility — this file already has clear internal sections (route
tables, state/lifecycle types, bind/serve, routing/gate, response helpers,
tests):

- `api/mod.rs` (~50 lines) — module doc, `pub mod` declarations (unchanged),
  imports, re-exports of the public items from the new submodules below so
  `crate::api::{ApiState, bind, serve, ApiHandle, BindError, json,
  err_json, canon_volume, P0_ROUTES, P1_ROUTES}` all keep resolving.
- `api/routes_table.rs` (~60 lines) — `P0_ROUTES`, `P1_ROUTES` consts + their
  doc comments (the "counted route table" contract).
- `api/state.rs` (~60 lines) — `ApiState`, `DeviceCache`, `BoundServer`,
  `ApiHandle` + its `shutdown`, `BindError`.
- `api/lifecycle.rs` (~90 lines) — `bind`, `classify_bind_error`,
  `probe_is_qbzd`, `serve` (the boot-step-5/11 socket lifecycle + the SSE
  special-case thread spawn inside `serve`'s loop).
- `api/router.rs` (~130 lines) — `route()` (the big match), `read_json_body`.
- `api/gate.rs` (~60 lines) — `GateReject`, `access_gate`, `constant_time_eq`
  (the pre-routing origin/token decision — pure, unit-testable in
  isolation).
- `api/response.rs` (~40 lines) — `canon_volume`, `json`, `err_json`,
  `error_body`.
- `api/tests.rs` (~145 lines) — the entire `#[cfg(test)] mod tests` block,
  included via `#[cfg(test)] mod tests;`, referencing everything through
  `super::*`.

## Re-export surface
`api/mod.rs` stays the public surface: it re-exports `ApiState`, `bind`,
`serve`, `ApiHandle`, `BindError`, `probe_is_qbzd`, `P0_ROUTES`, `P1_ROUTES`,
plus the `pub(crate)` helpers (`json`, `err_json`, `canon_volume`,
`error_body`) that `status.rs` and other route-handler submodules already
import via `crate::api::...` or `super::...`. Check each of `browse.rs`,
`discover.rs`, `fav.rs`, `play.rs`, `playback.rs`, `playlist.rs`, `queue.rs`,
`reco.rs`, `search.rs`, `artwork.rs`, `settings.rs`, `sse.rs`, `status.rs`
for `use super::{json, err_json, ...}` and adjust to `use
super::response::{json, err_json, ...}` (or keep the re-export in `mod.rs`
so those `use super::` lines need no change at all — preferred, since it's
zero-risk for 12 existing submodules).
- **Recommendation**: re-export everything from `mod.rs` rather than making
  submodules reach into `api::response`/`api::gate` directly, so the dozen
  sibling route-handler files (`browse.rs` etc.) require ZERO edits.

## Coupling / watch out
- `serve()`'s SSE special-case (checking `is_events` before the general
  `route()` call, then spawning `sse::stream` on its own thread) is
  entangled with `access_gate` — it calls `access_gate` directly (not
  through `route()`) so the SSE stream gets the same origin/token check
  without going through the JSON-response `route` match. Keep `serve()` and
  `gate.rs`'s `access_gate` importable from `lifecycle.rs`.
- `ApiState` fields have load-bearing doc comments explaining WHY each
  field is `Mutex`/`Arc`/etc. (audio settings WAL note, quality cell used by
  the driver's auto-advance, device cache TTL) — preserve every comment
  verbatim when moving the struct.
- The `route_table_matches_spec_count` and
  `p1_route_table_grows_only_with_a_shipped_caller` tests are a hard
  workspace invariant (comments call out prior "68-routes" regressions) —
  when moving to `tests.rs`, do not accidentally lose the exact counts
  (17 / 26) or the per-route `assert!(contains(...))` lines.
- `canon_volume`'s f32-widening comment is important context for
  `response.rs` — keep it.

## Verify after split
- `cargo test -p qbzd api` — every existing test (route counts, gate
  behavior, canon_volume rounding, error envelope shape) green.
- `cargo check -p qbzd` (compiles the 12 sibling route-handler submodules
  against the new re-export surface).
- Smoke-test: start qbzd, `curl localhost:PORT/api/ping`,
  `curl -H 'Origin: http://x' .../api/ping` (expect 403), and one token-gated
  route if `[server] token` is configured.
