# crates/qbzd/src/tui/clipboard.rs (336 lines)

## Summary
SSH-first tiered clipboard for the TUI wizard's config blocks: a pure tier
planner (`plan_tiers`) and OSC-52 payload builder, plus IO-side tier
attempts (`wl-copy`/`xclip`/tty-write/file-fallback) that never hard-fail.

## Proposed split
Textbook pure/IO split — the module doc already calls out "Two pure,
unit-tested pieces" as the design intent.

- `clipboard/mod.rs` (~20 lines) — module doc (lines 1-9), `pub use`
  re-exports of `Tier`, `ClipEnv`, `plan_tiers`, `osc52_payload`,
  `osc52_fits`, `wizard_dir`, `write_wizard_file`, `CopyReport`, `copy`.
- `clipboard/tiers.rs` (~70 lines) — pure: `Tier` enum + `short_label`,
  `ClipEnv` struct + `from_env` (env sampling is a thin IO read but kept
  here since it's the natural companion of `ClipEnv`), `plan_tiers`.
- `clipboard/osc52.rs` (~45 lines) — pure: `base64`, `osc52_payload`,
  `OSC52_MAX_B64_LEN`, `osc52_fits`.
- `clipboard/files.rs` (~25 lines) — IO: `wizard_dir`, `write_wizard_file`.
- `clipboard/copy.rs` (~90 lines) — IO orchestration: `CopyReport`, `copy`,
  `try_tier`, `write_osc52`, `pipe_to`.
- `clipboard/tests.rs` (~80 lines) — the entire `#[cfg(test)] mod tests`
  block, declared via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`clipboard/mod.rs` re-exports every public item at `crate::tui::clipboard::*`
so the wizard's config-block rendering code (which calls
`clipboard::copy(...)`, `clipboard::ClipEnv::from_env()`, etc.) needs no
import-path changes.

## Coupling / watch out
- `copy()` (in `copy.rs`) calls `plan_tiers` (from `tiers.rs`) and `base64`
  (from `osc52.rs`, currently private `fn base64`) — `base64` needs to
  become `pub(crate)` or `pub(super)` in `osc52.rs` so `copy.rs` can reuse
  it for the oversized-payload precheck; it is NOT otherwise exported
  today (only `osc52_fits`/`osc52_payload` are `pub`), so tighten
  visibility rather than making it fully `pub`.
- `try_tier` (in `copy.rs`) matches on `Tier` variants and calls
  `write_wizard_file` (from `files.rs`) and `write_osc52`/`pipe_to` (same
  file) — straightforward same-file/cross-file `use` additions, no shared
  mutable state anywhere in this file (everything here is pure functions
  or one-shot process spawns).
- Tests directly exercise `base64` (private) via `base64_matches_known_vectors`
  — after the split this test either moves to `osc52.rs`'s own
  `#[cfg(test)]` block (simplest — keeps the pure fn and its unit tests
  together) instead of the shared `tests.rs`, or `tests.rs` needs
  `super::osc52::base64` back in scope with adjusted visibility. Recommend
  splitting tests by file: `osc52.rs` keeps `base64_*`/`osc52_payload_*`/
  `osc52_fits_*` tests inline, `tiers.rs` keeps `plan_tiers_*` tests
  inline — this avoids the `tests.rs` re-export problem entirely and
  matches the file's own "two pure pieces" framing better than one big
  tests file.

## Verify after split
- `cargo test -p qbzd tui::clipboard::` — all 7 tests green
  (`base64_matches_known_vectors`, `osc52_payload_wraps_base64_in_the_escape`,
  `osc52_payload_tmux_passthrough_doubles_esc`, `plan_tiers_is_ssh_first_remote`,
  `plan_tiers_prefers_native_tools_locally`,
  `osc52_fits_thresholds_at_100kb_post_base64`, `plan_tiers_always_ends_in_file`).
- `cargo check -p qbzd` and grep for `tui::clipboard::` importers in the
  wizard's TUI screens to confirm the public path is unchanged.
- No slint-viewer check needed (this is a pure TUI/CLI file, no Slint UI).
