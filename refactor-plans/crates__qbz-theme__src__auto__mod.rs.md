# `crates/qbz-theme/src/auto/mod.rs` (287 lines)

Auto-theme entry point: `AutoSource` enum + `generate()` dispatcher (system/wallpaper/image
cascade), plus the `PaletteColor` color-math type and `ThemePalette`/`SystemColorScheme`
data structs. Submodules `generator`, `palette`, `system` already exist alongside it.

## Proposed split

- `mod.rs` (~50 lines) — stays the re-export/public surface: `pub mod` declarations,
  `pub use generator::{...}`, `pub use system::{...}`, `AutoSource` enum + `generate()`.
  This is what `crate::auto::generate` / `AutoSource` importers use — must not move.
- `auto/color.rs` (~130 lines) — `PaletteColor` struct + full impl (`luminance`,
  `saturation`, `contrast_ratio`, `shift_lightness`, `to_hsl`, `from_hsl`, `distance`) and
  its unit tests. This is self-contained pure color math, the natural first split target.
- `auto/scheme.rs` (~60 lines) — `ThemePalette` and `SystemColorScheme` struct
  definitions (pure data, no logic) — currently just data-carriers used by `palette`/
  `system`/`generator` submodules.
- Re-export `PaletteColor`, `ThemePalette`, `SystemColorScheme` from `mod.rs` via `pub use
  color::PaletteColor;` / `pub use scheme::{ThemePalette, SystemColorScheme};` so existing
  `crate::auto::PaletteColor` imports keep working.

## Coupling to flag

- `PaletteColor` is almost certainly used across `generator.rs`, `palette.rs`, and
  `system.rs` (siblings) — check those files for `use super::PaletteColor` before moving;
  the re-export in `mod.rs` should keep those `use` paths compiling unchanged.
- `generate()`'s cascade (system → wallpaper fallback → error) is the one piece of real
  control-flow logic here; keep it in `mod.rs` rather than splitting it out, since it's the
  documented "1:1 port of the legacy Tauri store cascade" and small.

## Verify after split

- `cargo test -p qbz-theme` (color math unit tests: luminance, contrast, hsl roundtrip,
  saturation — all currently in `mod.rs`'s test module).
- `cargo build --workspace` to confirm `qbz-theme::auto::{PaletteColor, ThemePalette,
  SystemColorScheme, generate, AutoSource}` importers elsewhere still resolve.
