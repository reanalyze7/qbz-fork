# crates/qbz-theme/src/auto/palette.rs (309 lines)

## Summary
K-means-based dominant-color/theme-palette extraction from an album cover
image: image decode (bounded against decompression bombs), a from-scratch
k-means implementation, and role assignment (background shades + an
accent color chosen for saturation + WCAG AA contrast).

## Proposed split
This crate already uses a directory module (`auto/mod.rs`,
`auto/generator.rs`, `auto/palette.rs`, `auto/system.rs`) — so the split stays
inside `auto/` as sibling files rather than a new nested directory, keeping
the existing flat convention at this level.

- `auto/palette.rs` (~65 lines) — keep as the public entry point: module doc,
  `extract_palette` (image loading + bounded decode + downsample, lines
  10-52) and `extract_palette_from_pixels` (54-64). This is the file other
  code actually calls (`super::{PaletteColor, ThemePalette}` import stays).
- `auto/kmeans.rs` (~75 lines) — `Cluster` struct, `kmeans`, `rgb_dist_sq`
  (66-152). Pure numeric algorithm, zero dependency on `image`/`Path` — the
  clearest "pure computation" extraction in this file.
- `auto/palette_roles.rs` (~135 lines, right at the line) — `build_palette`,
  `find_best_accent`, `adjust_for_contrast` (154-266). If it lands just over
  130 once `use` lines are added, move `adjust_for_contrast` (252-266, ~15
  lines) into its own tiny `auto/contrast.rs`, since it's already a
  self-contained WCAG-AA-search helper independent of clustering.
- Tests (268-309) move with their target function: the two `kmeans_*` tests
  go to `auto/kmeans.rs` (`#[cfg(test)] mod tests`), the
  `extract_from_pixels_dark_dominant` / `monochrome_fallback_accent` tests
  (which exercise `extract_palette_from_pixels` → `build_palette` →
  `find_best_accent`) stay with `auto/palette.rs` or move to
  `auto/palette_roles.rs` — either is defensible since they test the
  end-to-end pipeline; keep them wherever `extract_palette_from_pixels` ends
  up (`auto/palette.rs`) for the shortest `use super::*` chain.

## Re-export surface
`auto/mod.rs` already declares `pub mod palette;` (confirmed: `auto/mod.rs`
lines 14-16 list `generator`, `palette`, `system` as `pub mod`). After the
split, `auto/mod.rs` needs one addition: `mod kmeans;` and `mod
palette_roles;` as PRIVATE (non-`pub`) submodules-of-a-submodule — actually,
since `kmeans.rs`/`palette_roles.rs` are used only BY `palette.rs`, they
should be declared as `mod kmeans; mod palette_roles;` INSIDE `auto/palette.rs`
turning it into `auto/palette/mod.rs` + `auto/palette/kmeans.rs` + `auto/palette/palette_roles.rs`
(one more level of nesting), NOT declared at the `auto/mod.rs` level — this
keeps `kmeans`/`build_palette`/`find_best_accent` crate-internal
implementation details invisible outside `auto::palette`, exactly matching
today's visibility (they are all private `fn`/`struct` today, not `pub`).
`auto/mod.rs`'s existing `pub mod palette;` then transparently becomes `pub
mod palette;` pointing at a directory instead of a file — zero external
caller impact, since `extract_palette`/`extract_palette_from_pixels` are the
only `pub fn` and they stay in `auto/palette/mod.rs`.

## Coupling / watch out
- `build_palette` (now in `palette_roles.rs`) needs `super::{PaletteColor,
  ThemePalette}` imported via `super::super::` once nested one level deeper
  under `auto/palette/` — double-check the `use` path depth after adding a
  directory level (it was `use super::{PaletteColor, ThemePalette};`
  resolving to `auto::{PaletteColor, ThemePalette}`; from `auto/palette/
  palette_roles.rs` this becomes `use super::super::{PaletteColor,
  ThemePalette};` or more idiomatically `use crate::auto::{PaletteColor,
  ThemePalette};` — prefer the crate-absolute form during the split to avoid
  super-super confusion).
- `kmeans`'s centroid-init step (`let step = n / k;`) has a latent
  divide-by-zero risk if `k > n` after the `k.min(n)` guard runs — that guard
  already protects it (line 79), just confirm the guard travels with the
  function into its new file (it's 3 lines above the loop, easy to keep
  together).
- `is_dark`/`is_monochrome` thresholds (0.5 luminance, 40.0 max_distance) in
  `build_palette` are tuned constants ported 1:1 from the Tauri reference
  implementation (per the file's own doc comment) — do not "clean up" or
  rename them during the split; they must stay bit-identical to preserve the
  1:1 port guarantee the module doc promises.

## Verify after split
- `cargo build -p qbz-theme` and `cargo build --workspace`.
- `cargo test -p qbz-theme` — the 3 existing unit tests
  (`kmeans_basic_two_clusters`, `extract_from_pixels_dark_dominant`,
  `monochrome_fallback_accent`) must stay green in their new locations.
- `cargo clippy -p qbz-theme`.
- Smoke-test importers: `grep -rn "auto::palette::\|palette::extract_palette"
  crates/qbz crates/qbz-theme` — confirm `extract_palette`/
  `extract_palette_from_pixels` call sites (auto-theme generation from
  now-playing cover art) still compile and produce the same palette for a
  known test image.
