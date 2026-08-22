# crates/qbz-theme/src/auto/system.rs (673 lines)

## Summary
Linux desktop-environment detection plus per-DE wallpaper path and system
accent-color / full-color-scheme probing (GNOME, KDE Plasma, COSMIC, XFCE,
Cinnamon), via `gsettings`/`xfconf-query`/config-file parsing — a 1:1 port of
the Tauri `auto_theme::system` module.

## Proposed split
By-domain (per-DE) plus a shared parsing-helpers module — this is native Rust
business logic with light "IO" (Command/fs) mixed with pure string parsing, so
split by DE-family first, then pull the pure parsers into their own module:

- `system/mod.rs` (~70 lines) — module doc, imports, `DesktopEnvironment` enum +
  `display_name()`, `detect_desktop_environment()` (1-73), plus the two
  dispatcher functions `get_system_wallpaper`/`get_wallpaper_for_de` and
  `get_system_accent_color`/`get_accent_for_de` (75-110) and
  `get_system_color_scheme` dispatcher (481-492) — these tie the per-DE modules
  together and are the natural "public API" file.
- `system/gnome.rs` (~100 lines) — `get_gnome_wallpaper`, `get_gnome_accent`
  (114-162), plus `get_gnome_color_scheme` (543-606) since it's GNOME-specific.
- `system/kde.rs` (~120 lines) — `get_kde_wallpaper`, `get_kde_accent`,
  `read_kde_color_key` (166-266), plus `get_kde_color_scheme` (494-541) — KDE is
  the biggest single DE (accent + full scheme both parse `kdeglobals`), keep
  together since they share `read_kde_color_key`.
- `system/cosmic.rs` (~55 lines) — `get_cosmic_wallpaper`, `get_cosmic_accent`,
  `extract_path_from_cosmic_config`, `parse_cosmic_color` (270-320, plus the
  cosmic-specific parsers at 428-477 — see note below on parser placement).
- `system/xfce_cinnamon.rs` (~65 lines) — `get_cinnamon_wallpaper` (324-342) and
  `get_xfce_wallpaper` (346-385); these two are small and DE-adjacent (both
  minor DEs with no accent-color support), fine to share a file.
- `system/parse.rs` (~90 lines) — the shared pure parsing helpers:
  `parse_gsettings_uri`, `parse_file_uri`, `parse_rgb_csv`, `is_image_path`
  (387-425, 608-617). These are used across GNOME/Cinnamon (gsettings URI) and
  KDE (RGB CSV) and Cosmic (image-path check) — centralizing avoids duplicate
  `use` chains and is the one truly "pure function" module here.
  Note: `extract_path_from_cosmic_config`/`parse_cosmic_color` are COSMIC-only
  callers of `parse_file_uri`/`is_image_path` — keep those two in `cosmic.rs`
  rather than `parse.rs` since they're not shared beyond COSMIC.
- `system/tests.rs` (~55 lines) — the `#[cfg(test)] mod tests` block (619-673):
  detect_de smoke test, gsettings-URI variants, file-URI, RGB-CSV, cosmic-color
  float parsing, image-path matching. Since tests reference functions across
  every submodule above, either keep as one file with `use super::parse::*`
  etc., or (simpler) split tests alongside their target module
  (`parse.rs` gets its own `#[cfg(test)]`, `cosmic.rs` gets its own) — the
  latter better matches the "one README per package, colocated tests" spirit,
  and each piece is tiny (~10-15 lines) so it won't push any file over 130.

## Re-export surface
`system/mod.rs` re-exports everything currently public:
`pub use gnome::*` is unnecessary since GNOME/KDE/etc. functions are
`fn` (private) except the two dispatchers and `DesktopEnvironment` — verify
which functions are actually `pub` vs private in the original (looking at the
read file: `detect_desktop_environment`, `get_system_wallpaper`,
`get_system_accent_color`, `get_system_color_scheme`, and `DesktopEnvironment`
are the only `pub` items; every `get_gnome_*`/`get_kde_*`/etc. is a private
`fn`). So `system/mod.rs` is the ONLY file that needs `pub fn`/`pub enum` —
the DE-specific files just need `pub(super)` or `pub(crate)` visibility on
their functions so `mod.rs`'s dispatchers can call them across submodule
boundaries.

## Coupling / watch out
- `read_kde_color_key` is used by BOTH `get_kde_accent` (fallback path) and
  `get_kde_color_scheme` (every single field) — must stay in `kde.rs`, not
  split further away from either caller.
- The `Accent` field in `get_kde_color_scheme` reuses `get_kde_accent`'s logic
  path (`accent_explicit.or_else(...)`) — when splitting, don't accidentally
  duplicate the "check [General] AccentColor first" logic; call the shared
  helper instead of re-parsing.
- `PaletteColor` and `SystemColorScheme` are imported via `super::{PaletteColor,
  SystemColorScheme}` (from the parent `auto` module) — every new submodule
  file that constructs these types needs its own `use super::super::{...}` (or
  re-export through `system/mod.rs`) since the relative path changes when
  functions move into a subdirectory.
- All the `Command::new(...)` calls (gsettings, xfconf-query) have zero
  automated test coverage (can't run real DE tooling in CI) — that's expected
  and pre-existing, not something the split should try to fix.

## Verify after split
- `cargo build -p qbz-theme` and `cargo build --workspace` (frontends depend on
  `qbz_theme::auto::system::*` for the "Sync with OS" appearance setting).
- `cargo test -p qbz-theme system` — all 6 unit tests green (they're pure-parser
  tests, no real desktop environment needed, so they run in CI unaffected).
- `grep -rn "auto::system::" crates/` to confirm no external crate reaches past
  `system/mod.rs`'s public surface (should show only
  `detect_desktop_environment`, `get_system_wallpaper`, `get_system_accent_color`,
  `get_system_color_scheme`, `DesktopEnvironment` in use).
