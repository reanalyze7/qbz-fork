# crates/qbz/src/theme.rs (412 lines)

## Summary
Bridges the frontend-agnostic `qbz-theme` registry to the Slint `Theme`
global: color conversion (`Rgba` → Slint `Color`), `ThemeColors` struct
mapping, `push_colors`/`apply_theme`, and a large set of dropdown-index /
slug helpers (including the Auto/Custom synthetic entries and the Dark/
Light filter variants).

## Proposed split
By concern — color conversion vs dropdown-index math (two nearly
independent halves):

- `theme/mod.rs` (~40 lines) — `to_color`, `push_colors`, `apply_theme`,
  `pub use` of `dropdown`.
- `theme/palette_map.rs` (~55 lines) — `to_slint` (the big `ThemeColors`
  field-by-field mapping, including the `alpha_N` ladder).
- `theme/dropdown.rs` (~120 lines) — `AUTO_SLUG`/`AUTO_LABEL`/
  `CUSTOM_SLUG`/`CUSTOM_LABEL` consts, `FILTER_*` consts, `auto_index`,
  `custom_index`, `is_auto_index`, `is_custom_index`,
  `selected_index_for_slug`, `dropdown_themes`, `filtered_dropdown_themes`.
- `theme/dropdown_labels.rs` (~70 lines) — `dropdown_labels`,
  `filtered_dropdown_labels`, `filtered_auto_index`, `filtered_custom_index`,
  `filtered_id_for_index`, `filtered_selected_index_for_slug`.
- `theme/id_lookup.rs` (~25 lines) — `id_for_slug`, `id_for_index`,
  `index_for_id`.
- `theme/tests.rs` (~110 lines) — existing `#[cfg(test)] mod tests` (9
  tests covering roundtrips, fallbacks, and the Auto/Custom/filter logic).

## Re-export surface
`theme/mod.rs` stays the `mod theme;` target. `push_colors` and
`apply_theme` are the two functions called from outside (`main.rs`/
`auto_theme.rs`/`custom_theme.rs`) — unaffected. Every dropdown/slug helper
(`dropdown_themes`, `dropdown_labels`, `filtered_*`, `id_for_slug`,
`id_for_index`, `index_for_id`, `selected_index_for_slug`, `AUTO_SLUG`,
`CUSTOM_SLUG`, etc.) re-exported via `pub use dropdown::*; pub use
dropdown_labels::*; pub use id_lookup::*;` so `crate::theme::X` paths for
the Settings > Appearance screen are unchanged.

## Coupling / watch out
- `to_slint` (palette_map.rs) is called only by `push_colors` (mod.rs) —
  needs `use super::palette_map::to_slint;` or keep `to_slint` `pub(super)`.
- The Auto/Custom synthetic entries are appended ONLY in the unfiltered/
  `FILTER_ALL` view (`filtered_dropdown_labels`'s explicit `if filter ==
  FILTER_ALL` gate) — this asymmetry (Dark/Light filtered lists never show
  Auto/Custom) is intentional and tested (`narrowed_lists_omit_auto_and_
  custom`); do not "fix" it as an inconsistency during the split.
- `dropdown_themes()` / `filtered_dropdown_themes()` recompute from
  `qbz_theme::implemented_theme_list()` on every call (no caching) — many
  helpers call these repeatedly (e.g. `auto_index()` calls
  `dropdown_themes().len()`); preserve this recompute-per-call pattern,
  don't introduce a shared cache as a side effect of the split.
- Tests reference helpers across both intended files (`dropdown.rs` and
  `dropdown_labels.rs`) — `theme/tests.rs`'s `use super::*;` covers all of
  them once `theme/mod.rs` re-exports everything.

## Verify after split
- `cargo test -p qbz theme::` — all 9 existing tests green.
- `cargo build -p qbz` and smoke-test Settings > Appearance: theme dropdown
  (all/dark/light filter), selecting a theme, Auto and Custom entries.
