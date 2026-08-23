//! Line bed (mode 5): push the new spectral frame into the depth ring,
//! reshape it (Tauri's 512→256 chain), and upload the 200×256 heights.

use slint::wgpu_28::wgpu;

use super::super::f32_bytes;
use super::super::reshape::LINEBED;
use super::super::resources::GpuResources;
use super::super::{FrameAudio, LINEBED_BANDS, LINEBED_LINES};

pub(super) fn feed_linebed(queue: &wgpu::Queue, res: &GpuResources, a: &FrameAudio) {
    if let Some(ref bins) = a.spectral {
        if !bins.is_empty() {
            LINEBED.with(|lb| lb.borrow_mut().push(bins));
        }
    }
    LINEBED.with(|lb| {
        let lb = lb.borrow();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &res.heights_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            f32_bytes(&lb.ring),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(LINEBED_BANDS * 4),
                rows_per_image: Some(LINEBED_LINES),
            },
            wgpu::Extent3d {
                width: LINEBED_BANDS,
                height: LINEBED_LINES,
                depth_or_array_layers: 1,
            },
        );
    });
}
