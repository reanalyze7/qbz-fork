# crates/qbz-audio/src/pipewire_backend.rs (1053 lines)

## Summary
The PipeWire/PulseAudio `AudioBackend` implementation: device enumeration
(via `pw-dump` natively, falling back to `pactl`), DAC sample-rate
negotiation/forcing (clock.force-rate), CPAL stream creation with device
routing (locked-mode `PIPEWIRE_NODE` targeting), and availability probing.

## Proposed split
By responsibility — the file is the largest in this batch, so split into
several cohesive modules under a `pipewire/` directory, separating pure
parsing from the process-spawning IO and the big `create_output_stream`
orchestration:

- `pipewire/mod.rs` (~45 lines) — module doc, `CLOCK_FORCE_APPLIED` static,
  `PwNodeEnvGuard` (+ Drop impl), `PipeWireBackend` struct + `new()`,
  `reset_pipewire_clock`, re-exports, `impl AudioBackend for PipeWireBackend`
  delegating to the functions below (`backend_type`, `description`,
  `as_any` stay here — they're one-liners; `enumerate_devices`,
  `create_output_stream`, `is_available` delegate into the submodules).
- `pipewire/rates.rs` (~110 lines) — `get_alsa_card_for_sink`,
  `get_sink_supported_rates`, `get_pipewire_current_rate`,
  `find_best_fallback_rate` (the DAC rate-capability + fallback-family
  logic — mostly IO (proc/pw-metadata reads) with one pure helper
  (`find_best_fallback_rate`); keep them together since they're a tight
  "what rate should we use" cluster, or split `find_best_fallback_rate`
  into a `rates.rs` pure fn + the two proc/pw-metadata readers if a
  stricter pure/IO line is wanted).
- `pipewire/enumerate_pactl.rs` (~120 lines) — `enumerate_pipewire_sinks`
  (the pactl-based fallback enumerator + its line-by-line parser).
- `pipewire/enumerate_pwdump.rs` (~200 lines) — `enumerate_via_pw_dump` +
  the pure `parse_pw_dump_sinks` (the native pw-dump JSON parser) + its
  `#[cfg(test)] mod pw_dump_tests` (the two fixture tests) — this is
  already the file's own documented pure/IO seam ("Pure (no I/O) so it is
  unit-testable").
- `pipewire/probe.rs` (~50 lines) — `probe_command_ok` (the bounded
  availability-probe subprocess runner) + `is_available()`'s body.
- `pipewire/stream.rs` (~330 lines still over 130 — see below) —
  `create_output_stream`'s orchestration. This single function is ~380
  lines by itself and is the real challenge: it must be broken into
  sub-steps, e.g.:
  - `stream/sink_routing.rs` (~90 lines) — the `set-default-sink` /
    `skip_sink_switch` / effective-sink resolution block.
  - `stream/rate_forcing.rs` (~90 lines) — the pre-stream `clock.force-rate`
    apply + the post-stream re-apply/verify/retry block (two similar
    blocks — consider factoring a shared `force_and_verify_rate()` helper
    while splitting, since they're near-duplicates today).
  - `stream/device_select.rs` (~70 lines) — the CPAL host/device scoring
    loop (`best_device`/`best_score`) + `PIPEWIRE_NODE` guard setup.
  - `stream/build.rs` (~90 lines) — `StreamConfig`/`SupportedStreamConfig`
    construction, buffer-size selection, `DeviceSinkBuilder` call, and
    assembling the final `create_output_stream` function that calls the
    above four in sequence.

## Re-export surface
`pipewire/mod.rs` re-exports `PipeWireBackend` (the only public item other
crates use — check `qbz-audio/src/lib.rs` / wherever `AudioBackendType::
PipeWire` maps to this struct) so `qbz_audio::pipewire_backend::
PipeWireBackend` (or however it's currently path'd — confirm exact module
path in `qbz-audio/src/lib.rs`) is unaffected. `create_output_stream`,
`enumerate_devices`, `is_available` stay trait-method-only (not directly
called by name from outside), so their internal split is invisible to
callers as long as the `AudioBackend` impl block in `mod.rs` still wires
them up.

## Coupling / watch out
- **This is the highest-risk file in the batch.** `create_output_stream` is
  one long function with heavy internal sequencing (sink routing must
  happen before rate query, which must happen before rate forcing, which
  must happen before stream creation, which is followed by a RE-APPLY of
  the rate) — splitting it into helper functions is not just a lines-count
  exercise, it risks silently reordering side-effecting `Command::new(...)`
  calls if done carelessly. Recommend: extract each block into a private
  fn taking exactly the values it needs and returning what the next block
  needs (e.g. `resolve_effective_sink() -> Option<String>`,
  `negotiate_rate(sink, requested) -> u32`,
  `force_rate_if_unlocked(rate, skip_sink_switch)`, `select_cpal_device
  (host) -> Device`), keeping the ORIGINAL call order in the top-level
  `create_output_stream` that just chains these.
- `CLOCK_FORCE_APPLIED` (module-level `AtomicBool`) is shared mutable state
  between `create_output_stream` (sets it) and `reset_pipewire_clock` (reads
  + resets it) — both must stay reachable from wherever they end up; keep
  the static itself in `mod.rs` and have `stream/rate_forcing.rs` take a
  `&'static AtomicBool` or just reference `super::super::CLOCK_FORCE_APPLIED`
  directly (simplest: keep it `pub(crate)` in `mod.rs`).
- `PwNodeEnvGuard`'s `#[cfg(target_os = "linux")]` gating must travel with
  wherever the locked-mode routing block lands (`stream/device_select.rs`).
- The two rate-force blocks (pre-stream and post-stream re-apply/retry) are
  near-duplicate `pw-metadata` calls — worth a comment (not necessarily a
  fix, since this is a plan not a refactor) flagging them as a
  de-duplication opportunity for whoever does the real split.
- `#[cfg(test)] mod pw_dump_tests` at the bottom currently tests
  `parse_pw_dump_sinks` only — keep it colocated with that function
  (`enumerate_pwdump.rs`) rather than a separate top-level tests file.

## Verify after split
- `cargo test -p qbz-audio pw_dump` — the two `pw_dump_tests` fixture tests
  green.
- `cargo check -p qbz-audio` and `cargo check --workspace` (audio backend
  selection code elsewhere depends on `PipeWireBackend`).
- Manual smoke-test on a real Linux box with PipeWire: device list appears
  in Settings > Audio, selecting a device switches output, sample-rate
  switching (e.g. play a 96kHz then a 44.1kHz track) does not glitch, and
  stopping playback resets `clock.force-rate` (check via `pw-metadata -n
  settings 0 clock.force-rate` before/after).
- Given the risk noted above, this file's real split should land with an
  explicit manual audio test pass (device switch, rate switch, locked-mode
  routing with `skip_sink_switch`), not just `cargo test`, since most of
  the logic is subprocess/OS-interaction that unit tests can't cover.
