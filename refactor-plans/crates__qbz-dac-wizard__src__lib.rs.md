# crates/qbz-dac-wizard/src/lib.rs (795 lines)

## Summary
The whole HiFi Wizard (DAC setup) controller crate root: health-check /
remediation copy (Slice 6), DAC auto-detect + manual escape hatch (Slice 7),
self-service playback test / N6 read-back (Slice 9), and per-DAC config
generation + review (Slice 10) — all in one file, already informally divided
by the file's own `// ── Slice N: ... ───` banner comments.

## Proposed split
The file's own slice banners are the natural module boundaries — split
along them into one file per slice, keeping shared static state
(`LAST_HEALTH`, `TEST_TRACKS`) in whichever module owns it and having
other slices call through `pub` accessor fns (already the existing pattern
for `TEST_TRACKS` via `stash_test_tracks`/`test_tracks`):

- `lib.rs` (~15 lines) — crate doc comment (lines 1-8), `mod` declarations,
  and `pub use` re-exports of every public item from the slice submodules,
  so external callers keep using `qbz_dac_wizard::open_immediate(...)` etc.
  unqualified.
- `check.rs` (~230 lines, Slice 6, lines 9-297) — `LAST_HEALTH` static,
  `open_immediate`, `apply_health`, `set_distro`, `set_init`, `recompute`,
  `remediations`, `restart_cmd`, `pkg_pw_tools`, `pkg_pulse`, `install`,
  `install_reinstall`, `NIXOS_PIPEWIRE_BLOCK`, `reference_commands`,
  `full_stack_pkgs`. Still over budget — split further:
  - `check/state.rs` (~90 lines) — `LAST_HEALTH`, `open_immediate`,
    `apply_health`, `set_distro`, `set_init`, `recompute` (the
    UI-state-mutating half).
  - `check/remediation.rs` (~140 lines) — `remediations`, `restart_cmd`,
    `pkg_pw_tools`, `pkg_pulse`, `install`, `install_reinstall`,
    `NIXOS_PIPEWIRE_BLOCK`, `reference_commands`, `full_stack_pkgs` (the
    pure command-string generation, no Slint dependency at all — good
    candidate to eventually unit test independent of `AppWindow`).
- `select.rs` (~165 lines, Slice 7, lines 298-462, minus its own
  `#[cfg(test)] mod slice7_tests`) — `DacCandidateData`, `begin_detect`,
  `detect_blocking`, `apply_candidates`, `toggle_dac`, `validate_manual`,
  `validate_node_name`, `detect_dac_type`, `format_rates`, plus its tests.
  Roughly at budget; if trimming needed, move `format_rates` +
  `validate_node_name`/`detect_dac_type` (pure string helpers, ~40 lines)
  into a `select/pure.rs`.
- `test.rs` (~110 lines, Slice 9, lines 464-577) — `TestSeed`, `TEST_SEEDS`,
  `track_matches_seed`, `TEST_TRACKS` static, `stash_test_tracks`,
  `test_tracks`, `begin_test`, `end_test`, `queue_empty_notice`,
  `apply_poll`, `khz`.
- `review.rs` (~215 lines, Slice 10, lines 579-794, minus its own
  `#[cfg(test)] mod slice10_tests`) — over budget, split into:
  - `review/state.rs` (~90 lines) — `DacConfigData`, `checked_dacs`,
    `gen_configs_blocking`, `apply_configs`, `toggle_config`,
    `BACKUP_CMD`.
  - `review/conf.rs` (~130 lines) — `short_name`, `slugify`, `rates_list`,
    `pipewire_conf`, `pulse_conf`, `wireplumber_conf` (pure string
    generation, no Slint types at all) plus the `slice10_tests` module,
    since all its assertions are against `slugify`/`wireplumber_conf`.

## Re-export surface
`lib.rs` becomes the single `pub use check::*; pub use select::*; pub use
test::*; pub use review::*;` surface (with `check::state::*` /
`check::remediation::*` etc. re-exported transitively through `check/mod.rs`
if that slice is further split into a subdirectory). Every function currently
called as `qbz_dac_wizard::open_immediate(...)`,
`qbz_dac_wizard::apply_candidates(...)`, `qbz_dac_wizard::begin_test(...)`,
`qbz_dac_wizard::apply_configs(...)` etc. from the `qbz` Slint bin's wizard
wiring must resolve unchanged.

## Coupling / watch out
- `restart_cmd(init: InitSystem)` (defined in Slice 6 / `check/remediation.rs`)
  is called from BOTH `recompute` (check/state.rs) and `apply_configs`
  (review/state.rs, line 658) — cross-slice reuse of one pure fn. Keep it
  `pub(crate)` in `check::remediation` and `use crate::check::remediation::
  restart_cmd;` from `review::state`.
- `install` (check/remediation.rs) is called by both `remediations` (Slice 6)
  and `reference_commands` (Slice 6) — stays intra-module, fine.
- The two module-level `static Mutex<...>` items (`LAST_HEALTH`,
  `TEST_TRACKS`) are each read/written by multiple functions within their
  own slice only — no cross-slice static sharing, so splitting is safe as
  long as each stays `pub(crate)` (or private with the accessor fns staying
  `pub`) within its own module file.
- All functions take `&AppWindow` and reach `window.global::<DacWizardState>()`
  / other Slint globals (`DacCandidateRow`, `DacConfigRow`, `RemediationRow`)
  — these are generated Slint types imported from `qbz_ui`; every new file
  needs its own `use qbz_ui::{...}` for whichever rows it touches.
- `qbz_audio`, `qbz_i18n`, `qbz_models` are used across nearly every slice —
  each new file needs its own subset of `use` lines; don't try to
  consolidate them into one shared `use` re-export, just repeat as needed.

## Verify after split
- `cargo check -p qbz-dac-wizard` and `cargo test -p qbz-dac-wizard` (runs
  `slice7_tests` and `slice10_tests`, renamed/relocated but same assertions).
- `cargo build` for whichever Slint bin (`qbz`) wires up the wizard, to
  confirm the `qbz_dac_wizard::*` call sites still resolve.
- Smoke-test the actual HiFi Wizard flow in the running app: open wizard →
  check step renders remediations → select DACs → review generated configs
  → play test → confirms the N6 read-back label still populates — since
  this crate has real (if partial) test coverage but the Slint-glue paths
  (`open_immediate`, `apply_candidates`, `apply_configs`, `apply_poll`) are
  untested and easy to silently break with an import mistake.
