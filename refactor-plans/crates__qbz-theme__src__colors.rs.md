# crates/qbz-theme/src/colors.rs (168 lines)

## 1. Summary

The fully-materialized per-theme color contract: the 24-tier alpha
ramp constants/helpers (`ALPHA_PERCENTS`, `ALPHA_COUNT`, `alpha_byte`,
`alpha_index`), the `ThemeColors` struct (every themeable color field,
grouped by family: surfaces/text/accent/danger/warning/success/borders/
focus/extras/alpha), its `alpha_pct` lookup method, the `alpha_ramp`
builder function, and a small test suite.

## 2. Proposed module split

Only 168 lines (~1.3x the limit) — a minimal 2-way split is enough to
clear 130 lines per file without over-fragmenting a cohesive, mostly
declarative data-contract file.

| New file | Owns | ~lines |
|---|---|---|
| `colors/mod.rs` | `ThemeColors` struct definition (all fields + grouping comments) + module doc | ~75 |
| `colors/alpha.rs` | `ALPHA_PERCENTS`, `ALPHA_COUNT`, `alpha_byte`, `alpha_index`, `ThemeColors::alpha_pct` (as an `impl ThemeColors` block in this file), `alpha_ramp` | ~65 |
| `colors/tests.rs` | The entire `#[cfg(test)] mod tests` block | ~30 |

## 3. Re-export / public API surface

`colors/mod.rs`:

```rust
mod alpha;
#[cfg(test)]
mod tests;

pub use alpha::{alpha_byte, alpha_index, alpha_ramp, ALPHA_COUNT, ALPHA_PERCENTS};

use crate::color::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    // ... all fields unchanged
}
```

Every caller doing `use qbz_theme::colors::{ThemeColors, alpha_ramp,
ALPHA_PERCENTS, ...};` (the theme registry that builds one `ThemeColors`
per theme row, and the Slint bridge that reads `ThemeColors` fields)
keeps working unchanged.

## 4. Tricky coupling/shared state to watch out for

- `ThemeColors::alpha_pct` (in `alpha.rs`) reads `self.alpha` (the field
  declared on the struct in `mod.rs`) and `ALPHA_PERCENTS`/`alpha_index`
  (also in `alpha.rs`) — since it's an `impl ThemeColors` block in a
  different file than the struct decl, this is exactly the
  split-impl-across-files pattern used elsewhere in this refactor; it
  compiles fine within the same crate, just confirm the `impl` block's
  method visibility (`pub fn alpha_pct`) is unchanged.
- The struct's field-order comment ("Field order groups by family... to
  match the Slint `ThemeColors` struct and the plan's A.3 token list")
  is important: the Rust field ORDER should probably stay unchanged
  since it may be positionally significant if anything relies on
  struct layout (unlikely in safe Rust with named fields, but the
  comment suggests it mirrors an external Slint struct definition for
  readability parity — preserve the grouping/order when moving fields
  into `mod.rs`, don't alphabetize or reorder them).
- `alpha_ramp(is_light: bool)` is a free function (not a `ThemeColors`
  method) that OTHER code (the theme registry, building each theme row)
  likely calls to seed `ThemeColors.alpha` — confirm via grep that it's
  called by name (`colors::alpha_ramp` or `crate::colors::alpha_ramp`)
  and stays reachable after moving to `alpha.rs` (it will, since it's
  re-exported from `mod.rs`).
- `Rgba` import (`use crate::color::Rgba;`) is needed both in `mod.rs`
  (struct fields) and `alpha.rs` (return types) — duplicate the import
  rather than trying to share.

## 5. What to verify after the real split

- `cargo build -p qbz-theme` and `cargo test -p qbz-theme colors::` —
  all 3 tests green (`alpha_byte_rounds`, `ramp_polarity`,
  `alpha_count_is_24`).
- Grep the workspace for `qbz_theme::colors::` usages (the theme
  registry crate/module that builds each theme's `ThemeColors`, and any
  Slint FFI bridge reading these fields) to confirm import paths still
  resolve.
- Smoke-test: run the app, switch between at least two themes (one dark,
  one light) and confirm all themed surfaces/hover states/accent colors
  render identically to before the split (the alpha-ramp polarity flip
  for light vs. dark themes is the one behaviorally interesting bit
  here, so specifically check hover overlays on a light theme).
