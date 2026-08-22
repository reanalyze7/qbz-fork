# crates/qbz-theme/src/color.rs (193 lines)

## Summary
Plain `Rgba` color type + hand-rolled contrast math: WCAG 2.x relative
luminance/contrast ratio, and an APCA (Lc) approximation used as a secondary
a11y test gate. No external color crate dependency (ADR-006). ~156 lines of
logic, ~37 lines of tests.

## Proposed split
- `mod.rs` (~40 lines) — `Rgba` struct + `rgb`/`rgba`/`from_hex`/`to_hex`.
- `wcag.rs` (~25 lines) — `srgb_to_linear`, `relative_luminance`,
  `contrast_ratio` (WCAG 2.x).
- `apca.rs` (~55 lines) — the APCA constants block + `apca_screen_y`,
  `apca_soft_clamp`, `apca_lc`.
- `tests.rs` (~37 lines) — existing `#[cfg(test)] mod tests`, moved as-is.

## Re-export surface
`mod.rs` re-exports `Rgba`, `relative_luminance`, `contrast_ratio`,
`apca_lc` — the functions consumed by `qbz-theme`'s registry/generator code
(`crate::color::{Rgba, contrast_ratio, apca_lc}` style imports) so no caller
changes needed.

## Coupling / watch-outs
- Low risk: every function here is pure with no shared mutable state.
- The APCA module's constants (`APCA_SRGB_R` etc.) are only used within
  `apca.rs` — no cross-file coupling.
- Keep the doc comments on `apca_lc`'s sign convention (negative = light
  text on dark bg) verbatim — it's non-obvious and load-bearing for anyone
  reading the split-out file cold.

## Verify after split
`cargo test -p qbz-theme color::`; grep `crate::color::` across
`qbz-theme/src` to confirm no import breaks.
