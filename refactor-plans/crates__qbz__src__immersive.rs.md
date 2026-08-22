# crates/qbz/src/immersive.rs (425 lines)

## Summary
Pure image-processing helpers producing Immersive-view visual material from
decoded album-art pixels: a shared tiny-downscale step, the blurred
"atmosphere" texture pipeline, glow/spectrum/lyrics-accent color extraction,
a dominant-swatch picker, and low-level RGB/HSL + pixel-adjustment math.

## Proposed split
This file is already pure computation (no I/O) end-to-end, so the split is
by domain/output rather than pure/IO/render.

- `immersive/mod.rs` (~15 lines) — module doc (lines 1-8), `pub use`
  re-exports of every public function from the sibling files below.
- `immersive/atmosphere.rs` (~90 lines) — `cover_tiny_samples`,
  `generate_atmosphere`, `atmosphere_from_tiny8`, `generate_atmosphere_image`
  — the shared downscale + the blurred/warmed/vignetted background pipeline
  (these four are one conceptual pipeline: tiny-sample -> atmosphere bytes ->
  Slint image).
- `immersive/palette.rs` (~200 lines) — `glow_color`, `spectrum_colors`,
  `lyrics_accent_color`, `dominant_cover_color` — the four color-extraction
  functions consumed by AlbumReactive/Spectrum/Lyrics/playlist-card
  features. `spectrum_colors` and `lyrics_accent_color` duplicate the same
  hue-histogram binning (lines 106-120 and 192-205 are near-identical) —
  keep them in the same file so a future dedup (extract a shared
  `dominant_chromatic_hue(&RgbaImage) -> Option<f32>` helper) is a
  same-file, low-risk change rather than a cross-file one.
- `immersive/color_math.rs` (~90 lines) — `rgb_to_hsl`, `hsl_to_rgb` — the
  two pure colorspace-conversion primitives used by `palette.rs`.
- `immersive/image_adjust.rs` (~95 lines) — `color_adjust`,
  `saturate_brightness_contrast`, `vignette` — the pixel-buffer adjustment
  passes used only by `atmosphere.rs`'s pipeline.

## Re-export surface
`immersive/mod.rs` re-exports every function used by callers today
(`cover_tiny_samples`, `generate_atmosphere`, `atmosphere_from_tiny8`,
`generate_atmosphere_image`, `glow_color`, `spectrum_colors`,
`lyrics_accent_color`, `dominant_cover_color`) at `crate::immersive::*` —
existing callers (Immersive view controller, AlbumReactive, Spectrum
visualizer, lyrics focus, playlist card letterbox) keep using
`crate::immersive::<fn>` unchanged. The private helpers (`rgb_to_hsl`,
`hsl_to_rgb`, `color_adjust`, etc.) stay non-`pub` (or `pub(super)` if a
sibling file needs them) and are never re-exported.

## Coupling / watch out
- `atmosphere.rs`'s `atmosphere_from_tiny8` calls `color_adjust` and
  `vignette` (both proposed for `image_adjust.rs`) — these need `pub(crate)`
  or `pub(super)` visibility from `image_adjust.rs`.
- `palette.rs`'s four functions call `rgb_to_hsl`/`hsl_to_rgb` (proposed for
  `color_math.rs`) — same visibility bump needed (`pub(super)` from the
  `immersive` module is enough since both are children of `immersive/`).
- `spectrum_colors` and `lyrics_accent_color` are near-duplicate hue-binning
  logic as noted above — resist the urge to "fix" this during the pure
  mechanical split; flag it as a follow-up cleanup PR instead so the split
  itself stays behavior-preserving.
- No shared mutable state anywhere in this file — everything is `fn(&[u8],
  ...) -> T` or `fn(&RgbaImage) -> T`, so the split carries zero
  thread-safety/lifetime risk.

## Verify after split
- `cargo test -p qbz immersive::` (check whether any unit tests exist for
  these functions today; if not, this is a good place to note that a
  follow-up should add golden-pixel tests for `hsl_to_rgb`/`rgb_to_hsl`
  round-tripping and `dominant_cover_color` on a synthetic 2x2 buffer).
- `cargo check -p qbz` and grep `crate::immersive::` across `crates/qbz/src/`
  for every call site (Immersive view, AlbumReactive glow, Spectrum
  visualizer colors, lyrics accent, playlist-card dominant swatch) to
  confirm none break.
