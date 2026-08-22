# crates/qbz-theme/src/custom.rs (351 lines)

## Summary
User-authored custom theme derivation: `CustomThemeBase` (12 hand-picked
hex tokens), hex-parsing plumbing, `base_from_theme` (reduce an existing
`ThemeColors` to base tokens), and `theme_from_base` (derive the full
`ThemeColors` contract from the base), plus its unit tests (~110 lines).

## Proposed split
By responsibility — the public data type vs the derivation math vs tests:

- `custom/mod.rs` (~75 lines) — module doc (lines 1-19), `CustomThemeBase`
  struct + its `default_oled()` impl (lines 32-88). This is the small
  public surface other code constructs/serializes.
- `custom/convert.rs` (~40 lines) — the hex↔color plumbing: `parse`,
  `to_pal`, `from_pal`, `CARD_SHADOW` const (lines 28-30, 63-79). Pure,
  reusable helpers.
- `custom/derive.rs` (~120 lines) — `base_from_theme` (lines 99-119) and
  `theme_from_base` (lines 121-239) — the core derivation logic and its
  big doc-comment derivation table. This is the file's real payload; keep
  the derivation-table doc comment attached to `theme_from_base`.
- `custom/tests.rs` (~110 lines, as `#[cfg(test)] mod tests` at the bottom
  of `custom/derive.rs` or its own file included via `#[path]`/`mod tests;`
  under `#[cfg(test)]`) — lines 241-351, unchanged content, `use
  super::*;` adjusted to reach both `mod.rs` and `derive.rs` items (may
  need `use crate::custom::*;` instead of a single `super::*` once split
  across files — check which items each test actually touches).

## Re-export surface
`custom/mod.rs` stays the public surface: `pub use convert::*;` (if any of
`parse`/`to_pal`/`from_pal` need to stay crate-visible — likely `pub(crate)`
only, check for other callers first) and `pub use derive::{base_from_theme,
theme_from_base};`. The crate's `lib.rs` line `pub mod custom;` needs no
change — `custom/mod.rs` resolves identically to the current `custom.rs`.
`CustomThemeBase` stays defined directly in `custom/mod.rs`, so
`qbz_theme::custom::CustomThemeBase` is unaffected.

## Coupling / watch out
- `theme_from_base` in `derive.rs` calls `parse`/`to_pal`/`from_pal` (would
  move to `convert.rs`) — needs `use super::convert::{parse, to_pal,
  from_pal};` or similar; these helpers are currently private `fn`s with no
  visibility modifier, so they need at least `pub(super)` to be reachable
  from a sibling file.
- `theme_from_base` also calls into `crate::auto::generator::{tint,
  pick_btn_text_for_accent_set, ensure_text_contrast_target}` and
  `crate::auto::PaletteColor` — these are external crate-level imports,
  unaffected by this file's internal split, but worth confirming
  `crate::auto::generator` isn't itself one of the ~396 oversized files
  being split by another agent in a way that would change these exact
  names/paths.
- `base_from_theme`'s `border` fallback logic (opaque `border_subtle` vs
  `border_strong`) is a one-off rule specific to reducing legacy P1 themes
  — keep its comment intact when moving to `derive.rs`, it explains a
  non-obvious asymmetry with `theme_from_base`'s forward direction.
- The `seed_derive_roundtrip_is_coherent` test asserts
  `base_from_theme(theme_from_base(base)) == base` — this couples
  `base_from_theme` and `theme_from_base` tightly; keep both in the same
  `derive.rs` file (already planned) so this round-trip invariant is
  visually obvious to a future editor of either function.

## Verify after split
- `cargo build -p qbz-theme`.
- `cargo test -p qbz-theme custom::` — all 8 existing unit tests must pass
  unchanged (default-seed OLED dark, base-token straight-through mapping,
  derived status-family tints, polarity-driven alpha/edges, seed/derive
  round-trip, determinism, accent-text legibility, malformed-hex fallback,
  JSON round-trip).
- `cargo clippy -p qbz-theme`.
- Smoke-test importers: `grep -rn "custom::" crates/qbz-slint crates/qbz`
  (or wherever the "Custom" theme editor UI lives) — confirm
  `CustomThemeBase::default_oled()`, `base_from_theme`, `theme_from_base`
  construction sites still compile.
