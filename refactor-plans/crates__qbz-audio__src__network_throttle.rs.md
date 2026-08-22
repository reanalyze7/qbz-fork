# crates/qbz-audio/src/network_throttle.rs (264 lines)

## Summary
Adaptive prefetch-concurrency throttle: watches EMA-smoothed per-segment
bandwidth and audio-underrun events to compute a dynamic prefetch cap
(0..=memory-profile-default), with TCP-slow-start-style panic/recovery
behavior, exposed via a process-global `ThrottleState` singleton.

## Proposed split
The file is small (264 lines) and already cleanly divided into "pure data
+ config constants", "the stateful singleton + its methods", and "tests".
Split by pure/IO(state)/tests per the project's stated pure/IO/render
convention (there's no "render" concern here, so it's pure/state/tests):

- `network_throttle/mod.rs` (~65 lines) — module doc, the tuning constants
  (`BANDWIDTH_EMA_ALPHA`, `PANIC_WINDOW_SECS`, `SURVIVING_RATIO`,
  `CAUTIOUS_RATIO`, `RELAXED_RATIO`), `playback_mbps_for_quality()` (pure
  fn), `PlaybackQualityTag` enum, and `pub use` of `ThrottleState`/`state()`
  from the state submodule below.
- `network_throttle/state.rs` (~120 lines) — `ThrottleInner` struct,
  `ThrottleState` struct, the `GLOBAL: OnceLock<ThrottleState>` singleton +
  `state()` accessor, and the full `impl ThrottleState { record_segment_bandwidth,
  record_underrun, current_bandwidth_mbps, seconds_since_download,
  in_panic_mode, current_prefetch_cap }` block — this is the one stateful
  "IO-ish" (shared mutable state via RwLock) piece of the file, kept whole
  since its methods are tightly coupled through `self.inner`.
- `network_throttle/tests.rs` (~85 lines) — the existing `#[cfg(test)] mod
  tests` block (8 tests) moved verbatim, each constructing its own
  `ThrottleState { inner: RwLock::new(ThrottleInner::default()) }` directly
  rather than via the singleton (already isolated, so this move is
  mechanical).

## Re-export surface
`network_throttle/mod.rs` is the target of the existing `mod
network_throttle;` (or `pub mod network_throttle;`) declaration in
`crates/qbz-audio/src/lib.rs`. The prefetch dispatcher (whatever calls
`qbz_audio::network_throttle::state().current_prefetch_cap(...)` or
similar) and any code calling `playback_mbps_for_quality()` /
`PlaybackQualityTag` must keep working via `pub use state::{ThrottleState,
state};` in `mod.rs` — no external call site needs to change.

## Coupling / watch out
- `ThrottleInner` fields (`bandwidth_ema_mbps`, `last_underrun`,
  `last_successful_download`) are private to the module and only ever
  touched through `ThrottleState`'s methods via `self.inner.write()`/
  `.read()` — keep `ThrottleInner` and `ThrottleState` in the same file
  (`state.rs`) since they're a tight pair; do not split the struct
  definition from its impl block.
- `current_prefetch_cap()` reads BOTH `in_panic_mode()` (state.rs) AND the
  module-level ratio constants (`SURVIVING_RATIO`/`CAUTIOUS_RATIO`/
  `RELAXED_RATIO`, proposed to live in `mod.rs`) — these constants need
  `pub(super)` or `pub(crate)` visibility, or simply `use super::*;` in
  `state.rs`.
- `GLOBAL: OnceLock<ThrottleState>` is process-global singleton state — the
  split doesn't change its singleton nature, but flag for whoever
  implements it: don't accidentally create a second `OnceLock` if
  `state()` is referenced from multiple new files (it shouldn't be; only
  `state.rs` needs it, and `mod.rs` re-exports the accessor fn, not the
  static itself).
- `seconds_since_download()` / `last_successful_download` implements the
  "positive liveness signal" for the offline detector (issue #467,
  referenced in comments twice) — this is a cross-crate concern (some
  offline-detection code elsewhere presumably calls
  `state().seconds_since_download()`) — grep for that call site before
  splitting to confirm the accessor stays public at the same path.

## Verify after split
- `cargo test -p qbz-audio` — all 8 existing tests
  (`fresh_state_returns_default_cap`, `panic_mode_zeros_cap`,
  `surviving_ratio_zeros_prefetch`, `cautious_ratio_allows_one`,
  `relaxed_ratio_allows_two`, `abundant_bandwidth_unlocks_default`,
  `cap_never_exceeds_default`, `ema_smooths_spikes`) must pass unchanged.
- `cargo check -p qbz-audio` and `cargo check -p qbz` (or whichever crate
  hosts the CMAF streaming loop / prefetch dispatcher) to confirm
  `network_throttle::state()`, `record_segment_bandwidth`,
  `record_underrun`, `current_prefetch_cap`, `seconds_since_download`,
  `playback_mbps_for_quality` all still resolve.
- No manual smoke-test strictly required (this is pure computation +
  in-process state with full unit coverage), but if time permits, verify
  playback under a throttled/slow network condition still degrades
  prefetch concurrency as expected.
