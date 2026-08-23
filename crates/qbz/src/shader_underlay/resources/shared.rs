//! Build the shared (size-independent) GPU resources plus the initial sized
//! set. Runs once, on the first frame a shader scene is active.

mod textures;

use slint::wgpu_28::wgpu;

use super::bgl::build_bgl;
use super::sized::build_sized;
use super::{GpuResources, SHADER_SOURCES};
use textures::{build_heights_tex, build_spectrogram};

pub(in crate::shader_underlay) fn build_shared(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    w: u32,
    h: u32,
) -> GpuResources {
    // One bind group layout shared by all pipelines: the uniform buffer
    // (binding 0) plus the feedback history texture (binding 1) + its sampler
    // (binding 2). Only the plasma fluid samples 1/2; tunnel/aurora declare just
    // binding 0 (a pipeline whose shader uses a SUBSET of the layout is valid).
    let bgl = build_bgl(device);

    let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("qbz-shader-uniforms"),
        size: std::mem::size_of::<super::super::Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Bilinear sampler for the plasma feedback history (the history texture
    // itself is size-dependent and lives in build_sized).
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("qbz-shader-feedback-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let spectrogram = build_spectrogram(device);
    let heights_tex = build_heights_tex(device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("qbz-shader-pl"),
        bind_group_layouts: &[&bgl],
        // wgpu 28.x: replaces `push_constant_ranges`. We use none.
        immediate_size: 0,
    });

    // Scene pipelines compile on their first use (see render_frame); start empty.
    let mut pipelines: Vec<Option<wgpu::RenderPipeline>> =
        Vec::with_capacity(SHADER_SOURCES.len());
    pipelines.resize_with(SHADER_SOURCES.len(), || None);

    let sized = build_sized(
        device,
        queue,
        &bgl,
        &uniform_buf,
        &sampler,
        &spectrogram,
        &heights_tex,
        w,
        h,
    );

    GpuResources {
        bgl,
        pipeline_layout,
        uniform_buf,
        sampler,
        spectrogram,
        heights_tex,
        pipelines,
        linebed_pipeline: None,
        sized,
    }
}
