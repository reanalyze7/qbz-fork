# crates/qbz/src/custom_theme.rs (276 lines)

## Summary
Custom-theme controller + persistence: reads/writes the 12-token
`CustomThemeBase` editor state on `AppearanceState`, derives a full palette
via `qbz_theme::theme_from_base` and live-pushes it, and persists the base to
`<data_dir>/qbz/custom_theme.json`.

## Proposed split
Already close to the limit (276 lines); split by the three natural
concerns — color conversion, persistence (I/O), and the Slint-state
wiring (the pure/IO/render principle applies cleanly here):

- `custom_theme/mod.rs` (~40 lines) — module doc, imports, re-exports of
  the public API (`load`, `save`, `exists`, `load_or_seed`, `seed_state`,
  `apply_startup`, `set_token`, `set_token_hex`, `toggle_dark`,
  `seed_from_current`).
- `custom_theme/color.rs` (~35 lines) — pure color conversion: `to_color`,
  `rgba_of`, `hex_to_color` (lines 21-30, 102-107) — the only
  no-Slint-window-needed pure functions in the file.
- `custom_theme/persistence.rs` (~60 lines) — pure I/O: `custom_theme_path`,
  `load`, `save`, `exists` (lines 109-157) — no `AppWindow` dependency, so
  cleanly separable from the UI-wiring half.
- `custom_theme/fields.rs` (~40 lines) — the token-key <-> struct-field
  mapping: `set_field`, `set_one_swatch` (lines 53-91) — the two parallel
  `match key { "surface-main" => ..., ... }` tables that must stay in sync
  with each other and with `push_base_to_state`.
- `custom_theme/state.rs` (~100 lines) — the `AppWindow`-touching wiring:
  `base_from_state`, `apply_live`, `push_base_to_state`, `seed_state`,
  `apply_startup`, `set_token`, `set_token_hex`, `toggle_dark`,
  `seed_from_current`, `load_or_seed` (lines 32-51, 93-101, 159-276) — the
  "render" half that reads/writes `AppearanceState`/`Theme` globals and
  calls `crate::theme::push_colors`.

## Re-export surface
`custom_theme/mod.rs` is what `crate::custom_theme::{...}` callers use today
(e.g. wherever Settings > Appearance wires the Custom theme option) — keep
every current `pub fn` name reachable at the same `crate::custom_theme::foo`
path via `pub use`.

## Coupling / watch out
- `set_field` (fields.rs) and `set_one_swatch` (fields.rs) and
  `push_base_to_state` (state.rs) all enumerate the SAME 11 token keys in
  the same order — if split across files, a future new token must be added
  in three places across two files instead of one; call this out
  explicitly in a comment at the top of `fields.rs` pointing to
  `push_base_to_state` in `state.rs` as the third place to update.
- `apply_live` (state.rs) calls `save` (persistence.rs) and
  `crate::theme::push_colors` — cross-module but one-directional, fine.
- `hex_to_color` (color.rs) is used by `push_base_to_state` (state.rs) —
  cross-module call, import explicitly.
- All the `AppWindow`/`AppearanceState`/`SlintTheme` Slint-generated types
  are only used in `state.rs`; keep `color.rs` and `persistence.rs` free of
  any `AppWindow` parameter so they stay easily unit-testable in isolation
  (currently the whole file has no `#[cfg(test)]` block at all — this split
  is also what would make `persistence.rs`/`color.rs` testable going
  forward, since they no longer need a live Slint window).

## Verify after split
- `cargo build -p qbz` (this crate's main binary/lib).
- Smoke-test in the running app: open Settings > Appearance > Custom theme,
  edit a swatch color (live re-derivation), toggle dark/light polarity,
  "Start from current theme", restart the app to confirm persistence
  round-trips via `custom_theme.json`.
