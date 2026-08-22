# crates/qbz-theme/src/lib.rs (176 lines)

## Summary
Crate root for `qbz-theme`: module declarations/re-exports, the
`ThemeListEntry` struct, `default_theme_id()`, `is_light()`,
`is_high_contrast()`, `theme_list()`/`implemented_theme_list()` builders,
and a `#[cfg(test)] mod tests` block (6 tests) covering all of the above.

## Proposed split
This file is only marginally over budget (176 vs 130) and is the crate's
public API surface — keep it as the thin root, just carve the test module
out (the single biggest chunk, ~90 of the 176 lines) into its own file:

- `lib.rs` (~90 lines) — keep `pub mod`/`mod`/`pub use` declarations,
  `ThemeListEntry`, `default_theme_id()`, `is_light()`,
  `is_high_contrast()`, `theme_list()`, `implemented_theme_list()`. Add
  `#[cfg(test)] mod tests;` pointing at the new file (or, if the project
  convention is inline `#[path]`-free `mod tests;` resolving to
  `tests.rs` alongside `lib.rs`, use that).
- `tests.rs` (~90 lines) — the existing `#[cfg(test)] mod tests { use
  super::*; ... }` body, moved verbatim into its own file as
  `mod tests { use crate::*; ... }` (or keep `use super::*;` if declared
  as `#[path = "tests.rs"] mod tests;` inside `lib.rs` — either is fine,
  pick whichever the rest of the crate already does for test-only files).

## Re-export surface
No change needed — `lib.rs` itself remains the crate's public surface
(`pub use auto::..., color::..., colors::..., custom::..., id::...,
registry::palette`); every downstream crate (`qbz-ui`, `qbz-app`, the Tauri
build, TUI) keeps importing `qbz_theme::{ThemeId, palette, is_light, ...}`
exactly as before. Only the test module physically moves.

## Coupling / watch out
- This is a genuinely small, low-risk file — the only "coupling" is that
  the tests reference essentially every public symbol in the crate
  (`default_theme_id`, `palette`, `ALPHA_COUNT`, `is_light`, `ALL`,
  `implemented_theme_list`, `theme_list`, `is_high_contrast`), so the moved
  test file needs a fairly broad `use` (or `use super::*;` if kept as a
  child module via `#[path]`).
- Given this file is barely over the limit, it's reasonable to defer this
  split to very last / lowest priority relative to the other 9 files in
  this batch — mentioning it here for completeness per the task, but it's
  not where the 130-line rule is being meaningfully violated.

## Verify after split
- `cargo test -p qbz-theme` — all 6 existing tests
  (`default_is_oled`, `registry_returns_populated_struct_for_default`,
  `light_dark_flag_from_luminance`, `implemented_list_is_every_theme`,
  `light_dark_filter_is_luminance_correct`, `full_list_has_all_entries`,
  `high_contrast_flag_only_for_hc_themes`) must still pass unchanged.
- `cargo check -p qbz-theme` to confirm no downstream crate's `use
  qbz_theme::...` import broke.
