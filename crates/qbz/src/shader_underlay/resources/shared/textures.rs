//! The two fixed-size persistent textures: the spectral-ribbon spectrogram
//! and the line-bed heights.

use slint::wgpu_28::wgpu;

use super::super::super::{LINEBED_BANDS, LINEBED_LINES, SPECTRO_BANDS, SPECTRO_COLS};

/// Spectral-ribbon spectrogram: 512 freq bands (width) × SPECTRO_COLS time
/// columns (height), R8. Written one row per spectral frame in render_frame,
/// sampled by spectral_ribbon.wgsl at binding 3.
pub(super) fn build_spectrogram(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("qbz-shader-spectrogram"),
        size: wgpu::Extent3d {
            width: SPECTRO_BANDS,
            height: SPECTRO_COLS,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// Line-bed heights: an R32Float texture (256 band wide × 200 line tall,
/// depth-ordered rows), uploaded per frame in the mode-5 path and read via
/// textureLoad in the line_bed vertex shader. A sampled texture avoids the
/// vertex-stage storage-buffer limit (0) on Slint's downlevel device.
pub(super) fn build_heights_tex(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("qbz-linebed-heights"),
        size: wgpu::Extent3d {
            width: LINEBED_BANDS,
            height: LINEBED_LINES,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
