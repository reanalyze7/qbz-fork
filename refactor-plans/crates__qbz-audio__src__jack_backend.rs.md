# crates/qbz-audio/src/jack_backend.rs (181 lines)

## Summary
Native JACK output backend: registers `qbz` as a JACK client with stable
stereo output ports, auto-connects them to physical playback, and bridges a
lock-free SPSC f32 ring buffer between the player's feeder thread
(`JackStream::write_f32`) and JACK's realtime `process` callback
(`JackProcess`).

## Proposed split
Only modestly over budget (181 lines) — a two-way split by "RT callback" vs.
"client lifecycle/API" is enough, no need for a deep directory.

- `jack_backend/mod.rs` (~35 lines) — module doc, consts
  (`RING_CAPACITY_FRAMES`, `MAX_NFRAMES`), `pub use` re-export of
  `JackStream` (and `JackProcess` if anything outside the module needs the
  type name, otherwise keep it private to `process.rs`).
- `jack_backend/process.rs` (~35 lines) — `JackProcess` struct +
  `impl jack::ProcessHandler for JackProcess` (the realtime audio callback —
  isolated because it's the most safety-sensitive part: allocation-free,
  lock-free, must stay auditable on its own).
- `jack_backend/stream.rs` (~115 lines) — `JackStream` struct + `impl`:
  `new` (client open, port registration, ring setup, activation,
  auto-connect — the biggest chunk), `sample_rate`, `channels`, `write_f32`,
  `underruns`.

## Re-export surface
`jack_backend/mod.rs` re-exports `JackStream` at `crate::jack_backend::*`
(i.e. `qbz_audio::jack_backend::JackStream`) — the audio-backend selection
code elsewhere in `qbz-audio` (wherever backends are chosen: ALSA direct,
CoreAudio, JACK, etc.) constructs `JackStream::new(channels)` and calls
`write_f32`/`sample_rate`/`channels`; that call site is unaffected.

## Coupling / watch out
- `JackStream::new` constructs the `JackProcess` inline and passes it to
  `client.activate_async((), process)` — if `JackProcess` moves to
  `process.rs`, it needs to be `pub(crate)` or re-exported so `stream.rs` can
  construct it; keep both files in the same module so `pub(super)` suffices.
- The ring buffer split (`HeapRb::split()` -> `producer`/`consumer`) is the
  ONLY hookup between `stream.rs` (owns `producer`) and `process.rs` (owns
  `consumer`) — the two halves must be created together in `JackStream::new`
  and never re-split; this stays true regardless of file layout.
- `underruns: Arc<AtomicU64>` is shared (cloned) between `JackStream` and
  `JackProcess` — both files need `use std::sync::Arc` and
  `use std::sync::atomic::{AtomicU64, Ordering}`.
- `#[allow(dead_code)]` on `underruns()` — likely used only by future
  diagnostics/tests; preserve the attribute so it doesn't start warning.

## Verify after split
- No `#[cfg(test)]` block exists in this file — verification is compile +
  manual only (JACK requires a running JACK/pipewire-jack server to actually
  test the RT path).
- `cargo check -p qbz-audio` (this crate likely gates JACK behind a feature
  flag — check `Cargo.toml` for a `jack` feature and check with
  `--features jack` or similar if present).
- Manual/smoke: with a JACK or pipewire-jack server running, select the JACK
  output backend in QBZ, confirm `qbz:out_FL`/`qbz:out_FR` appear in
  qjackctl/qpwgraph, get auto-connected to physical playback, and audio
  plays without underrun spam in the logs.
