# `crates/qbz-theme/src/auto/generator.rs` (500 lines)

## 1. Summary
Assembles a full `ThemeColors` from either a k-means–extracted
image/wallpaper palette (`theme_from_palette`) or a desktop-environment
color scheme (`theme_from_scheme`), 1:1-ported from the Tauri
`auto_theme::generator` — shared derivation logic (`assemble`), WCAG
contrast-enforcement helpers, plus ~115 lines of tests.

## 2. Proposed module layout

Convert to `generator/` directory:

- `generator/mod.rs` (~20) — `mod` declarations + `pub use` re-exports of
  `theme_from_palette`, `theme_from_scheme` (the two entry points already
  re-exported from `auto/mod.rs`), and `pub(crate) use` re-exports of
  `tint`, `pick_btn_text_for_accent_set`, `ensure_text_contrast_target`
  (needed by `crate::custom` — see coupling note below). **This is the
  re-export/public-API surface.**
- `generator/assemble.rs` (~100) — `CARD_SHADOW` const, `opaque()`, `tint()`,
  and the shared `assemble()` function (the big parameter list building a
  `ThemeColors` from already-chosen solid colors).
- `generator/from_palette.rs` (~80) — `theme_from_palette()`.
- `generator/from_scheme.rs` (~115) — `theme_from_scheme()`. Marginally over
  130 is unlikely but if it lands high, split the "Surfaces"/"Text" local
  variable derivation into a private helper fn at the top of the same file
  rather than a new file (it's all one cohesive DE-scheme mapping).
- `generator/contrast.rs` (~65) — `pick_btn_text_for_accent_set`,
  `ensure_text_contrast`, `ensure_text_contrast_target`.
- `generator/tests.rs` (~115) — the whole `#[cfg(test)] mod tests` block
  (dark/light palette fixtures + the 5 tests). Small enough to stay as one
  file; split into `tests_palette.rs`/`tests_scheme.rs` only if it grows.

## 3. Re-export / public API surface
`generator/mod.rs` must replicate the exact set of items currently visible
at `crate::auto::generator::*`, because **`crates/qbz-theme/src/custom.rs`
imports directly from this path**:
```rust
use crate::auto::generator::{ensure_text_contrast_target, pick_btn_text_for_accent_set, tint};
```
`auto/mod.rs` also does `pub use generator::{theme_from_palette,
theme_from_scheme};` and calls `generator::theme_from_scheme(...)` /
`generator::theme_from_palette(...)` directly by path in several places.
Both of these must keep working with zero changes at the call site — so
`generator/mod.rs`'s re-exports need the SAME visibility (`pub(crate)` for
the three helpers, `pub` for the two entry points) as today.

## 4. Tricky coupling to watch
- This is the file's single biggest risk: **`tint`,
  `pick_btn_text_for_accent_set`, and `ensure_text_contrast_target` are
  `pub(crate)` and consumed by `custom.rs` outside this module.** If they
  move to `generator/assemble.rs` and `generator/contrast.rs` respectively,
  `generator/mod.rs` MUST re-export them (`pub(crate) use assemble::tint;`
  / `pub(crate) use contrast::{pick_btn_text_for_accent_set,
  ensure_text_contrast_target};`) so `custom.rs`'s existing `use
  crate::auto::generator::{...}` line keeps resolving without edits.
  Verify with `grep -rn "auto::generator::" crates/qbz-theme/src/` before
  and after.
- `assemble()` takes ~17 positional args (already `#[allow(clippy::
  too_many_arguments)]`) — don't be tempted to "clean this up" as part of
  the file-split; that's a separate refactor with its own risk, and the
  task here is purely mechanical extraction.
- Dark/light polarity logic (alpha ramp base, `success` hue, translucent
  edge base) is duplicated in spirit between `assemble()` and the two entry
  points — keep the "which polarity picks which literal color" logic in
  `from_palette.rs`/`from_scheme.rs` and only the assembly math in
  `assemble.rs`, matching the current file's own internal boundary.
- Test fixtures (`dark_palette()`, `light_palette()`) reference
  `ThemePalette` from `super::{PaletteColor, SystemColorScheme,
  ThemePalette}` — after the split these come from `generator::` (parent),
  so `tests.rs` needs `use super::*;` to still resolve, same as today.

## 5. What to verify after the real split
- `cargo test -p qbz-theme` — all 5 generator tests plus anything in
  `custom.rs`'s own test suite (which exercises the shared helpers) must
  stay green.
- `cargo build -p qbz-theme` and `cargo build --workspace` (qbz-ui and
  qbz-app both likely consume `qbz_theme::auto::{theme_from_palette,
  theme_from_scheme}` or the crate's public theme-building API) to confirm
  no downstream break.
- Explicitly re-check `custom.rs` compiles — it's the one file outside
  `auto/` that reaches into `generator`'s internals.
