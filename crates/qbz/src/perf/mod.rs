//! Settings > Performance: the live frame figures behind the panel.
//!
//! WHY IT LIVES IN THE RENDERING NOTIFIER: Slint has no "current fps"
//! anywhere — the only per-frame hook is the window's rendering notifier,
//! and a window may have exactly ONE. `shader_underlay` already owns it, so
//! this module does not register a second: main.rs calls `tick()` from the
//! existing closure's `BeforeRendering` arm. That closure is only installed
//! for the wgpu renderer, which is why the panel shows dashes on the
//! software/femtovg tiers instead of a fabricated zero — see `wire`'s
//! `measuring` argument.
//!
//! WHY A 1 s PUSH RATHER THAN A PROPERTY WRITE PER FRAME: writing a Slint
//! property from inside the notifier marks the UI dirty, which schedules the
//! next frame, which writes the property again. A performance panel that
//! makes the app render continuously is a measurement that changes what it
//! measures. Frames are counted into a plain struct; a timer pushes one
//! sample per second.

mod meter;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::time::{Duration, Instant};

use slint::ComponentHandle;

use crate::{AppWindow, PerfState};
use meter::{FrameMeter, Sample};

const WINDOW: Duration = Duration::from_secs(1);

thread_local! {
    static METER: RefCell<FrameMeter> = RefCell::new(FrameMeter::new(WINDOW));
    /// The most recent closed window, waiting to be pushed to the UI.
    static LATEST: RefCell<Option<Sample>> = const { RefCell::new(None) };
    /// A dropped `Timer` stops firing — this keeps it alive for the session.
    static PUSH_TIMER: RefCell<Option<slint::Timer>> = const { RefCell::new(None) };
}

/// One rendered frame. Called from the rendering notifier on the UI thread;
/// must stay allocation-free and cheap, it runs before every paint.
pub(crate) fn tick() {
    let now = Instant::now();
    METER.with(|m| {
        if let Some(sample) = m.borrow_mut().record(now) {
            LATEST.with(|l| *l.borrow_mut() = Some(sample));
        }
    });
}

/// Fill the renderer rows once and start the 1 s push. `measuring` is false
/// when no frame source exists (no notifier on this renderer tier).
pub(crate) fn wire(window: &AppWindow, measuring: bool) {
    let (renderer, adapters) = crate::renderer_decision_summary();
    let state = window.global::<PerfState>();
    state.set_renderer(renderer.into());
    state.set_adapters(adapters.into());
    state.set_measuring(measuring);
    if !measuring {
        return;
    }

    let weak = window.as_weak();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, WINDOW, move || {
        let Some(window) = weak.upgrade() else { return };
        // take(): an idle window renders nothing, so no window closes and no
        // sample arrives. Reporting the last busy second forever would read
        // as "60 fps" on a still app; zeros are the truth.
        let sample = LATEST.with(|l| l.borrow_mut().take());
        let state = window.global::<PerfState>();
        match sample {
            Some(s) => {
                state.set_fps(s.fps);
                state.set_frame_ms(s.frame_ms);
                state.set_worst_ms(s.worst_ms);
            }
            None => {
                state.set_fps(0.0);
                state.set_frame_ms(0.0);
                state.set_worst_ms(0.0);
            }
        }
    });
    PUSH_TIMER.with(|t| *t.borrow_mut() = Some(timer));
}
