//! `install()`: wires the FFT producer thread, the persistent Slint models,
//! and the ~30fps drain timer, then registers the `set-enabled` handler.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_audio::visualizer::spawn_visualizer_thread;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::adapter::SlintAdapter;
use crate::{AppWindow, VisualizerState};

use super::cells::{SlintVizSink, VizCells};
use super::drain::{self, DrainState};

thread_local! {
    /// Keeps the drain timer alive for the app lifetime (a dropped `Timer` stops
    /// firing) and reachable from the set-enabled handler, which restarts/stops
    /// it with the tap. Lives on the UI thread, like the models it writes.
    static DRAIN_TIMER: RefCell<Option<slint::Timer>> = const { RefCell::new(None) };
}

/// Wire the visualizer. Call once, on the UI thread, after the runtime is built
/// and before `window.run()`. No-op when the runtime carries no tap (i.e. it was
/// built without [`AppRuntime::with_visualizer`]).
pub fn install(window: &AppWindow, runtime: &Arc<AppRuntime<SlintAdapter>>) {
    let Some(tap) = runtime.visualizer_tap().cloned() else {
        log::warn!("[viz] runtime has no visualizer tap; immersive visualizers disabled");
        return;
    };

    // Persistent models — created once, set on the global once, then mutated per
    // frame so the bound views never see a new model identity.
    let bars: Rc<VecModel<f32>> = Rc::new(VecModel::from(vec![0.0f32; 16]));
    let spectral: Rc<VecModel<f32>> = Rc::new(VecModel::from(vec![0.0f32; 512]));
    let energy: Rc<VecModel<f32>> = Rc::new(VecModel::from(vec![0.0f32; 5]));
    let waveform: Rc<VecModel<f32>> = Rc::new(VecModel::from(vec![0.0f32; 512]));

    let st = window.global::<VisualizerState>();
    st.set_bars(ModelRc::from(bars.clone()));
    st.set_spectral(ModelRc::from(spectral.clone()));
    st.set_energy(ModelRc::from(energy.clone()));
    st.set_waveform(ModelRc::from(waveform.clone()));

    // Producer thread: computes the five streams, latches each into its cell.
    // Keep its `Thread` handle so the set-enabled handler (registered below,
    // after the drain timer exists) can unpark it out of its disabled idle.
    let cells = Arc::new(VizCells::default());
    let sink = Arc::new(SlintVizSink {
        cells: cells.clone(),
    });
    let fft_thread = spawn_visualizer_thread(tap.clone(), sink).thread().clone();

    // ~30 fps drain on the UI thread: copy the latest frames into the models.
    let weak = window.as_weak();
    let timer = slint::Timer::default();
    let mut state = DrainState::default();
    // Second handle to the producer thread for the resume unpark below (the
    // original moves into the set-enabled handler).
    let fft_thread_drain = fft_thread.clone();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(33),
        move || {
            drain::tick(
                &mut state,
                &weak,
                &cells,
                &fft_thread_drain,
                &bars,
                &spectral,
                &energy,
                &waveform,
            );
        },
    );
    // The drain only needs to run while the tap captures (it used to tick for
    // the whole app lifetime doing lock/None-takes). Register the callback via
    // start(), then park it stopped; the set-enabled handler below restarts /
    // stops it together with the tap. All on the UI thread.
    timer.stop();
    DRAIN_TIMER.with(|t| *t.borrow_mut() = Some(timer));

    // set-enabled → toggle capture on the tap, wake the parked FFT producer,
    // and start/stop the UI drain timer. Registered AFTER the timer is stored
    // so any invoke — including the initial seed in main.rs, which runs right
    // after install() — always finds it.
    {
        let tap = tap.clone();
        st.on_set_enabled(move |on| {
            tap.set_enabled(on);
            if on {
                // The producer parks (park_timeout) while disabled; unpark for
                // an instant wake instead of waiting out its idle poll.
                fft_thread.unpark();
            }
            DRAIN_TIMER.with(|t| {
                if let Some(timer) = t.borrow().as_ref() {
                    if on {
                        timer.restart();
                    } else {
                        timer.stop();
                    }
                }
            });
        });
    }
    log::info!("[viz] producer + 30fps drain installed (idle until the tap enables)");
}
