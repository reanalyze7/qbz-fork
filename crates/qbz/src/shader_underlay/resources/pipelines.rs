//! Scene pipeline builders: the shared fullscreen-triangle pipeline (one per
//! `SHADER_SOURCES` entry) and line-bed's own line-strip pipeline.

use slint::wgpu_28::wgpu;

use super::SHADER_SOURCES;
use super::super::TEX_FORMAT;

/// Compile one fullscreen scene pipeline (`SHADER_SOURCES[idx]`). All scenes
/// share the pipeline layout / bind group / uniform buffer / vertex stage; only
/// the fragment shader source differs. `vs_main` / `fs_main` entry points are
/// identical across the scene WGSL files (the fullscreen-triangle template).
pub(in crate::shader_underlay) fn build_scene_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    idx: usize,
) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("qbz-shader-module"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SOURCES[idx].into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("qbz-shader-pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: TEX_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    log::debug!("[shader] scene pipeline {idx} built");
    pipeline
}

/// Line-bed (mode 5): a SEPARATE pipeline — line-strip topology + alpha blend
/// + the projecting vertex shader. Shares the pipeline layout / bind group.
pub(in crate::shader_underlay) fn build_linebed_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    let linebed_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("qbz-linebed-module"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../../qbz-ui/ui/shaders/line_bed.wgsl").into(),
        ),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("qbz-linebed-pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &linebed_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &linebed_module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: TEX_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
