//! GPU resource construction: bind group layout / buffers / textures /
//! pipelines. Everything here is built lazily on the first frame a shader
//! scene is active — compiling six WGSL pipelines + allocating the
//! history/spectrogram textures at first window paint costs startup time
//! and VRAM even when the immersive shaders are never opened.

mod bgl;
mod pipelines;
mod shared;
mod sized;

pub(super) use pipelines::{build_linebed_pipeline, build_scene_pipeline};
pub(super) use shared::build_shared;
pub(super) use sized::build_sized;

use slint::wgpu_28::wgpu;

/// GPU resources whose size tracks the window (recreated on resize by
/// `render_frame`): the plasma history accumulator, the bind group (the VIEW of
/// `history` is baked into it), and the rotating pool of offscreen targets.
pub(super) struct SizedResources {
    /// Clamped target size: min(window physical size, TEX_W x TEX_H).
    pub(super) size: (u32, u32),
    /// Persistent feedback accumulator for the plasma fluid (Direction A). The
    /// plasma shader samples it (binding 1); each plasma frame is copied into it
    /// after the pass, so the next frame advects the previous one.
    pub(super) history: wgpu::Texture,
    pub(super) bind_group: wgpu::BindGroup,
    /// Rotating 3-deep pool of offscreen targets (replaces a fresh 14.7 MB
    /// texture per frame). Safe: Slint's femtovg renderer holds only the
    /// CURRENT frame's texture (per-item graphics cache keyed on the `source`
    /// property; `ImageCacheKey` is None for WGPU textures so nothing lingers
    /// in the shared texture cache — see vendored femtovg images.rs) — so a
    /// pool texture is re-rendered ~2 ticks after Slint stopped referencing
    /// it, and same-queue submission ordering serializes any residual GPU
    /// reads regardless.
    pub(super) targets: [wgpu::Texture; 3],
    pub(super) next_target: usize,
}

/// Everything built lazily on the FIRST frame a shader scene is active —
/// compiling six WGSL pipelines + allocating the history/spectrogram textures
/// at first window paint costs startup time and VRAM even when the immersive
/// shaders are never opened. Scene pipelines are additionally per-scene lazy.
pub(super) struct GpuResources {
    pub(super) bgl: wgpu::BindGroupLayout,
    pub(super) pipeline_layout: wgpu::PipelineLayout,
    pub(super) uniform_buf: wgpu::Buffer,
    pub(super) sampler: wgpu::Sampler,
    /// Persistent spectrogram for the spectral-ribbon scene (binding 3); written
    /// one column per spectral frame, sampled for display. Fixed size.
    pub(super) spectrogram: wgpu::Texture,
    /// Line-bed (mode 5) 256×200 heights texture (binding 4). Fixed size.
    pub(super) heights_tex: wgpu::Texture,
    /// One render pipeline per fullscreen shader scene, indexed like
    /// `SHADER_SOURCES` (modes 1-4 → 0..3, mode 6 → 4), each compiled+cached on
    /// its first use. All share one pipeline layout + bind group (uniform +
    /// history texture + sampler); scenes ignore the bindings they don't
    /// declare (a shader using a SUBSET of the layout is valid). `render_frame`
    /// picks the pipeline by index and, for plasma, copies the frame into
    /// `history`.
    pub(super) pipelines: Vec<Option<wgpu::RenderPipeline>>,
    /// Line-bed (mode 5): its own line-strip pipeline, also lazy.
    pub(super) linebed_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) sized: SizedResources,
}

pub(super) struct RenderState {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) start: std::time::Instant,
    /// None until the first `render_frame` with a shader scene active.
    pub(super) res: Option<GpuResources>,
}

/// The WGSL source for each scene, in mode order (index = mode - 1). Adding a
/// scene = one `include_str!` here + one entry in the picker (state/UI). All
/// must declare the SAME `Uniforms` block (group0/binding0) as plasma.wgsl.
pub(super) const SHADER_SOURCES: &[&str] = &[
    include_str!("../../../../qbz-ui/ui/shaders/plasma.wgsl"),          // [0] mode 1
    include_str!("../../../../qbz-ui/ui/shaders/tunnel.wgsl"),          // [1] mode 2
    include_str!("../../../../qbz-ui/ui/shaders/aurora.wgsl"),          // [2] mode 3
    include_str!("../../../../qbz-ui/ui/shaders/spectral_ribbon.wgsl"), // [3] mode 4
    include_str!("../../../../qbz-ui/ui/shaders/liquid_spectrum.wgsl"), // [4] mode 6
    include_str!("../../../../qbz-ui/ui/shaders/ambient.wgsl"),         // [5] mode 7 (app-wide)
];
