# crates/qbz-audio/src/output_sinks.rs (185 lines)

## Summary
CPAL output-device enumeration shaped for the frontend (`OutputSinkInfo`:
name/description/volume/is_default), with a Linux-specific implementation
(extra diagnostic logging of supported configs) and a shared non-Linux
implementation.

## Proposed split
Only slightly over budget; split along the existing `#[cfg(target_os =
"linux")]` / `#[cfg(not(target_os = "linux"))]` boundary that's already in
the file, plus pull the always-shared pieces into their own module:

- `output_sinks/mod.rs` (~55 lines) — module doc, `OutputSinkInfo` struct,
  `cpal_device_name` helper (lines 1-44) — shared across both platform
  impls; re-exports `list_output_sinks` and `OutputSinkInfo`.
- `output_sinks/linux.rs` (~90 lines) — the `#[cfg(target_os = "linux")]`
  `list_output_sinks` (lines 55-140), with its extra per-device
  `supported_output_configs` diagnostic logging.
- `output_sinks/other.rs` (~45 lines) — the `#[cfg(not(target_os =
  "linux"))]` `list_output_sinks` (lines 142-185).
- `mod.rs` then does:
  ```
  #[cfg(target_os = "linux")]
  pub use linux::list_output_sinks;
  #[cfg(not(target_os = "linux"))]
  pub use other::list_output_sinks;
  ```
  so exactly one implementation is compiled in, matching today's behavior.

## Re-export surface
`output_sinks/mod.rs` re-exports `list_output_sinks` and `OutputSinkInfo` at
`crate::output_sinks::{list_output_sinks, OutputSinkInfo}` — the audio
settings UI / `AudioOutputBadges` component callers are unaffected.

## Coupling / watch out
- Both platform impls call the shared `cpal_device_name` helper and
  `crate::device_filter::retain_real_outputs` — keep `cpal_device_name` in
  `mod.rs` (already correctly the only non-cfg'd function) so both
  `linux.rs` and `other.rs` can `use super::cpal_device_name;`.
- Given the file is barely over 130 lines, an even lighter option is
  keeping it as ONE file but simply trimming the Linux diagnostic-logging
  block (lines 93-111) into a small private helper `fn log_configs(...)`
  in the same file — mention this as an alternative to the implementer if
  a full module split feels like overkill for ~55 lines of overage; the
  three-file split above is still the safer/more conventional choice given
  the project's per-file cap.

## Verify after split
- `cargo build -p qbz-audio` on both a Linux target and (if cross-compiling
  or CI covers it) a non-Linux target, or at minimum `cargo check --target
  x86_64-pc-windows-gnu -p qbz-audio` / similar to confirm the `other.rs`
  arm still compiles.
- Manual smoke test: Settings > Audio output device list still populates
  and correctly flags the current default.
