# crates/qbzd/src/tui/wizard_core.rs (629 lines)

## Summary
Frontend-agnostic HiFi-setup-wizard logic for the daemon's TUI (a copy of
the Slint `qbz-dac-wizard` controller logic, minus the Slint/`DacWizardState`
plumbing): DAC auto-detection, audio-stack-health remediation command
generation (distro/init-system aware), per-DAC PipeWire/Pulse/WirePlumber
config-snippet generation, and a curated playback self-test track/seed
lookup — plus one large `#[cfg(test)] mod tests` block.

## Proposed split
The file's own `// ── section ── ` banner comments already mark clean
domain boundaries — turn into a `wizard_core/` directory:

- `wizard_core/mod.rs` (~30 lines) — module doc/adaptation notes (trimmed),
  `use qbz_audio::{...};`, and `pub use` re-exports of every public item
  from the submodules below so `crate::tui::wizard_core::Foo` keeps working.
- `wizard_core/detect.rs` (~95 lines) — lines ~28-110: "select-dacs" section
  — `DacCandidateData`, `detect_blocking()`, `validate_node_name()`,
  `detect_dac_type()`, `format_rates()`.
- `wizard_core/remediate.rs` (~180 lines) — lines ~112-285: "check step"
  section — `remediations()`, `restart_cmd()`, `pkg_pw_tools()`,
  `pkg_pulse()`, `install()`, `install_reinstall()`, `NIXOS_PIPEWIRE_BLOCK`,
  `reference_commands()`, `full_stack_pkgs()`.
- `wizard_core/config_gen.rs` (~165 lines) — lines ~287-450:
  "review-and-apply" section — `DacConfigData` (+ its `short()` /
  `target_paths()` / `full_block()` methods), `gen_configs_blocking()`,
  `BACKUP_CMD`, `short_name()`, `slugify()`, `rates_list()`,
  `pipewire_conf()`, `pulse_conf()`, `wireplumber_conf()`.
- `wizard_core/test_seeds.rs` (~60 lines) — lines ~452-509: "self-service
  playback test" section — `TestSeed`, `TEST_SEEDS`, `track_matches_seed()`,
  `seed_for_rate_depth()`, `khz()`, `negotiated_label()`.
- `wizard_core/tests.rs` (~120 lines) — the existing `#[cfg(test)] mod
  tests` block moved verbatim (change `use super::*;` to `use
  crate::tui::wizard_core::*;` or keep as a child module of `mod.rs` via
  `#[path]` so `use super::*;` still works).

## Re-export surface
`wizard_core/mod.rs` becomes the target of the existing `mod wizard_core;`
in `crates/qbzd/src/tui/mod.rs` (or wherever it's declared). Every symbol
the sibling TUI screen module currently reaches as
`crate::tui::wizard_core::Foo` — `DacCandidateData`, `DacConfigData`,
`detect_blocking`, `validate_node_name`, `detect_dac_type`, `remediations`,
`restart_cmd`, `reference_commands`, `gen_configs_blocking`, `BACKUP_CMD`,
`TestSeed`, `TEST_SEEDS`, `track_matches_seed`, `seed_for_rate_depth`,
`khz`, `negotiated_label` — must stay reachable via `pub use detect::*; pub
use remediate::*; pub use config_gen::*; pub use test_seeds::*;` in
`mod.rs`.

## Coupling / watch out
- `DacConfigData::short()` (in `config_gen.rs`) calls the free fn
  `short_name()` (same file) which calls `slugify()` (same file) — keep
  these three together, they're a tight pure-string-processing cluster.
- `gen_configs_blocking()` calls `qbz_audio::query_dac_capabilities` AND
  `short_name`/`pipewire_conf`/`pulse_conf`/`wireplumber_conf` — all local
  to `config_gen.rs`, no cross-file coupling risk there.
- `seed_for_rate_depth()` constructs a throwaway `qbz_models::Track` to
  reuse `track_matches_seed()` — both stay in `test_seeds.rs` together,
  fine as-is.
- The doc comment at the top of the file explicitly documents 4 numbered
  "adaptations vs the original" — when splitting, keep this doc block
  (trimmed) in `mod.rs` since it explains WHY this file exists at all
  (a deliberate copy from `qbz-dac-wizard`, not a shared crate yet — there's
  a `TODO(converge: dac-wizard)` marker future work should notice).
- No shared mutable state (no `static`/`OnceLock`) in this file — it's all
  pure functions plus one blocking-I/O detector, so this split is low-risk
  compared to files with global caches.

## Verify after split
- `cargo test -p qbzd` — all 10 existing tests
  (`validates_node_names_like_tauri`, `detects_dac_type`,
  `formats_rates_khz`, `slugifies_descriptions`,
  `wireplumber_conf_pins_node_and_rates`,
  `full_block_and_paths_cover_the_three_files`,
  `seed_lookup_matches_known_reference_rates`,
  `remediations_nixos_collapses_to_one_config_block`,
  `remediations_debian_names_the_alsa_bridge_and_is_init_aware`,
  `reference_commands_used_in_sandbox_full_stack`,
  `negotiated_label_shows_rate_format_channels`) must pass unchanged.
- `cargo check -p qbzd` to confirm the sibling TUI wizard screen module
  (whatever calls into `wizard_core::*`) still compiles.
- Manual smoke-test of the daemon's setup wizard TUI flow (detect → check →
  review-and-apply → playback test) since this logic drives real
  filesystem-writing config generation.
