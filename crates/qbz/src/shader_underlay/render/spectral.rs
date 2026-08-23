//! Spectral ribbon (mode 4): feed the persistent spectrogram before the
//! display pass. Reset (clear) on track-change/seek, then write the new
//! 512-band column at the playback-time position (paint-as-you-play).

use slint::wgpu_28::wgpu;

use super::super::reshape::{SPECTRO_LAST_COL, SPECTRO_SCRATCH};
use super::super::resources::GpuResources;
use super::super::{FrameAudio, SPECTRO_BANDS, SPECTRO_COLS};

pub(super) fn feed_spectral_ribbon(queue: &wgpu::Queue, res: &GpuResources, a: &FrameAudio) {
    if a.reset {
        // Full-texture clear — rare (track change / seek only), so the
        // 1 MB zero buffer is not worth keeping around.
        let zeros = vec![0u8; (SPECTRO_BANDS * SPECTRO_COLS) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &res.spectrogram,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &zeros,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SPECTRO_BANDS),
                rows_per_image: Some(SPECTRO_COLS),
            },
            wgpu::Extent3d {
                width: SPECTRO_BANDS,
                height: SPECTRO_COLS,
                depth_or_array_layers: 1,
            },
        );
        SPECTRO_LAST_COL.with(|c| c.set(0));
    }
    let Some(ref bins) = a.spectral else { return };
    if bins.is_empty() {
        return;
    }
    let col = (a.progress.clamp(0.0, 1.0) * (SPECTRO_COLS - 1) as f32) as u32;
    let n = SPECTRO_BANDS as usize;
    SPECTRO_SCRATCH.with(|scratch| {
        let (row, data) = &mut *scratch.borrow_mut();
        row.clear();
        row.resize(n, 0);
        for (i, slot) in row.iter_mut().enumerate() {
            if i < bins.len() {
                *slot = (bins[i].clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
        // Gap-fill: paint every column skipped since the last write
        // (progress updates ~1 Hz, so the column jumps several slots).
        let last = SPECTRO_LAST_COL.with(|c| c.get());
        let start = if col > last { last + 1 } else { col };
        let count = col + 1 - start;
        data.clear();
        data.reserve(n * count as usize);
        for _ in 0..count {
            data.extend_from_slice(&row[..]);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &res.spectrogram,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: start, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &data[..],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SPECTRO_BANDS),
                rows_per_image: Some(count),
            },
            wgpu::Extent3d {
                width: SPECTRO_BANDS,
                height: count,
                depth_or_array_layers: 1,
            },
        );
        SPECTRO_LAST_COL.with(|c| c.set(col));
    });
}
