//! The ~30fps drain-timer tick body, extracted to a free function so
//! `install.rs` can dispatch to it with a one-line call.
use slint::ComponentHandle;

use std::rc::Rc;
use std::sync::Arc;

use slint::{Model, VecModel, Weak};

use crate::{AppWindow, NowPlayingState, VisualizerState};

use super::cells::VizCells;
use super::shader;

/// Cross-tick state for the drain closure. Bundled into one struct (rather
/// than a long parameter list) so `install()` owns a single `let mut state =
/// DrainState::default();` and passes `&mut state` in each tick.
#[derive(Default)]
pub(super) struct DrainState {
    pub(super) last_tr: f32,
    pub(super) last_energy: [f32; 5],
    pub(super) last_bars16: [f32; 16],
    pub(super) last_level_smooth: f32,
    pub(super) last_beat: f32,
    pub(super) last_phase: f32,
    pub(super) last_track_id: String,
    pub(super) last_progress: f32,
    pub(super) last_peak: f32,
    drain_saw_playing: bool,
}

/// One drain tick: fan out the latest latched frames into the persistent
/// Slint models, then (if a dynamic background shader is active) render one
/// WGPU underlay frame from the enriched audio pack via `shader::render`.
#[allow(clippy::too_many_arguments)]
pub(super) fn tick(
    state: &mut DrainState,
    weak: &Weak<AppWindow>,
    cells: &Arc<VizCells>,
    fft_thread_drain: &std::thread::Thread,
    bars: &Rc<VecModel<f32>>,
    spectral: &Rc<VecModel<f32>>,
    energy: &Rc<VecModel<f32>>,
    waveform: &Rc<VecModel<f32>>,
) {
    // Paused gate: while NowPlayingState says not-playing, skip the whole
    // drain (cell takes, set_row_data models, shader frame) — the producer is
    // parked via the tap's `paused` flag (playback.rs mirrors every
    // set_playing flip), so there is no fresh data; the bars simply freeze at
    // their last values. On the paused→playing edge, unpark the producer so
    // resume feels instant.
    let Some(win) = weak.upgrade() else {
        return;
    };
    let playing = win.global::<NowPlayingState>().get_playing();
    if playing && !state.drain_saw_playing {
        fft_thread_drain.unpark();
    }
    state.drain_saw_playing = playing;
    if !playing {
        return;
    }
    if let Some(b) = cells.bars.lock().unwrap().take() {
        for (i, v) in b.iter().enumerate() {
            bars.set_row_data(i, *v);
        }
        state.last_bars16 = b;
    }
    if let Some(b) = cells.energy.lock().unwrap().take() {
        for (i, v) in b.iter().enumerate() {
            energy.set_row_data(i, *v);
        }
        state.last_energy = b;
    }
    // Capture the latest spectral frame for the spectral-ribbon shader
    // (mode 4): a Some here = a new column to paint this tick.
    let mut new_spectral: Option<Vec<f32>> = None;
    if let Some(b) = cells.spectral.lock().unwrap().take() {
        for (i, v) in b.iter().enumerate() {
            spectral.set_row_data(i, *v);
        }
        new_spectral = Some(b);
    }
    if let Some(b) = cells.waveform.lock().unwrap().take() {
        for (i, v) in b.iter().enumerate() {
            waveform.set_row_data(i, *v);
        }
    }
    if let Some(x) = cells.transient.lock().unwrap().take() {
        if let Some(w) = weak.upgrade() {
            w.global::<VisualizerState>().set_transient(x);
        }
        state.last_tr = x.max(state.last_tr);
        state.last_beat = x.max(state.last_beat);
    }

    shader::render(weak, state, &mut new_spectral);
}
