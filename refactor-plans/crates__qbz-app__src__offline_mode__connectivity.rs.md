# crates/qbz-app/src/offline_mode/connectivity.rs (598 lines)

## Summary
The Rust-owned connectivity actor that replaces Tauri's flaky poll-only
checker: a pure decision core (`ConnectivityJudge`) layered with an OS
default-route check, passive audio-liveness detection, a hardened
multi-endpoint HTTP probe set, asymmetric up/down hysteresis, and a
suspend/resume guard, broadcasting state over a `tokio::sync::watch` channel
via `ConnectivityActor`.

## Proposed split
This is close to a textbook pure/IO split already — the doc comment itself
enumerates 5 layers.

- `connectivity/mod.rs` (~40 lines) — module declarations, re-exports of
  `Connectivity`, `ConnectivitySnapshot`, `ConnectivityJudge`, `JudgeAction`,
  `ProbeOutcome`, `ConnectivityActor`, `has_default_route`, `probe_all`.
- `connectivity/judge.rs` (~130 lines) — PURE decision core: `Connectivity`,
  `ConnectivitySnapshot` (+ `Default`), `ProbeOutcome`, `JudgeAction`,
  `ConnectivityJudge` struct + its impl (`new`, `snapshot`, `on_no_route`,
  `on_liveness`, `reset_streak`, `on_probe`) + `Default`. Zero I/O, fully
  unit-testable in isolation — this is the file's real "pure computation"
  core.
- `connectivity/route.rs` (~60 lines) — OS route signal: `ipv4_has_default_route`,
  `ipv6_has_default_route`, `has_default_route`. Pure parsing functions plus
  one `/proc` read.
- `connectivity/probe.rs` (~110 lines) — HTTP probing (I/O): `ProbeExpect`,
  `ProbeEndpoint`, `PROBES` const, `PROBE_TIMEOUT`, `probe_endpoint`,
  `probe_all`, `audio_liveness_recent`.
- `connectivity/actor.rs` (~120 lines) — the actor/orchestration layer:
  `LIVENESS_WINDOW_SECS`, `POLL_INTERVAL`, `CONFIRM_DELAYS`, `RESUME_JUMP`
  constants, `ConnectivityActor` struct + `spawn`/`subscribe`/`snapshot`/
  `request_recheck`. This is the tokio task that wires judge + route + probe
  together.
- `connectivity/tests.rs` (~125 lines) — the existing `#[cfg(test)] mod
  tests` block (judge state-machine tests + route-parsing tests), included
  via `#[cfg(test)] mod tests;` from `mod.rs`.

## Re-export surface
`connectivity/mod.rs` is the public-API surface — re-export every currently
public item (`Connectivity`, `ConnectivitySnapshot`, `ConnectivityJudge`,
`JudgeAction`, `ProbeOutcome`, `ConnectivityActor`, `has_default_route`,
`probe_all`) at the same path so `crate::offline_mode::connectivity::X`
(or however the parent `offline_mode` module re-exports it) keeps working
unchanged.

## Coupling / watch out
- The constants (`LIVENESS_WINDOW_SECS`, `POLL_INTERVAL`, `CONFIRM_DELAYS`,
  `RESUME_JUMP`, `PROBE_TIMEOUT`) are read from multiple layers (judge uses
  none directly but `actor.rs` uses all of them, `probe.rs` uses
  `PROBE_TIMEOUT` and `LIVENESS_WINDOW_SECS` via `audio_liveness_recent`) —
  decide one home (`actor.rs` is the natural one since it owns the loop) and
  have `probe.rs` import `super::actor::LIVENESS_WINDOW_SECS` or move
  `audio_liveness_recent` itself into `actor.rs` instead, since it's really
  a decision input, not a probe.
- `ConnectivityJudge::on_probe` is the crux of the whole design (asymmetric
  hysteresis, confirmation burst, time-bounded streak) — keep it intact,
  don't split it further even though it's the longest single function.
- The tests directly exercise `ConnectivityJudge` and the two route-parsing
  functions — after the split they need `use super::judge::*;` and
  `use super::route::*;`.
- `qbz_audio::network_throttle::state()` dependency in
  `audio_liveness_recent` is an external crate coupling — no change needed,
  just note it crosses crate boundaries so don't accidentally break the
  import path when moving the function.

## Verify after split
- `cargo build -p qbz-app`
- `cargo test -p qbz-app connectivity` (all existing judge + route-parsing
  tests green, unchanged assertions)
- Grep for `ConnectivityActor::spawn` / `subscribe` call sites in the
  offline-mode engine to confirm the actor's public API is unchanged.
- Manual smoke test: toggle network off/on and confirm the app's offline
  banner still reacts within the documented hysteresis window.
