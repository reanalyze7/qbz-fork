# crates/qbzd/src/tui/screens/wizard.rs (1193 lines)

## Summary
The qbzd setup TUI's HiFi/DAC Wizard screen: six-step flow (Welcome →
Check → SelectDacs → Review → Test → Done). Owns transient step/UI state
(`WizardState`), per-step key handling, clipboard/save actions for the
Review step's generated config blocks, and all `draw_*` rendering. Heavy
frontend-agnostic logic already lives elsewhere (`tui::wizard_core`); this
file is UI-only. Has a substantial `#[cfg(test)]` module (lines 960-1193,
~230 lines) with step-transition and render-snapshot tests.

## Proposed split
- `wizard/mod.rs` (~20 lines) — module doc + `pub use` re-exports of
  `WStep`, `STEP_ORDER`, `next_step`, `prev_step`, `WizardState`.
- `wizard/step.rs` (~95 lines) — `WStep` enum, `STEP_ORDER`, `step_index`,
  `next_step`, `prev_step`, `impl WStep::title` (lines 40-91).
- `wizard/state.rs` (~230 lines) — `CheckField`, `Candidate` (+ `from_data`),
  `ConfigBlock`, `WizardState` struct + its non-key/non-draw methods:
  `new`, `is_editing`, `editing_label`, `claims_horizontal`, `help_text`,
  `set_health`, `sample_host`, `set_candidates`, `set_configs`,
  `set_test_result`, `checked_dacs`, `has_selection`, `Default impl`
  (lines 96-311, 898-903).
- `wizard/keys.rs` (~330 lines) — every `handle_key`/`keys_*`/
  `on_escape`/`advance`/`retreat`/`open_check_editor`/
  `handle_check_editor`/`handle_manual_input`/`follow_focus`/
  `review_content_lines`/`max_review_scroll`/`copy_focused_block`/
  `copy_all_blocks`/`write_focused_block` methods on `WizardState` (lines
  317-628), as a second `impl WizardState` block in this file.
- `wizard/draw.rs` (~370 lines) — `draw` + all `draw_welcome`/
  `draw_check`/`check_block`/`draw_select`/`draw_review`/`draw_test`/
  `draw_done` methods (lines 632-897), plus the free functions
  `sandbox_name`, `block_line_count`, `append_block_lines` (lines 904-970)
  which only the draw code calls.
- `wizard/tests.rs` (~235 lines) — the entire `#[cfg(test)] mod tests`
  block (lines 960-1193), using `super::*`.

## Re-export surface
`wizard/mod.rs` becomes the `mod wizard;` target for
`crate::tui::screens::wizard::{WStep, WizardState, ...}` — `pub use
step::*; pub use state::*;` keeps `WStep`/`STEP_ORDER`/`next_step`/
`prev_step`/`WizardState` reachable at their current paths. `keys.rs` and
`draw.rs` are just additional `impl WizardState` blocks (via `impl
super::state::WizardState` or `use super::state::WizardState;`), so no new
public surface is needed for them — Rust allows an impl block in any
sibling module as long as the type is imported.

## Coupling / watch out
- `WizardState`'s fields (many, e.g. `review_focus`, `configs`,
  `clip_env`, `status_flash`) must stay `pub(crate)` or accessible enough
  for `keys.rs` and `draw.rs`'s `impl WizardState` blocks to reach them —
  since they're additional `impl` blocks on the SAME type in the SAME
  crate, private fields declared in `state.rs` are still visible to
  siblings only if declared `pub(super)`/`pub(crate)` or the impls stay in
  a child module of `state`'s defining module. Simplest: keep `WizardState`
  struct's fields at their current visibility and put `keys.rs`/`draw.rs`
  as `mod keys; mod draw;` UNDER the same `wizard` module so field
  privacy (module-private) still permits cross-file access within
  `wizard::*`.
- `copy_all_blocks`/`write_focused_block` depend on `clipboard::` and
  `wizard_core::BACKUP_CMD` — keep those `use` imports in `keys.rs`.
- The render-snapshot tests call private `draw_*` helpers indirectly via
  `render_step`/`render_step_sized` (defined inside the test module) — as
  long as `tests.rs` is a child of `wizard` (not a separate crate-level
  test file), `super::*` still resolves everything.
- This file is explicitly flagged in its own header comment as an
  "owner-sanctioned cap break" (FB4) for the TUI's step count, not for
  line count — the split here is purely about the 130-line rule, unrelated
  to that sanctioned exception.

## Verify after split
- `cargo test -p qbzd tui::screens::wizard` (or equivalent) green,
  including the step-transition, selection-gate, and 80x24/120x30
  render-snapshot tests.
- `cargo build -p qbzd`.
