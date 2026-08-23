//! `setup`/`teardown` + the `STATE`/`PALETTE` thread-locals every other
//! submodule (audio, render) reads/writes.

use std::cell::RefCell;

use slint::wgpu_28::wgpu;

use super::audio::Palette;
use super::resources::RenderState;

thread_local! {
    pub(super) static STATE: RefCell<Option<RenderState>> = const { RefCell::new(None) };
    pub(super) static PALETTE: RefCell<Palette> = const { RefCell::new(Palette::DEFAULT) };
}

/// Stash Slint's device/queue. Called once at `RenderingSetup`. Deliberately
/// CHEAP: no WGSL compilation, no texture allocation — those happen lazily in
/// `render_frame` on first shader use, so sessions that never open a shader
/// scene pay nothing at window paint. A second call re-stashes and drops any
/// built resources (only happens if the rendering surface is re-created).
pub fn setup(device: wgpu::Device, queue: wgpu::Queue) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(RenderState {
            device,
            queue,
            start: std::time::Instant::now(),
            res: None,
        });
    });
    log::info!("[shader] wgpu device/queue captured (GPU resources build on first shader use)");
}

/// Drop the pipeline at surface teardown.
pub fn teardown() {
    STATE.with(|s| *s.borrow_mut() = None);
    log::info!("[shader] wgpu underlay torn down");
}
