# crates/qbz-audio/src/alsa_backend.rs (1535 lines)

## Summary
ALSA direct-hardware audio backend: `/proc/asound`-based device enumeration
(no `aplay`/`alsa-utils` dependency), device-id parsing/normalization/fallback
helpers, PipeWire sink suspend/resume for exclusive mode, and the
`AudioBackend` trait impl (`AlsaBackend`) that creates CPAL/direct-ALSA output
streams including DoP and native-DSD paths.

## Proposed split
By domain — this is the single largest file in scope (11.8x budget), so it
splits into a `alsa_backend/` directory with a thin `mod.rs` re-export.

- `alsa_backend/mod.rs` (~40 lines) — module declarations + `pub use`
  re-exports so `super::alsa_backend::AlsaBackend`,
  `normalize_device_id_to_stable`, `resolve_stable_to_current_hw`,
  `device_supports_sample_rate`, `get_device_supported_rates`,
  `create_dop_stream`, `create_native_dsd_stream`, `resume_suspended_sink`
  all keep their current import paths.
- `alsa_backend/pipewire_suspend.rs` (~50 lines) — `SUSPENDED_SINK` static,
  `suspend_default_sink_for_exclusive`, `resume_suspended_sink` (issue #263
  fix). Self-contained, only touches `pactl` via `std::process::Command`.
- `alsa_backend/proc_asound.rs` (~200 lines) — all `/proc/asound` reading:
  `ProcCardInfo`, `ProcPcmInfo`, `read_proc_asound_cards`,
  `parse_proc_card_line`, `read_card_pcm_devices`, `build_card_info_map`,
  `find_card_number_by_name`, `get_hw_supported_rates`. Pure parsing + fs
  reads, easy to unit test in isolation.
- `alsa_backend/device_id.rs` (~180 lines) — device-id string manipulation:
  `is_known_pcm_id`, `build_hw_fallback_id`, `raw_open_ids`,
  `extract_card_name_from_device`, `is_card_present_in_proc`,
  `normalize_device_id_to_stable`, `resolve_stable_to_current_hw`,
  `device_supports_sample_rate`, `get_device_supported_rates`. Depends on
  `proc_asound` for card lookups but is otherwise pure string logic.
- `alsa_backend/sample_rates.rs` (~90 lines) — `COMMON_SAMPLE_RATES`,
  `find_best_fallback_rate`, `get_supported_sample_rates` (CPAL-facing rate
  probing, distinct from the `/proc`-based hardware rates in `proc_asound.rs`).
- `alsa_backend/enumerate.rs` (~200 lines) — `AlsaBackend::new`,
  `enumerate_with_proc_descriptions`, `build_cpal_device_map` (the
  `impl AlsaBackend` block's device-listing half).
- `alsa_backend/direct_stream.rs` (~260 lines) — `try_create_direct_stream`
  (the hw:/plughw: fallback dance with busy-retry backoff), plus
  `create_dop_stream` and `create_native_dsd_stream` (DSD plan phases 2-3),
  since all three share the same retry-backoff + PipeWire-suspend pattern.
- `alsa_backend/output_stream.rs` (~230 lines) — the `AudioBackend` trait impl
  for `AlsaBackend`: `backend_type`, `enumerate_devices` (delegates),
  `create_output_stream` (the CPAL `MixerDeviceSink` path with rate fallback),
  `is_available`, `description`, `as_any`.
- `alsa_backend/tests.rs` (~160 lines) — the existing `#[cfg(test)] mod tests`
  block, included via `#[cfg(test)] mod tests;` from `mod.rs`. Covers
  `build_hw_fallback_id`, `extract_card_name_from_device`,
  `is_card_present_in_proc`, `is_known_pcm_id`, `raw_open_ids`.

## Re-export surface
`alsa_backend/mod.rs` becomes the public-API surface: it must re-export every
currently-`pub` item at the same path (`pub use device_id::*;`,
`pub use direct_stream::create_dop_stream;`, etc.) plus define
`pub struct AlsaBackend` itself (likely declared in `mod.rs` with impl blocks
split across `enumerate.rs`/`direct_stream.rs`/`output_stream.rs` via
`impl AlsaBackend { ... }` in each file — Rust allows multiple `impl` blocks
for one struct across files in the same module).

## Coupling / watch out
- `AlsaBackend` struct itself must stay a single type; splitting its impl
  across 3 files (enumerate/direct_stream/output_stream) is fine in Rust but
  means the struct definition (`pub struct AlsaBackend { host: ... }`) needs
  a clear home — put it in `mod.rs` or `enumerate.rs` and import it in the
  other two.
- Heavy use of `super::AlsaDirectStream` and `super::backend::{...}` (from the
  parent `qbz-audio` module) — after the split, these become
  `super::super::AlsaDirectStream` / `crate::AlsaDirectStream` depending on
  nesting depth; audit every `super::` path when moving code into a
  subdirectory.
- `device_id.rs` and `proc_asound.rs` are mutually needed by `enumerate.rs`
  and `direct_stream.rs` — keep functions `pub(super)` or `pub(crate)`, not
  private, so cross-file calls within the new `alsa_backend/` module work.
- The PipeWire-suspend retry-backoff pattern (`[50,100,200,400,800]` ms) is
  duplicated 3x (hw attempt, plughw attempt, DoP, native-DSD) — tempting to
  extract a shared helper during the split, but that's a behavior-preserving
  refactor beyond scope; at minimum keep the constant array defined once if
  extracting.
- Tests reference `build_hw_fallback_id`, `extract_card_name_from_device`,
  `is_card_present_in_proc`, `is_known_pcm_id`, `raw_open_ids` directly via
  `super::*` — after the split these live in `device_id.rs`; the test module
  needs `use super::super::device_id::*;` or equivalent.

## Verify after split
- `cargo build -p qbz-audio`
- `cargo test -p qbz-audio alsa_backend` (all existing unit tests green,
  unchanged assertions)
- Grep for `alsa_backend::` and `AlsaBackend` usages across the workspace
  (player, settings, DAC wizard) to confirm no import path broke.
- Manual smoke test on Linux with a real ALSA device if available (device
  enumeration + exclusive-mode playback), since this file has zero
  higher-level test coverage for `create_output_stream`.
