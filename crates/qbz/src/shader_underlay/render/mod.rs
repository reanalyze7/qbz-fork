//! `render_frame` — render one frame of the active shader scene and hand it
//! back to Slint as an `Image`. Driven at 30 fps from visualizer.rs.

mod linebed_feed;
mod pipeline_pick;
mod spectral;
mod submit;
mod uniforms_push;

use slint::Image;

use super::lifecycle::{PALETTE, STATE};
use super::resources::{build_shared, build_sized};
use super::{FrameAudio, TEX_H, TEX_W};
use linebed_feed::feed_linebed;
use pipeline_pick::pick_pipeline;
use spectral::feed_spectral_ribbon;
use submit::submit_frame;
use uniforms_push::push_uniforms;

/// Render one frame of scene `mode` into the next pool target and return it as
/// a Slint `Image`. `mode` is the `ImmersiveState.shader-mode` value (1 =
/// plasma, 2 = tunnel, 3 = aurora, ...). `win_w`/`win_h` is the window's
/// PHYSICAL pixel size: the offscreen target is clamped to it (capped at
/// `TEX_W`x`TEX_H`) and the pool is rebuilt when it changes. Returns `None`
/// before `setup()` has run, for `mode <= 0`, or for a zero-sized window.
/// Driven at 30 fps from visualizer.rs while the immersive view is open with a
/// shader scene active.
pub fn render_frame(mode: i32, a: &FrameAudio, win_w: u32, win_h: u32) -> Option<Image> {
    if mode <= 0 || win_w == 0 || win_h == 0 {
        return None;
    }
    STATE.with(|s| {
        let mut borrow = s.borrow_mut();
        let st = borrow.as_mut()?;
        let (tw, th) = (win_w.min(TEX_W), win_h.min(TEX_H));

        // Lazy one-time build of the shared GPU resources (first shader open).
        if st.res.is_none() {
            let t0 = std::time::Instant::now();
            st.res = Some(build_shared(&st.device, &st.queue, tw, th));
            log::info!(
                "[shader] GPU resources built lazily in {:?} ({tw}x{th})",
                t0.elapsed()
            );
        }
        let res = st.res.as_mut()?;
        if res.sized.size != (tw, th) {
            res.sized = build_sized(
                &st.device,
                &st.queue,
                &res.bgl,
                &res.uniform_buf,
                &res.sampler,
                &res.spectrogram,
                &res.heights_tex,
                tw,
                th,
            );
            log::info!("[shader] render targets resized to {tw}x{th}");
        }

        // Rotate the target pool BEFORE taking the pipeline reference (the
        // pipeline borrow below is shared; this is the last `res` mutation
        // besides the lazy pipeline builds).
        let texture = res.sized.targets[res.sized.next_target].clone();
        res.sized.next_target = (res.sized.next_target + 1) % res.sized.targets.len();

        let pipeline = pick_pipeline(res, &st.device, mode)?.clone();

        let pal = PALETTE.with(|p| *p.borrow());
        push_uniforms(&st.queue, res, a, pal, st.start.elapsed().as_secs_f32(), tw, th);

        // Spectral ribbon (mode 4) / line bed (mode 5): feed their persistent
        // textures before the display pass.
        if mode == 4 {
            feed_spectral_ribbon(&st.queue, res, a);
        }
        if mode == 5 {
            feed_linebed(&st.queue, res, a);
        }

        submit_frame(&st.device, &st.queue, res, &pipeline, mode, texture, tw, th)
    })
}
