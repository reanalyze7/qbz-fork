//! The plasma feedback history texture: allocate + clear-to-black once so
//! the first plasma frame samples black, not uninitialized GPU memory.

use slint::wgpu_28::wgpu;

use super::super::super::TEX_FORMAT;

pub(super) fn build_history(device: &wgpu::Device, queue: &wgpu::Queue, w: u32, h: u32) -> wgpu::Texture {
    let history = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("qbz-shader-history"),
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
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let history_view = history.create_view(&wgpu::TextureViewDescriptor::default());

    // Clear the accumulator once so the first plasma frame samples black, not
    // uninitialized GPU memory.
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("qbz-shader-history-clear"),
    });
    {
        let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("qbz-shader-history-clear-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &history_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    queue.submit(Some(enc.finish()));

    history
}
