# crates/qbz-audio/src/alsa_direct.rs (972 lines)

## Summary
Direct (bypass-CPAL) ALSA hardware access for bit-perfect `hw:`/`plughw:`
playback: `AlsaDirectStream` construction (exclusive PCM open + D-Bus device
reservation) for standard PCM, DoP (DSD-over-PCM), and native DSD, plus its
several per-format `write*` methods, drain/stop lifecycle, hardware-volume
mixer control, and a non-Linux stub `impl` so the crate still compiles
off-Linux.

## Proposed split
Turn into an `alsa_direct/` directory:

- `alsa_direct/mod.rs` (~70 lines) — module doc, the `#[cfg(target_os =
  "linux")]` struct `AlsaDirectStream` definition (with its detailed
  field-order safety-comment kept verbatim — this comment is load-bearing
  documentation, do not trim it), the `#[cfg(not(target_os = "linux"))]`
  stub struct, `PIPEWIRE_VACATE_MARGIN` const, and `pub use` re-exports of
  everything below.
- `alsa_direct/recovery.rs` (~90 lines) — `log_pcm_recovery`,
  `recover_write_error`, `ensure_exact_rate` (the shared error-recovery /
  rate-verification helpers used by every constructor and every write path).
- `alsa_direct/open.rs` (~200 lines) — `AlsaDirectStream::new` (standard PCM
  open with format-priority negotiation): the reservation-acquire, PCM open,
  hwparams negotiation (access/format-priority-loop/channels/rate/buffer/
  period), and `prepare()`.
- `alsa_direct/open_dsd.rs` (~150 lines) — `AlsaDirectStream::new_dop` and
  `AlsaDirectStream::new_native_dsd` (the two DSD-plan-Phase-2/3 additive
  constructors; grouped together since they share the DSD context and are
  much smaller individually than the 130-line budget would need split
  further).
- `alsa_direct/write_pcm.rs` (~330 lines) — `write` and `write_f32` (the two
  large per-format match blocks: FloatLE/S32LE/S16LE/S243LE/S24LE branches
  each doing conversion + `io.writei` + recovery). This is the single
  biggest chunk and still exceeds 130 lines on its own — split further by
  the two functions:
  - `write_pcm/from_i16.rs` (~170 lines) — `write` (i16 source).
  - `write_pcm/from_f32.rs` (~170 lines) — `write_f32` (f32 source).
- `alsa_direct/write_dop.rs` (~40 lines) — `write_dop_i32`.
- `alsa_direct/lifecycle.rs` (~90 lines) — `drain`, `stop` (the bounded-drain
  polling loop and the drop+prepare stop ritual — both carry critical
  hardware-quirk comments, keep them attached to their functions verbatim).
- `alsa_direct/mixer.rs` (~55 lines) — `set_hardware_volume`.
- `alsa_direct/util.rs` (~15 lines) — `is_hw_device` (present on both the
  Linux and non-Linux `impl` blocks; keep both `#[cfg]` variants together
  here since it's the one method every platform needs, and it's the
  smallest surface).
- `alsa_direct/stub.rs` (~35 lines) — the entire `#[cfg(not(target_os =
  "linux"))] impl AlsaDirectStream` block (all methods return
  "Linux only" errors or defaults).

## Re-export surface
`alsa_direct/mod.rs` is the target of the existing `mod alsa_direct;` (likely
in `crates/qbz-audio/src/lib.rs`). Every `impl AlsaDirectStream` block across
the new files stays additive (multiple `impl` blocks for one struct is
legal Rust) — no `pub use` gymnastics needed beyond the struct itself
already being `pub` from `mod.rs`. External callers use
`qbz_audio::alsa_direct::AlsaDirectStream` exactly as before.

## Coupling / watch out
- The struct's field-order safety comment (PCM must drop before
  `_reservation`) is critical correctness documentation — it must stay on
  the struct definition in `mod.rs`, and every constructor (`open.rs`,
  `open_dsd.rs`) that builds `Self { .. }` must preserve `_reservation` as
  the LAST field in the literal, even split across files.
- `crate::DeviceReservation` and `crate::network_throttle::state()` are
  crate-level deps referenced from `open.rs`/`open_dsd.rs` and
  `recovery.rs` respectively — re-`use` in each file.
- The `#[cfg(target_os = "linux")]` gate must be repeated on every new file
  (or wrapped at the mod-declaration level in `mod.rs` via `#[cfg(...)] mod
  open;`) — do not let a non-Linux build accidentally try to compile ALSA
  types.
- `Arc<Mutex<PCM>>` is shared state read/written by every write/drain/stop
  method — no functional change needed, just make sure every file that
  locks `self.pcm` imports `Mutex`/`Arc` correctly (already re-exported via
  the struct field type, no need to re-`use` alsa's `PCM` type itself in
  files that only call `self.pcm.lock()`).
- `recover_write_error`/`log_pcm_recovery`/`ensure_exact_rate` are called
  from write_pcm, write_dop, and open/open_dsd — keep them `pub(super)` or
  `pub(crate)` in `recovery.rs` so all siblings can call them.

## Verify after split
- `cargo build -p qbz-audio` and `cargo build -p qbz-audio --target
  <non-linux-triple>` if cross-compilation is available (or at minimum
  grep for existing CI cross-checks) — this file is one of the few with a
  meaningfully different non-Linux code path, so both `#[cfg]` arms must
  compile.
- `cargo test -p qbz-audio` (check for any `#[cfg(test)]` — none observed
  in this read, but re-check after the real split in case a test module
  exists elsewhere referencing `alsa_direct::*` directly).
- Hardware smoke-test on a real Linux box with a `hw:`/`plughw:` DAC: open
  each of the three stream kinds (PCM/DoP/native-DSD), play audio through
  each `write*` conversion branch, and exercise natural-end drain + manual
  stop (the drain/stop logic has hardware-quirk-driven behavior that unit
  tests cannot cover).
