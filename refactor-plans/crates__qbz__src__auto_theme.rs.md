# crates/qbz/src/auto_theme.rs (149 lines)

## Summary
Wires the Settings "Auto (dynamic)" theme option to `qbz_theme::auto`
palette generation: builds an `AutoSource` from persisted prefs, seeds the
Settings UI state, applies the theme synchronously at startup (with a
default-theme fallback on failure), regenerates off-thread on demand, and
handles the native image-picker flow for the `image` source.

## Proposed split
Only 19 lines over budget — a light two-way split by "startup/seed" vs
"interactive regenerate/picker" is enough, no directory needed:

- `auto_theme.rs` (~75 lines) — becomes the re-export/entry surface: module
  doc, `source_from_prefs` (shared helper), `detected_de`, `seed_state`,
  `apply_startup` (the read-mostly, startup-path functions), plus `mod
  auto_theme_interactive;` declaration with `pub use
  auto_theme_interactive::{regenerate, select_image, set_source};`.
- `auto_theme_interactive.rs` (~80 lines) — `regenerate`, `select_image`,
  `set_source` (the three functions that spawn async work off the event
  loop and push results back via `upgrade_in_event_loop`).

## Re-export surface
`auto_theme.rs` stays the target of the existing `mod auto_theme;` in
`crates/qbz/src/lib.rs` (or `main.rs`). It must re-export `regenerate`,
`select_image`, `set_source` from the new sibling file so every existing
caller path (`crate::auto_theme::detected_de`,
`crate::auto_theme::seed_state`, `crate::auto_theme::apply_startup`,
`crate::auto_theme::regenerate`, `crate::auto_theme::select_image`,
`crate::auto_theme::set_source`) keeps resolving unchanged — these are
presumably called from the Settings screen's Slint-callback Rust glue and
from app startup.

## Coupling / watch out
- `source_from_prefs` is called by `apply_startup` (in `auto_theme.rs`) AND
  by `regenerate` (in `auto_theme_interactive.rs`) — keep it in
  `auto_theme.rs` and make it `pub(super)` or `pub(crate)` so the sibling
  file can call it via `use super::source_from_prefs;`.
- `crate::theme::push_colors` / `crate::theme::apply_theme` and
  `crate::toast::error` are called from both the startup path and the
  interactive path — no special handling, just re-`use` in each file.
- `crate::ui_prefs::{load, save, auto_theme_source_index,
  auto_theme_source_for_index}` are read/written from both files (seed_state
  reads, select_image/set_source write) — same store, no risk of
  duplication since these are free functions in a different crate module,
  not local state.
- `slint::Weak<AppWindow>` + `tokio::runtime::Handle` threading (weak
  upgrade-in-event-loop pattern) appears in every function in
  `auto_theme_interactive.rs` — keep the exact clone-before-move pattern
  (`weak_flag`/`weak.clone()`) noted in `regenerate`'s comment about why a
  plain `upgrade()` would fail from a tokio context; do not "simplify" this
  during the split.

## Verify after split
- `cargo build -p qbz` (check for any `#[cfg(test)]` — none observed in
  this read; re-check after the real split).
- Manually smoke-test in the running app: Settings → Appearance → Auto
  theme — switch source (System/Wallpaper/Image), pick a custom image, hit
  "Regenerate", and restart the app to confirm `apply_startup`'s fallback
  path still triggers correctly on a forced generation failure (e.g. an
  invalid persisted image path).
