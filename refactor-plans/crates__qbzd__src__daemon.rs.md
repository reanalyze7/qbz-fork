# crates/qbzd/src/daemon.rs (1063 lines)

## Summary
The `qbzd run` boot sequence (normative numbered steps: lock, port bind,
runtime composition, credential/session restore, playback driver, queue
persistence, scrobbler, MPRIS, HTTP serve, signal park, ordered shutdown)
plus the `POST /api/settings/reload` reconciliation logic (T11), with a
substantial `#[cfg(test)]` module (~200 lines) covering the pure decision
functions.

## Proposed split
Turn into a `daemon/` directory, split along the two life-cycle phases the
file itself already documents (boot/run vs. reload), plus tests:

- `daemon/mod.rs` (~90 lines) — `BootedRuntime` struct, module doc, `use`
  block, `pub use` re-exports of `run`, `boot`-adjacent pub(crate) fns
  needed by other daemon modules (`restore_activate`, `set_needs_auth`,
  `set_logged_in`, `latch_undecryptable_token`, `latch_auth_error`,
  `is_auth_rejection`) so `reload.rs` can call them.
- `daemon/run.rs` (~180 lines) — the public `run()` fn (lines 42-216): argv/
  config/lock/bind/boot/driver-spawn/subsystem-spawn/HTTP-serve/signal-park/
  ordered-shutdown. This is the top-level orchestrator; keep it whole even
  though it's long — splitting the ordered shutdown sequence away from `run`
  would obscure the #521 ordering invariant that's the whole point of the
  comments.
- `daemon/boot.rs` (~130 lines) — `boot()` (lines 221-320): store/runtime
  composition, artist-vector store, credential restore branch.
- `daemon/driver_deps.rs` (~40 lines) — `build_driver_deps` (lines 326-360).
- `daemon/queue_persist.rs` (~45 lines) — `spawn_queue_persist` (lines
  368-403).
- `daemon/session.rs` (~130 lines) — `restore_activate`, `new_shared`,
  `set_needs_auth`, `set_logged_in`, `latch_undecryptable_token`,
  `latch_auth_error`, `is_auth_rejection`, `spawn_auth_retry` (lines
  405-594) — the credential/session state-machine helpers shared by boot,
  retry, and reload.
- `daemon/bind.rs` (~50 lines) — `resolve_bind_addr`, `diagnose_port_conflict`,
  `diagnose_lock`, `wait_for_signal` (lines 596-671).
- `daemon/reload.rs` (~175 lines) — the entire T11 section (lines 673-862):
  `reload`, `reload_audio`, `audio_routing_changed`, `reload_quality`,
  `CredentialAction`, `decide_credential_action`, `reload_credentials`.
- `daemon/tests.rs` (~200 lines) — the `#[cfg(test)] mod tests` block (lines
  864-1063), referencing items via `super::*`; split further into
  `tests/session.rs` + `tests/reload.rs` if a reviewer wants ownership to
  mirror the split above, but a single `tests.rs` is fine at ~200 lines.

## Re-export surface
`daemon/mod.rs` is what `crates/qbzd/src/main.rs` already imports as
`crate::daemon::{run, ...}` — re-export `run` and any `pub(crate)` items
other `qbzd` modules reach into (`crate::daemon::set_needs_auth` etc., if
referenced from `api.rs`/`cli.rs`) so no caller path changes.

## Coupling / watch out
- `is_auth_rejection`, `set_needs_auth`, `latch_auth_error`,
  `restore_activate` are shared across `boot.rs`, `session.rs`
  (spawn_auth_retry), and `reload.rs` — put them in ONE module
  (`session.rs`) and have the others `use super::session::*` or
  `use crate::daemon::session::*`.
- The ordered-shutdown block inside `run()` documents a strict drop-order
  invariant (#521: driver → queue_persist → scrobbler → mpris →
  save_session_now → auth_retry → api.shutdown() → drop(booted) → linux
  clock-release) tied to `Arc<AppRuntime>` refcounts. Do not extract any
  piece of this sequence into a separate function that could get reordered
  by a future edit — keep the whole sequence inline in `run.rs` with the
  existing comments intact.
- `#[cfg(test)] mod tests` uses `super::*`, so after the split it needs
  `use super::*;` to resolve into whichever module(s) it now lives beside —
  if split into multiple test files, each needs `use super::super::X::*;`
  for the functions it exercises (session fns, reload fns).

## Verify after split
- `cargo test -p qbzd daemon` — all existing tests (is_auth_rejection,
  credential_action_*, audio_routing_changed_*, logged_in, needs_auth) green.
- `cargo build -p qbzd` and a real `qbzd run` smoke test (login flow, clean
  shutdown via SIGTERM, `settings/reload` via the CLI) since the ordering
  guarantees here are only partially covered by unit tests.
