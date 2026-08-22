# crates/qbz-audio/src/backend.rs (932 lines)

## Summary
Audio backend abstraction: cross-backend enums/config types
(`AudioBackendType`, `AlsaPlugin`, `AudioDevice`, `BackendConfig`,
`AlsaDirectError`, `BitPerfectMode`), the `AudioBackend` trait, the
`BackendManager` factory (detection + construction), and the full
`CpalDefaultBackend` implementation including a large macOS-only exclusive-
mode / sample-rate-matching subsystem.

## Proposed split
By responsibility — types vs factory vs the one concrete backend
implementation (which is itself split further by platform concern), mirroring
the sibling `pipewire_backend.rs`/`alsa_backend.rs`/`pulse_backend.rs` files
already in this crate (so `backend/` becomes this crate's first
directory-backed module, same convention as `qbz-theme/src/auto/`).

- `backend/mod.rs` (~30 lines) — module doc + `pub use` re-exports of every
  public item below, so `use qbz_audio::backend::X` (used by `qbz-player` and
  `qbz` per the crate's public API) is unaffected.
- `backend/types.rs` (~130 lines) — `AudioBackendType` + its `Default` impl
  (16-59), `AlsaPlugin` + `Default` (62-87), `AudioDevice` (90-116),
  `BackendConfig` (118-145), `BackendResult` (148). Pure data, no logic.
- `backend/error.rs` (~90 lines) — `AlsaDirectError` enum, its `Display` impl,
  `allows_plughw_fallback`, `from_alsa_error` (150-222), and `BitPerfectMode`
  (224-233). Pure classification logic, no I/O.
- `backend/trait_def.rs` (~40 lines) — the `AudioBackend` trait itself
  (235-270). Kept separate from types so it reads as the contract, not mixed
  with data shapes.
- `backend/manager.rs` (~150 lines) — `BackendManager`, `available_backends`,
  `create_backend`, `is_pipewire_available`, `is_pulse_available`
  (272-421). The factory/detection layer — shells out to `pw-cli`/`pactl`,
  reads `XDG_RUNTIME_DIR` — genuinely I/O-flavored, cleanly separable.
- `backend/cpal_default.rs` (~230 lines) — `CpalDefaultBackend` struct +
  `new()` + the cross-platform `AudioBackend` impl (428-645): `backend_type`,
  `enumerate_devices`, `create_output_stream` (the non-macOS-exclusive path),
  `is_available`, `description`, `as_any`.
- `backend/cpal_macos.rs` (~290 lines) — the entire `#[cfg(target_os =
  "macos")] impl CpalDefaultBackend` block (647-932):
  `create_output_stream_with_exclusive_guard`,
  `open_macos_shared_stream_with_retry`, `current_macos_nominal_rate`,
  `shared_mode_nominal_stream_config`, `probe_macos_device`,
  `switch_sample_rate_if_needed`, `switch_default_device_rate_if_needed`.
  Compiles to nothing on Linux/Windows, so isolating it means Linux/Windows
  contributors never need to scroll past 300 lines of CoreAudio code. Still
  ~290 lines on macOS itself — split further into `cpal_macos/exclusive.rs`
  (guard + retry-open) vs `cpal_macos/rate_switch.rs` (the three rate-query/
  switch helpers) if it must hit 130 even under `#[cfg(target_os =
  "macos")]`.

## Re-export surface
`backend/mod.rs` stays the public surface: `pub use types::*; pub use
error::*; pub use trait_def::AudioBackend; pub use manager::BackendManager;
pub use cpal_default::CpalDefaultBackend;`. The crate's `lib.rs` line `pub
mod backend;` needs no change — `backend/mod.rs` resolves identically to the
current `backend.rs`.

## Coupling / watch out
- `CpalDefaultBackend`'s macOS `impl` block (in `cpal_macos.rs`) and its
  cross-platform `impl` block (in `cpal_default.rs`) are BOTH `impl
  CpalDefaultBackend { ... }` — legal as multiple inherent-impl blocks across
  files in the same crate, but every method called from one file into the
  other (e.g. `create_output_stream` calling `switch_sample_rate_if_needed`)
  must resolve via plain `Self::method()` — this works automatically once
  both files are `mod`-included under `backend/`, no explicit visibility
  changes needed since they're all crate-internal on the same type.
- `create_output_stream_with_exclusive_guard`'s default trait-level body
  (`trait_def.rs`) delegates to `create_output_stream` (default impl calling
  `self.create_output_stream(config).map(|sink| (sink, None))`) — the macOS
  override in `cpal_macos.rs` is a DIFFERENT (inherent, not trait-default)
  path; make sure `AudioBackend for CpalDefaultBackend`'s trait impl in
  `cpal_default.rs` does NOT redeclare `create_output_stream_with_exclusive_
  guard` — the macOS override must live on the trait impl block, not as a
  separate inherent method, or the macOS behavior silently stops being used
  through the `dyn AudioBackend` trait object. Read the original file's
  `#[cfg(target_os = "macos")] impl AudioBackend for CpalDefaultBackend`
  block carefully (line 600) — it's a SEPARATE trait-impl block from the
  main one at line 440, both need to move together or the macOS build breaks.
- `MACOS_SHARED_OPEN_MAX_ATTEMPTS`/`MACOS_SHARED_OPEN_RETRY_DELAY` constants
  (647-650) are macOS-only; keep them in `cpal_macos.rs` next to their only
  use site.
- `BackendManager::create_backend` references `crate::pipewire_backend::
  PipeWireBackend`, `crate::alsa_backend::AlsaBackend`,
  `crate::pulse_backend::PulseBackend`, `crate::alsa_error_handler::
  install_once`, `crate::coreaudio_direct::*`, `crate::device_filter::
  retain_real_outputs` — all absolute `crate::` paths, unaffected by the
  split, but confirm none of these sibling modules import FROM
  `backend::AudioBackendType` etc. in a way that would create a cycle once
  `backend` becomes a directory (it won't — same crate, same resolution).

## Verify after split
- `cargo build -p qbz-audio --all-features` on Linux (the CI platform) AND,
  if available, a macOS cross-check — since the macOS-only code (`cpal_macos.rs`)
  cannot be exercised in a Linux-only CI run, at minimum verify it still
  parses via `cargo check --target x86_64-apple-darwin -p qbz-audio` if a
  target/toolchain is installed, or flag for manual macOS verification.
- `cargo test -p qbz-audio`.
- `cargo clippy -p qbz-audio --all-features`.
- Smoke-test importers: `grep -rn "qbz_audio::backend::" crates/qbz-player
  crates/qbz` still compiles — specifically `BackendManager::
  available_backends()`/`create_backend()`, `AudioBackendType`,
  `BackendConfig`, `AudioDevice` construction sites.
