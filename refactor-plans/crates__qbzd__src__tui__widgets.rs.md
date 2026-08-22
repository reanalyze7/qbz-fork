# crates/qbzd/src/tui/widgets.rs (1008 lines)

## Summary
Reusable ratatui primitives shared by every setup-TUI screen: pure layout helpers
(sidebar width, word wrap, control-column math, follow-focus scroll math), the
titled-section-box renderer (with a scrolling variant), the bottom help bar, a
centered modal/panel/spinner overlay, plus two interactive sub-widgets (`SelectPopup`
filterable list and `TextInput` line editor). Roughly 40% of the file is its
`#[cfg(test)]` module.

## Proposed split
By responsibility (pure math vs render vs interactive widget vs tests) — this
matches the pure/IO/render principle closely since almost everything here is
either pure computation or ratatui rendering (no real I/O).

- `tui/widgets/mod.rs` (~15 lines) — module doc, `pub use` re-exports of everything
  below so `super::widgets::X` paths used by every TUI screen are unchanged.
- `tui/widgets/layout.rs` (~90 lines) — pure layout math: `sidebar_width`,
  `sidebar_is_wide`, `wrap`, `control_column`, `centered_rect`, `follow_scroll`,
  `truncate`, `pad_to`.
- `tui/widgets/field.rs` (~115 lines) — `Field` struct, `field_block`, `toggle_tone`,
  `focus_style`, `mask` (the field-row rendering concern).
- `tui/widgets/section.rs` (~130 lines) — `Section` struct, `sections`, `FocusAnchor`,
  `push_section`, `sections_height`, `sections_scroll` (the section-box stack +
  follow-focus scroll rendering).
- `tui/widgets/lines.rs` (~60 lines) — small line/span helpers: `action_line`, `blank`,
  `note_line`, `warn_line`, `err_line`, `wrapped_note`.
- `tui/widgets/help.rs` (~40 lines) — `is_key_glyph`, `help_spans`, `help_bar`.
- `tui/widgets/overlay.rs` (~90 lines) — `modal`, `titled_block`, `panel`,
  `spinner_frame`, `busy_overlay`.
- `tui/widgets/select_popup.rs` (~130 lines) — `SelectPopup`, `SelectOutcome` (the
  filterable select popup widget + its `draw`).
- `tui/widgets/text_input.rs` (~40 lines) — `TextInput`, `InputOutcome`.
- `tui/widgets/tests.rs` (~190 lines) — the entire `#[cfg(test)] mod tests` block,
  included as `#[cfg(test)] mod tests;` from `mod.rs`, referencing all the above via
  `use super::*;`. Given the size (~190 lines), consider a further split into
  `tests/layout_tests.rs`, `tests/section_tests.rs`, `tests/field_tests.rs` if it
  stays over budget — but as one `#[cfg(test)]` file it's lower priority since test
  files are more tolerant of the 130-line guideline per the README's spirit (still,
  try to keep the rule).

## Re-export surface
`tui/widgets/mod.rs` re-exports every public item (`Field`, `Section`, `FocusAnchor`,
`SelectPopup`, `SelectOutcome`, `TextInput`, `InputOutcome`, and all free functions) so
every existing `use super::widgets::{...}` / `use crate::tui::widgets::{...}` call
site across the TUI screens compiles unchanged.

## Coupling / watch out
- `theme` and `strings` sibling modules are imported via `super::theme` / `super::strings`
  — each new submodule needs its own `use super::super::{theme, strings};` (one level
  deeper now that widgets became a directory).
- `field_block` calls `focus_style` (from `field.rs` itself) and `theme::dim()` /
  `theme::ok()` — keep `toggle_tone` colocated with `field_block` since it's a private
  helper only it uses.
- `sections_scroll` depends on `sections` (fallback when content fits) and
  `sections_height` — keep both in `section.rs` together; also depends on
  `follow_scroll` from `layout.rs` — cross-module import needed.
- `modal`/`panel`/`busy_overlay` all call the private `titled_block` — keep it in
  `overlay.rs`, not promoted to `mod.rs`.
- `SelectPopup::draw` calls `centered_rect` (layout.rs), `focus_style` (field.rs),
  `note_line` (lines.rs), `theme::accent_bold()`, and `help_spans`/`Line` from
  help.rs — this is the most cross-cutting widget; expect several `use` lines.
- The `#[cfg(test)]` block at the bottom directly tests `sections_scroll` using
  `ratatui::backend::TestBackend` — when splitting tests out, this specific test
  needs `use super::section::*;` (or whatever the final module layout is) plus the
  `ratatui::backend`/`Terminal` imports.

## Verify after split
- `cargo test -p qbzd tui::widgets` — all existing tests (sidebar_width, wrap,
  control_column, follow_scroll, sections_scroll, field_block — ~13 tests) must stay
  green.
- `cargo check -p qbzd` to confirm every TUI screen file that does
  `use super::widgets::...` (or similar) still resolves.
- Manual smoke-test: run the setup TUI, confirm section boxes, help bar, modals,
  select popups and text inputs render/behave identically (focus highlighting,
  filter typing, scroll indicators on overflow).
