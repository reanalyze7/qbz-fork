//! The rotating 3-deep pool of offscreen render targets.

use slint::wgpu_28::wgpu;

use super::super::super::TEX_FORMAT;

/// The rotating offscreen pool. Image::try_from REQUIRES Rgba8Unorm/Srgb +
/// TEXTURE_BINDING | RENDER_ATTACHMENT (Slint graphics/wgpu_28.rs); COPY_SRC
/// feeds the plasma history copy.
pub(super) fn build_targets(device: &wgpu::Device, w: u32, h: u32) -> [wgpu::Texture; 3] {
    let make_target = || {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("qbz-shader-target"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEX_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    [make_target(), make_target(), make_target()]
}
