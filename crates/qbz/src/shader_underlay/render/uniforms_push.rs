//! Assemble this frame's `Uniforms` (time + audio + palette) and upload it.

use slint::wgpu_28::wgpu;

use super::super::resources::GpuResources;
use super::super::{uniforms_bytes, FrameAudio, Uniforms};
use super::super::audio::Palette;

#[allow(clippy::too_many_arguments)]
pub(super) fn push_uniforms(queue: &wgpu::Queue, res: &GpuResources, a: &FrameAudio, pal: Palette, time: f32, tw: u32, th: u32) {
    let uniforms = Uniforms {
        time,
        phase: a.phase,
        beat: a.beat,
        level: a.level,
        res_x: tw as f32,
        res_y: th as f32,
        level_smooth: a.level_smooth,
        transient: a.transient,
        energy_lo: [a.energy[0], a.energy[1], a.energy[2], a.energy[3]],
        energy_hi: [a.energy[4], a.spectral_peak, 0.0, 0.0],
        bands_lo: [a.bands[0], a.bands[1], a.bands[2], a.bands[3]],
        bands_hi: [a.bands[4], a.bands[5], a.bands[6], a.bands[7]],
        primary: pal.primary,
        secondary: pal.secondary,
        accent: pal.accent,
    };
    queue.write_buffer(&res.uniform_buf, 0, uniforms_bytes(&uniforms));
}
