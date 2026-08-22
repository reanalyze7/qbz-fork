# crates/qbz/src/selection.rs (133 lines)

## Summary
Shared keyboard-driven multi-select core (Excel-style additive Shift+Click
range) used by every selectable surface (album/artist/playlist/favorites/
label/local tracks/offline/mix/local albums). Only 3 lines over budget.

## Proposed split
Barely over — trim by moving the anchor bookkeeping out from the generic
span-fill helpers:

- `selection/mod.rs` (~75 lines) — the `SURFACE_*` constants, `Anchor`
  struct, `ANCHOR` thread_local, `set_anchor`, `clear_anchor`, `anchor_for`,
  `pub use` of `span`.
- `selection/span.rs` (~60 lines) — `apply_shift_range`, `select_all`,
  `resolve_anchor` (the generic `VecModel`-operating helpers).

## Re-export surface
`selection/mod.rs` stays the `mod selection;` target; the `SURFACE_*`
constants and `Anchor`-related fns stay there. `pub use span::*;` keeps
`apply_shift_range`, `select_all`, `resolve_anchor` at
`crate::selection::X` — every surface controller (mix.rs, label.rs, etc.)
calls these paths unchanged.

## Coupling / watch out
- Every consumer (`mix.rs::set_multi_select` etc.) calls
  `crate::selection::clear_anchor()` on enter/leave select-mode — a stable,
  small public surface; splitting internals doesn't change any call site.
- `resolve_anchor` depends on `anchor_for` (mod.rs) — needs
  `use super::anchor_for;` in `span.rs`.
- Given this file is barely over budget, consider whether adding a
  submodule for 3 lines is worth it vs. just trimming a comment — flagging
  both options for the reviewer; a mechanical split is still safe either
  way.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap
  given how many surfaces depend on this shared logic; a real split PR
  should add unit tests for `apply_shift_range`/`resolve_anchor`).
- Smoke-test Shift+Click range-select on at least two different surfaces
  (e.g. album tracks and a playlist) to confirm the additive-range and
  anchor-resolution-by-id behavior survives re-sort/filter.
