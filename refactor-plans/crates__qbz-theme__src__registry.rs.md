# crates/qbz-theme/src/registry.rs (1879 lines)

## Summary
`ThemeId -> ThemeColors` registry: the `palette()` dispatcher, the `StdSpec`
builder shared by "standard" themes, ~32 individual theme-materializing
`fn theme_name() -> ThemeColors` functions (Core/Dark/Light/Accessibility
groups mirroring `ThemeCategory` in `id.rs`), plus two `#[cfg(test)]` modules
(general completeness checks + contrast/APCA verification for the a11y rows).

## Proposed split
Turn into a `registry/` directory. Group boundaries mirror the
`ThemeCategory` groups already declared in `id.rs` (Core / Dark / Light /
Accessibility) exactly as the task brief requests, plus a shared builder
module and the tests split by concern:

- `registry/mod.rs` (~90 lines) — module doc (trimmed), `LEGACY_*` constants,
  the `palette()` dispatcher `match` (this is the one function that touches
  every theme id, so it stays central), and `pub use` re-exports of the
  group modules' private `fn`s via `use` (they can stay private to the
  submodules and only `palette()` needs to see them, so `mod.rs` just
  declares `mod core_themes; mod dark_themes; mod light_themes;
  mod a11y_themes; mod std_spec;` and calls straight into them).
- `registry/std_spec.rs` (~110 lines) — `StdSpec` struct, its `Default` impl,
  `impl StdSpec { const TINT_*; fn build(...) }`, `bg_is_light()`, and the
  `with_alpha()` helper. This is the pure "builder" shared by every
  non-accessibility theme.
- `registry/core_themes.rs` (~180 lines) — `dark()`, `oled()`,
  `tokyo_night()`, `light()`, `sepia()` (the P1 four + standard Light +
  Sepia — matches `ThemeCategory::Core`).
- `registry/dark_themes.rs` (~420 lines, still large — see note) —
  `warm()`, `nord()`, `dracula()`, `catppuccin_mocha/latte/frappe/macchiato()`,
  `breeze_dark()`, `adwaita_dark()`, `aurora()`, `ikari()`, `ayanami()`,
  `iscariot()`, `stratego()`, `rumi()`, `zoey()`, `mira()`, `frost()`,
  `langley()` (matches `ThemeCategory::Dark` — the "branded/community dark"
  block in `id.rs`). At ~420 lines this alone still exceeds 130; split
  further alphabetically into two files, e.g. `dark_themes_a.rs` (warm
  through breeze_dark/adwaita_dark) and `dark_themes_b.rs` (aurora through
  langley) — each fn is fully independent so any split point is safe.
- `registry/light_themes.rs` (~180 lines) — `alucard()`, `rose_pine_dawn()`,
  `breeze_light()`, `adwaita_light()`, `duotone_snow()`, `snow_storm()`,
  `kurosaki()` (matches `ThemeCategory::Light`).
- `registry/a11y_themes.rs` (~260 lines, still slightly over — split into
  `a11y_themes_light.rs` (`wcag_light()`, `high_contrast_light()`) and
  `a11y_themes_dark.rs` (`wcag_dark()`, `high_contrast()`, `colorblind()`)
  if needed) — the 5 REDESIGNED accessibility rows, materialized directly
  (no `StdSpec`) because they use solid, not alpha-tinted, status surfaces.
- `registry/tests_basic.rs` (~130 lines) — the first `#[cfg(test)] mod
  tests` block: `fully_populated` helper + population/legacy-value/alpha
  tests (lines ~1342-1445 in the original, i.e. up to
  `alpha_byte_helper_matches_with_alpha`).
- `registry/tests_a11y.rs` (~180 lines) — the second `#[cfg(test)] mod
  tests` block: the APCA/contrast-ratio helpers (`over`, `simulate_deutan`,
  `delta_e`) and the `wcag_*`/`high_contrast_*`/`colorblind_*` contrast
  assertions, plus `all_32_themes_fully_populated_no_zero_color`.

## Re-export surface
`registry/mod.rs` is the module Rust code already imports as
`crate::registry` (re-exported from `qbz-theme/src/lib.rs` as `pub use
registry::palette;`). Only `palette()` needs to be `pub(crate)` visible from
`mod.rs`; the individual `dark()`/`nord()`/etc. functions can stay private to
their own submodule as long as `mod.rs`'s `match` arms can call them (bring
each submodule's functions into scope with plain `use core_themes::*;` etc.,
or fully-qualify each match arm — either works, no external API changes
needed since `lib.rs` only ever calls `registry::palette`).

## Coupling / watch out
- `with_alpha()` (in `std_spec.rs` per this plan) is used by `StdSpec::build`
  AND directly by the a11y-adjacent test blocks — check for any accessory
  test-only reference before moving it.
- `bg_is_light()` is called by literally every dark/light theme fn via
  `s.build(bg_is_light(s.bg_primary))` — must be `pub(super)` or re-exported
  so `dark_themes.rs`/`light_themes.rs` can reach it from `std_spec.rs`.
- `LEGACY_SURFACE_HOVER` / `LEGACY_BORDER_SUBTLE` / `LEGACY_BORDER_MUTED` /
  `LEGACY_CARD_SHADOW` constants are used by `core_themes.rs` (dark/oled/
  tokyo_night) AND referenced in doc comments elsewhere — keep them in
  `mod.rs` and `pub(super) use` into each theme-group submodule.
- The two `#[cfg(test)] mod tests` blocks both do `use super::*;` — after
  the split each test file needs `use crate::registry::*;` (or the specific
  paths) instead, plus their own `use crate::colors::{alpha_byte, ALPHA_COUNT};
  use crate::id::ALL;`.
- `ThemeId` variant coverage: the `palette()` match is exhaustive over every
  `ThemeId` variant declared in `id.rs` — if `id.rs` ever gains a new theme,
  the compiler will only catch a missing arm in `mod.rs`'s `match`, so keep
  that match centralized there and not duplicated in submodules.

## Verify after split
- `cargo check -p qbz-theme` and `cargo test -p qbz-theme` — this crate is
  explicitly designed (per its own module doc) to compile fast standalone,
  so this is a cheap, high-confidence check.
- Re-run both test modules (`tests_basic`, `tests_a11y`) and confirm the
  `every_registered_theme_is_fully_populated` / `all_32_themes_fully_populated_no_zero_color`
  tests still iterate `ALL` (from `id.rs`) — a missed re-export could
  silently shrink test coverage rather than fail to compile.
- Smoke-test: any frontend that calls `qbz_theme::palette(id)` (the Slint
  theme switcher, the Tauri-parity contrast tests) should be unaffected
  since the public path (`qbz_theme::registry::palette` / `qbz_theme::palette`
  re-export in `lib.rs`) does not change.
