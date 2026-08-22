# crates/qbz-audio/src/coreaudio_direct.rs (471 lines)

## Summary
macOS-only CoreAudio direct-access layer: sample-rate probing/switching,
device enumeration/naming, Hog Mode (exclusive access) acquire/release with
hardware-volume save/restore, and a `CoreAudioExclusiveGuard` RAII wrapper;
plus non-macOS stub fns so cross-platform callers compile everywhere.

## Proposed split
Split by domain: sample-rate ops, device query/enum, hog-mode + volume, the
RAII guard, and the non-macOS stubs (each already delimited by `#[cfg(...)]`
and doc comments in the source).

- `coreaudio_direct/mod.rs` (~30 lines) — module doc, `#![cfg_attr(...allow(deprecated))]`,
  `pub mod` declarations, `pub use` re-exports so `qbz_audio::coreaudio_direct::X`
  is unchanged. Keep the `transport_types` mod and `COMMON_SAMPLE_RATES` const
  here too (small, shared-ish).
- `coreaudio_direct/sample_rate.rs` (~75 lines, `#[cfg(target_os = "macos")]`) —
  `query_supported_sample_rates`, `set_nominal_sample_rate`,
  `get_nominal_sample_rate`.
- `coreaudio_direct/devices.rs` (~75 lines, macOS-only) —
  `get_default_output_device`, `get_output_device_ids`, `get_device_name`,
  `find_device_by_name`, `resolve_output_device_id`,
  `resolve_output_device_name`, `get_device_transport_type` (this last one
  uses `transport_types`, keep that mod importable from here).
- `coreaudio_direct/hog_mode.rs` (~65 lines, macOS-only) —
  `get_hogging_pid`, `set_hog_mode`.
- `coreaudio_direct/volume.rs` (~105 lines, macOS-only) —
  `get_hardware_volume`, `set_hardware_volume` (the two loop-over-elements
  functions).
- `coreaudio_direct/guard.rs` (~75 lines, macOS-only) —
  `CoreAudioExclusiveGuard` struct + `impl` (`acquire`, `release`,
  `set_hardware_volume`) + `impl Drop`.
- `coreaudio_direct/stub.rs` (~25 lines, `#[cfg(not(target_os = "macos"))]`) —
  the four non-macOS stub fns + the stub `CoreAudioExclusiveGuard` unit
  struct.

## Re-export surface
`coreaudio_direct/mod.rs` re-exports every `pub fn`/`pub struct`/`pub type`
(`AudioDeviceID`, all `query_*`/`get_*`/`set_*`/`resolve_*` fns,
`CoreAudioExclusiveGuard`) at `crate::coreaudio_direct::*` (i.e.
`qbz_audio::coreaudio_direct::*`), matching today's flat single-file surface
so the audio backend selection code elsewhere in `qbz-audio` (and any
`qbz-app`/`qbz` caller) needs no import changes.

## Coupling / watch out
- The macOS/non-macOS split is ALREADY per-function `#[cfg(target_os = ...)]`
  attributes, not a file-level split — when moving functions into new files,
  each file stays either all-macOS or all-stub (as proposed above) rather
  than mixing `#[cfg]` blocks per function inside one file, so the split
  reads cleanly.
- `CoreAudioExclusiveGuard::acquire`/`release`/`Drop` depend on
  `get_hardware_volume`/`set_hardware_volume` (volume.rs) and `set_hog_mode`
  (hog_mode.rs) — cross-module calls via `super::volume::...` /
  `super::hog_mode::...` or re-exported at `mod.rs` and used via
  `crate::coreaudio_direct::...` from within the crate; either works, just
  be consistent.
- `transport_types` (FourCC constants) is used only by
  `get_device_transport_type` in devices.rs — could move into devices.rs
  directly instead of mod.rs if preferred; either placement is fine as long
  as it's not duplicated.
- No `#[cfg(test)]` block exists in this file — verification is compile-only
  (and only meaningful on macOS; non-macOS CI only exercises the stub file).

## Verify after split
- `cargo check -p qbz-audio --target x86_64-apple-darwin` (or on an actual
  macOS runner) to verify the macOS-gated code compiles; on this Linux
  workstation, `cargo check -p qbz-audio` will only compile the stub path
  (`coreaudio_direct/stub.rs`), which is a much weaker signal — flag for a
  macOS CI runner or a macOS-owning agent to double check the gated files.
- `cargo check -p qbz-audio` (Linux) to at least confirm the stub module and
  the `mod.rs` re-exports compile.
- Grep for `coreaudio_direct::` usages across `qbz-audio`/`qbz-app`/`qbz` to
  confirm the flat re-exported path still resolves after the split.
