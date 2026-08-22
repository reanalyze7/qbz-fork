# crates/qbz-audio/src/health.rs (353 lines)

## Summary
HiFi Wizard "check" step diagnostics: cheap read-only shell probes for the
Linux audio-stack health (WirePlumber, pw-dump, CPAL/PipeWire ALSA bridge,
pactl, sink presence) plus Linux distro/init-system/sandbox detection so the
wizard can show the right `apt`/`dnf`/`pacman` remediation commands.

## Proposed split
Two largely independent concerns share the file: "is the audio stack
healthy" and "what distro/init/sandbox am I on" — clean domain boundary.

- `health/mod.rs` (~15 lines) — module declarations + re-exports of
  `AudioStackHealth`, `audio_stack_health`, `Distro`, `InitSystem`,
  `Sandbox`, `detect_sandbox`, `detect_init`, `detect_distro`.
- `health/audio_stack.rs` (~65 lines) — `AudioStackHealth` struct + its
  `is_ready` impl, `sh_ok` helper, `audio_stack_health()`. The
  WirePlumber/pw-dump/pactl/CPAL-sees-PipeWire probes.
- `health/distro.rs` (~90 lines) — `Distro` enum (+ `ALL`, `index`, `label`),
  `detect_distro()`, and the pure `parse_distro` classifier.
- `health/init_system.rs` (~90 lines) — `InitSystem` enum (+ `ALL`, `index`,
  `label`), `detect_init()`, and the pure `parse_init_from_comm` classifier.
- `health/sandbox.rs` (~20 lines) — `Sandbox` enum, `detect_sandbox()`. Small,
  but kept separate since both `detect_init` and `detect_distro` depend on it
  (avoids a circular "who owns Sandbox" question between distro.rs and
  init_system.rs).
- `health/tests.rs` (~65 lines) — the existing `#[cfg(test)] mod tests`
  block, included via `#[cfg(test)] mod tests;` from `mod.rs`. Covers
  `parse_distro` and `parse_init_from_comm` cases plus the `Distro::ALL`
  round-trip.

## Re-export surface
`health/mod.rs` is the public-API surface — re-export every currently-`pub`
item at the same path so `qbz_audio::health::AudioStackHealth` (etc.) is
unaffected by the file becoming a directory.

## Coupling / watch out
- `detect_init()` and `detect_distro()` both call `detect_sandbox()` first
  and short-circuit / branch on its result — `sandbox.rs`'s `Sandbox` enum
  and `detect_sandbox` fn must be `pub(super)` or `pub(crate)` (not private)
  so `init_system.rs` and `distro.rs` can import them.
- The doc comments explaining WHY sandbox detection matters (Flatpak/Snap
  host-path exposure) are split across `detect_init`'s and `detect_distro`'s
  doc comments in the current file — keep each explanation attached to its
  own function when moving, don't consolidate/lose the reasoning.
- `parse_distro`'s systemd-free-derivative-before-parent-family ordering
  (antiX before Debian, Artix before Arch) is load-bearing and tested —
  don't reorder the `if`/`else if` chain during the move.
- Tests reference `parse_distro` and `parse_init_from_comm` directly (not
  through the enums) — after the split these are `pub(super)` fns in
  `distro.rs`/`init_system.rs`; the test module needs
  `use super::super::distro::parse_distro;` (or re-export via `mod.rs`) —
  simplest is `use super::*;` if `mod.rs` re-exports the pure classifiers
  too (even though they're not part of the "real" public API, tests need
  them).

## Verify after split
- `cargo build -p qbz-audio`
- `cargo test -p qbz-audio health` (all existing distro/init classification
  tests green, unchanged assertions)
- Grep for `audio_stack_health`, `detect_distro`, `detect_init`,
  `detect_sandbox`, `Distro::`, `InitSystem::` call sites in the HiFi Wizard
  UI-glue code to confirm no import path broke.
