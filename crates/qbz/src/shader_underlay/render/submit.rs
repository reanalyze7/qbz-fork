//! Render into the picked pool texture, submit, and hand back a Slint
//! `Image`.

use slint::wgpu_28::wgpu;
use slint::Image;

use super::super::resources::GpuResources;
use super::super::{LINEBED_BANDS, LINEBED_LINES, LINEBED_SUBDIV};

#[allow(clippy::too_many_arguments)]
pub(super) fn submit_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    res: &GpuResources,
    pipeline: &wgpu::RenderPipeline,
    mode: i32,
    texture: wgpu::Texture,
    tw: u32,
    th: u32,
) -> Option<Image> {
    // Render into the pool texture picked by the caller (a clone = the same
    // underlying wgpu texture; Image::try_from consumes our handle while
    // the pool keeps its own).
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("qbz-shader-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("qbz-shader-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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
            // wgpu 28.x render passes also carry the multiview layer mask;
            // we don't use multiview (single 2D target), so None.
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &res.sized.bind_group, &[]);
        if mode == 5 {
            // 200 instanced line strips, each a subdivided Catmull-Rom curve
            // of (255 * SUBDIV + 1) points.
            pass.draw(0..((LINEBED_BANDS - 1) * LINEBED_SUBDIV + 1), 0..LINEBED_LINES);
        } else {
            pass.draw(0..3, 0..1);
        }
    }
    // Plasma fluid (mode 1) feeds back: copy this frame into the history
    // accumulator so the next frame advects it. Tunnel/aurora skip this.
    if mode == 1 {
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &res.sized.history,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: tw,
                height: th,
                depth_or_array_layers: 1,
            },
        );
    }
    queue.submit(Some(encoder.finish()));

    match Image::try_from(texture) {
        Ok(img) => Some(img),
        Err(e) => {
            log::warn!("[shader] Image::try_from failed: {e:?}");
            None
        }
    }
}
