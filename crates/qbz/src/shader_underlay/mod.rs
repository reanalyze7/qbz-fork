//! WGPU UNDERLAY SPIKE — GPU fragment-shader background for the ImmersiveView.
//!
//! Validates the renderer-swap path (`renderer-femtovg` GL -> `renderer-femtovg-wgpu`)
//! by rendering a WGSL fragment shader into a wgpu texture and handing it back to
//! Slint as an `Image` (the texture-into-scene shape from Slint's upstream
//! `wgpu_texture` example). The texture is bound to an `Image` placed at the
//! bottom of `ImmersiveView`'s z-stack (see ui/immersive/ImmersiveView.slint).
//!
//! Lifecycle:
//!   * `setup()` is called once by the rendering notifier in main.rs at
//!     `RenderingState::RenderingSetup`, with Slint's OWN wgpu Device/Queue —
//!     mandatory so `Image::try_from` operates on the same device Slint renders
//!     with. It only STASHES them (cheap); pipelines/textures build lazily on
//!     the first frame a shader scene is active (one-time hitch on first open),
//!     and each scene's pipeline compiles on its first use.
//!   * `render_frame()` is called from the 30 fps drain in visualizer.rs while a
//!     shader scene is active AND the immersive view is open. It renders one
//!     frame into the next texture of a rotating 3-deep pool sized to the
//!     window (capped at `TEX_W`x`TEX_H`) and returns an `Image`. The caller
//!     sets it on `ImmersiveState.shader-texture`.
//!   * `teardown()` clears the state at `RenderingState::RenderingTeardown`.
//!
//! All three run on the UI thread (notifier + Timer share it), so the state lives
//! in a `thread_local`. This file is downstream of the read-only visualizer feed
//! and touches NONE of the protected audio backend.

use slint::wgpu_28::wgpu;

mod audio;
mod lifecycle;
mod render;
mod reshape;
mod resources;

pub use audio::{set_palette, FrameAudio};
pub use lifecycle::{setup, teardown};
pub use render::render_frame;

/// Offscreen render target CEILING. The actual target tracks the window's
/// physical pixel size (no point burning fill rate above it on small screens —
/// Raspberry Pi class hardware) but never exceeds this cap; the `Image` is
/// shown with `image-fit: fill`, so the immersive viewport stretches whatever
/// size to fit.
pub(super) const TEX_W: u32 = 2560;
pub(super) const TEX_H: u32 = 1440;
pub(super) const TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Spectral-ribbon spectrogram dims: 512 frequency bands wide × time columns
/// tall (R8). One row is written per new spectral frame at the playback-time
/// column; un-written columns stay zero (the ribbon paints as the track plays).
pub(super) const SPECTRO_BANDS: u32 = 512;
pub(super) const SPECTRO_COLS: u32 = 2048;

/// Line-bed lattice: 200 depth lines × 256 frequency points (matches the Tauri
/// LinebedPanel NUM_LINES / VISUAL_BANDS).
pub(super) const LINEBED_LINES: u32 = 200;
pub(super) const LINEBED_BANDS: u32 = 256;
/// Each band span is subdivided into LINEBED_SUBDIV Catmull-Rom steps in the
/// vertex shader so the polylines read as smooth curves. MUST match SUBDIV in
/// line_bed.wgsl. Vertex count per line = (LINEBED_BANDS - 1) * SUBDIV + 1.
pub(super) const LINEBED_SUBDIV: u32 = 6;

/// Mirrors the WGSL `Uniforms` struct in all three `ui/shaders/*.wgsl`. Plain
/// `f32` / `[f32;4]` (align 4) with manual field ordering so the byte offsets
/// match the WGSL std140 layout exactly (every `vec4` lands on a 16-byte
/// boundary; the `res_x`/`res_y` pair is read as a `vec2`), with no vec types or
/// bytemuck needed. 144 bytes = 9×vec4. Offset table:
/// qbz-nix-docs/immersive-shaders-2026-06-28/00-analysis-and-design-spec.md §2.2.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct Uniforms {
    pub(super) time: f32,           //   0
    pub(super) phase: f32,          //   4  audio-reactive forward-motion clock (host accumulator)
    pub(super) beat: f32,           //   8  onset envelope (~0.88 decay) — the "punch"
    pub(super) level: f32,          //  12  instantaneous overall level = mean(energy bands)
    pub(super) res_x: f32,          //  16  } WGSL reads these two as `resolution: vec2<f32>`
    pub(super) res_y: f32,          //  20  }
    pub(super) level_smooth: f32,   //  24  slow EMA of level (breathing / inertia)
    pub(super) transient: f32,      //  28  fast transient (*0.85) — kept for the legacy bodies
    pub(super) energy_lo: [f32; 4], //  32  sub, bass, mid, presence
    pub(super) energy_hi: [f32; 4], //  48  air, 0, 0, 0
    pub(super) bands_lo: [f32; 4],  //  64  log bars 0..3
    pub(super) bands_hi: [f32; 4],  //  80  log bars 4..7
    pub(super) primary: [f32; 4],   //  96  album-art palette (rgb, a = 1)
    pub(super) secondary: [f32; 4], // 112
    pub(super) accent: [f32; 4],    // 128
} // size_of == 144, align 4

// Drift guard: WGSL is compiled at runtime (naga), so cargo cannot catch a
// Rust/WGSL layout mismatch. This catches the Rust side; the WGSL side is the
// manual offset table in the spec (and the Slice-0 canary — the unchanged
// shaders must look identical).
const _: () = assert!(core::mem::size_of::<Uniforms>() == 144);

/// View `Uniforms` as raw bytes for `Queue::write_buffer`. Sound: `Uniforms` is
/// `#[repr(C)]`, all-`f32`, no padding holes with undefined values we read back —
/// every byte is part of a defined `f32` field.
pub(super) fn uniforms_bytes(u: &Uniforms) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (u as *const Uniforms) as *const u8,
            std::mem::size_of::<Uniforms>(),
        )
    }
}

/// View a `&[f32]` as bytes for the heights upload. Same soundness as
/// `uniforms_bytes` — plain `f32`, no padding holes.
pub(super) fn f32_bytes(s: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(s.as_ptr() as *const u8, std::mem::size_of_val(s)) }
}
