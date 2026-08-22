# crates/qbzd/src/state.rs (163 lines)

## Summary
Shared in-memory daemon state: `DaemonShared` struct (one
`Arc<Mutex<DaemonShared>>` used by the playback driver + HTTP API),
`AuthState` enum, `LatchedErrors`, and `token_fingerprint`. ~65 lines of
logic, ~87 lines of tests. Barely over budget.

## Proposed split
Given the small size, a light 2-file split is enough:
- `mod.rs` (~70 lines) — `LatchedErrors`, `AuthState`, `DaemonShared` (struct
  + its two inherent methods `network_online`/`set_network_online`),
  `token_fingerprint`.
- `tests.rs` (~87 lines) — existing `#[cfg(test)] mod tests`, moved as-is.

If a deeper split is wanted later: `latched_errors.rs` (the
`LatchedErrors`/`AuthState` types) + `shared.rs` (`DaemonShared` itself) +
`fingerprint.rs` (`token_fingerprint`), each well under 130 lines, but at 65
logic lines this is optional.

## Re-export surface
`mod.rs` re-exports everything at `crate::state::*` exactly as today
(`DaemonShared`, `AuthState`, `LatchedErrors`, `token_fingerprint`) — no
caller changes needed (`crate::state::AuthState` is imported from
`api/fav.rs`, `api/queue.rs`, and others).

## Coupling / watch-outs
- `DaemonShared` has NO `#[derive(Default/Clone)]` (comment notes
  `Instant` isn't `Serialize`) — the test `daemon_shared_holds_the_fields...`
  is a compile-time guard that the field set matches `api::status::assemble`.
  Any split must not accidentally change field types/order.
- `network_online`/`set_network_online` intentionally use `Relaxed`
  ordering under the outer `Mutex<DaemonShared>` guard — keep the doc
  comment explaining why, since it looks under-synchronized without it.

## Verify after split
`cargo test -p qbzd state::`; grep `crate::state::` across `qbzd/src` to
confirm no import breaks.
