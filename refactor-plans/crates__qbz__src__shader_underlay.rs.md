# crates/qbz/src/shader_underlay.rs (1036 lines)

## Summary
WGPU fragment-shader background renderer for the ImmersiveView: builds/owns
GPU pipelines/textures for 6 fullscreen shader scenes + a line-strip line-bed
scene, renders one frame per tick into a rotating texture pool, and hands the
result back to Slint as an `Image`.

## Proposed split
Split by responsibility: data/constants, DSP-ish reshaping math (pure), GPU
resource construction, and the per-frame render driver.

- `shader_underlay/mod.rs` (~55 lines) — module doc, `pub use` re-exports of
  `setup`, `teardown`, `set_palette`, `render_frame`, `FrameAudio`. Keeps the
  `TEX_W`/`TEX_H`/`TEX_FORMAT`/`SPECTRO_*`/`LINEBED_*` consts (shared by
  multiple submodules) and the `Uniforms` struct + its size assert + byte
  helpers (`uniforms_bytes`, `f32_bytes`) since nearly everything downstream
  depends on them.
- `shader_underlay/audio.rs` (~40 lines) — `FrameAudio` struct + doc (the
  per-frame driver input type), `Palette` struct + `DEFAULT` + `set_palette`.
- `shader_underlay/reshape.rs` (~100 lines) — the pure line-bed DSP chain:
  `reshape_512_to_256`, `apply_average`, `smooth3`, `LineBedState` +
  `LineBedState::new`/`push` and its two `thread_local!`s (`LINEBED`,
  `SPECTRO_LAST_COL`/`SPECTRO_SCRATCH` can stay here too since they are only
  used by the spectral-ribbon/line-bed math).
- `shader_underlay/resources.rs` (~330 lines) — `SizedResources`,
  `GpuResources`, `RenderState`, `SHADER_SOURCES` const, `build_shared`,
  `build_sized`, `build_scene_pipeline`, `build_linebed_pipeline` (the
  GPU-object construction code; this is the biggest chunk so it likely needs
  a further split into `resources/shared.rs` (bgl/uniform buf/sampler/
  textures + `build_shared`) and `resources/pipelines.rs` (the two
  pipeline-builder fns) to land under 130 lines each).
- `shader_underlay/lifecycle.rs` (~40 lines) — `setup`, `teardown`, the
  `STATE`/`PALETTE` thread_locals.
- `shader_underlay/render.rs` (~280 lines) — `render_frame` itself (the
  biggest single function — 764-1036, ~270 lines). This one function will
  likely need internal extraction (not just file-move) into smaller private
  helpers, e.g. `pick_pipeline`, `write_spectral_ribbon_column`,
  `write_linebed_heights`, `submit_frame` — flag this as the highest-risk
  part of the split since `render_frame` is a single large `STATE.with`
  closure with many local mutable borrows of `res`/`st`.

## Re-export surface
`shader_underlay/mod.rs` re-exports `setup`, `teardown`, `set_palette`,
`render_frame`, `FrameAudio` at `crate::shader_underlay::*` — these are the
only symbols called from `main.rs`'s rendering-notifier wiring and
`visualizer.rs`'s 30fps drain, so no caller-visible path changes.

## Coupling / watch out
- `render_frame` mutates `GpuResources` fields lazily (pipelines compiled on
  first use) AND reads/writes several `thread_local!`s (`STATE`, `PALETTE`,
  `LINEBED`, `SPECTRO_LAST_COL`, `SPECTRO_SCRATCH`) — if `render.rs` and
  `resources.rs`/`reshape.rs` end up in different files, these thread_locals
  must stay visible (`pub(super)` or `pub(crate)`) to `render.rs`.
- The WGSL `Uniforms` repr(C) layout MUST byte-match all 6 `.wgsl` shader
  files (`ui/shaders/*.wgsl`) — the size assert (`assert!(size_of::<Uniforms>() == 144)`)
  is the only compile-time guard; do not reorder fields across the split.
- `SHADER_SOURCES` indexing (`mode - 1` etc., with mode 5 = line-bed's own
  pipeline, mode 6/7 special-cased) is fragile hand-rolled logic in
  `render_frame` — keep it in ONE place, do not duplicate the mode->index
  mapping across files.
- `build_shared`/`build_sized` share many parameters (bgl, uniform_buf,
  sampler, spectrogram, heights_tex) — if split into separate files, prefer
  passing `&GpuResources`-shaped structs over threading 6+ raw params, but
  that is a follow-up refactor, not part of this mechanical split.

## Verify after split
- `cargo check -p qbz --features <wgpu-underlay-feature-if-gated>` (check
  Cargo.toml for a feature flag gating this file — it's a "spike"/experimental
  module per the doc comment).
- No `#[cfg(test)]` block exists in this file — no unit tests to keep green;
  verification here is compile-only plus a manual smoke-test.
- Manual: run the app, open the ImmersiveView, cycle through shader modes
  1-7 (plasma/tunnel/aurora/spectral-ribbon/line-bed/liquid-spectrum/ambient)
  and confirm each renders without panicking and audio-reactivity still works.
