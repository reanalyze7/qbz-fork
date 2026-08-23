//! WGPU underlay render step: derive the enriched audio pack from the
//! latched drain state and render one dynamic-background shader frame.

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, ImmersiveState, NowPlayingState};

use super::drain::DrainState;

/// Render one GPU shader frame into the wgpu texture and hand it to
/// `ImmersiveState`. Only runs while the app-wide dynamic background picked
/// a scene (`app-shader-mode` > 0). The device/queue were captured by the
/// rendering notifier (main.rs); `render_frame` is a no-op until that fires.
pub(super) fn render(
    weak: &Weak<AppWindow>,
    state: &mut DrainState,
    new_spectral: &mut Option<Vec<f32>>,
) {
    state.last_tr *= 0.85;
    state.last_beat *= 0.88;
    let Some(w) = weak.upgrade() else {
        return;
    };
    let imm = w.global::<ImmersiveState>();
    let m = imm.get_app_shader_mode();
    if m == 0 {
        return;
    }
    // Derive the enriched audio pack from the latched cells.
    let mut bands8 = [0.0f32; 8];
    for i in 0..8 {
        bands8[i] = (state.last_bars16[2 * i] + state.last_bars16[2 * i + 1]) * 0.5;
    }
    let level = (state.last_energy[0]
        + state.last_energy[1]
        + state.last_energy[2]
        + state.last_energy[3]
        + state.last_energy[4])
        * 0.2;
    state.last_level_smooth = state.last_level_smooth * 0.96 + level * 0.04;
    // Forward-motion clock: host-side (rate is audio-dependent), wrapped at
    // an integer so fract()-based ring patterns stay continuous across the
    // wrap.
    state.last_phase += 0.012 + level * 0.02 + state.last_beat * 0.02;
    if state.last_phase >= 4096.0 {
        state.last_phase -= 4096.0;
    }
    // Real-time ceiling (mode 4): the highest band with signal, smoothed
    // (EMA) so the line tracks the audio without jitter.
    if m == 4 {
        if let Some(bins) = new_spectral.as_ref() {
            let n = bins.len();
            if n > 1 {
                let mut hi = 0usize;
                for (i, &v) in bins.iter().enumerate() {
                    if v > 0.05 {
                        hi = i;
                    }
                }
                let target = hi as f32 / (n - 1) as f32;
                state.last_peak = state.last_peak * 0.85 + target * 0.15;
            }
        }
    }
    // Spectral feed: mode 4 (ribbon) AND mode 5 (line bed) both consume the
    // 512-band frame. The ribbon also needs the playback fraction + a reset
    // (track change / seek).
    let sp = if m == 4 || m == 5 { new_spectral.take() } else { None };
    let (progress, reset) = if m == 4 {
        let np = w.global::<NowPlayingState>();
        let tid = np.get_track_id().to_string();
        let prog = np.get_progress();
        let rst = tid != state.last_track_id
            || prog + 0.01 < state.last_progress
            || prog > state.last_progress + 0.15;
        state.last_track_id = tid;
        state.last_progress = prog;
        (prog, rst)
    } else {
        (0.0, false)
    };
    let audio = crate::shader_underlay::FrameAudio {
        level,
        level_smooth: state.last_level_smooth,
        beat: state.last_beat,
        phase: state.last_phase,
        transient: state.last_tr,
        energy: state.last_energy,
        bands: bands8,
        spectral: sp,
        progress,
        reset,
        spectral_peak: state.last_peak,
    };
    // Window physical size → the underlay clamps its offscreen target to it
    // (capped at its 2560x1440 ceiling).
    let win_size = w.window().size();
    if let Some(img) =
        crate::shader_underlay::render_frame(m, &audio, win_size.width, win_size.height)
    {
        imm.set_shader_texture(img);
    }
}
