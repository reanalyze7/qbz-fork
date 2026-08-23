//! (Re)build the window-size-tracking resources: the plasma history
//! accumulator, the rotating target pool, and the bind group (the history VIEW
//! is baked into it, so a size change forces a bind-group rebuild). A resize
//! drops the plasma feedback content — it re-accumulates within a few frames.

mod history;
mod targets;

use slint::wgpu_28::wgpu;

use super::SizedResources;
use history::build_history;
use targets::build_targets;

#[allow(clippy::too_many_arguments)]
pub(in crate::shader_underlay) fn build_sized(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bgl: &wgpu::BindGroupLayout,
    uniform_buf: &wgpu::Buffer,
    sampler: &wgpu::Sampler,
    spectrogram: &wgpu::Texture,
    heights_tex: &wgpu::Texture,
    w: u32,
    h: u32,
) -> SizedResources {
    let history = build_history(device, queue, w, h);
    let history_view = history.create_view(&wgpu::TextureViewDescriptor::default());
    let spectrogram_view = spectrogram.create_view(&wgpu::TextureViewDescriptor::default());
    let heights_view = heights_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("qbz-shader-bg"),
        layout: bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&history_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&spectrogram_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&heights_view),
            },
        ],
    });

    let targets = build_targets(device, w, h);

    SizedResources {
        size: (w, h),
        history,
        bind_group,
        targets,
        next_target: 0,
    }
}
