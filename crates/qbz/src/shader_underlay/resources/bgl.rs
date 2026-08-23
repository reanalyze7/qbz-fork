//! The shared bind group layout: uniform buffer + feedback history texture +
//! sampler + spectrogram + line-bed heights. One layout for every pipeline;
//! a shader using a SUBSET of it is valid (tunnel/aurora ignore most slots).

use slint::wgpu_28::wgpu;

pub(super) fn build_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("qbz-shader-bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                // VERTEX too: the line-bed vertex shader reads `resolution`.
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // Binding 3: the spectral-ribbon spectrogram (R8). Only that scene
            // samples it; the others declare a subset of the layout (valid).
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Binding 4: the line-bed heights as an R32Float TEXTURE (256 band ×
            // 200 line), read via textureLoad in the VERTEX stage by line_bed.wgsl.
            // A SAMPLED texture (not a storage buffer) so it works without the
            // VERTEX_STORAGE downlevel capability that Slint's device lacks (a
            // vertex storage buffer fails BGL creation: limit is 0). Others ignore.
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}
