//! Shared frame store between the FFT producer thread and the UI drain.

use std::sync::{Arc, Mutex};

use qbz_audio::visualizer::{VizFrame, VizSink};

/// Single-slot, latest-wins frame store shared with the FFT producer thread.
/// Each cell holds at most the most recent frame for that stream; the UI drain
/// `take()`s it. A stalled UI therefore drops intermediate frames instead of
/// growing an unbounded queue.
#[derive(Default)]
pub(super) struct VizCells {
    pub(super) bars: Mutex<Option<[f32; 16]>>,
    pub(super) spectral: Mutex<Option<Vec<f32>>>,
    pub(super) energy: Mutex<Option<[f32; 5]>>,
    pub(super) waveform: Mutex<Option<Box<[f32; 512]>>>,
    pub(super) transient: Mutex<Option<f32>>,
}

/// The producer-side sink: latches frames into the shared cells (no Slint access
/// from the FFT thread).
pub(super) struct SlintVizSink {
    pub(super) cells: Arc<VizCells>,
}

impl VizSink for SlintVizSink {
    fn submit(&self, frame: VizFrame) {
        match frame {
            VizFrame::Viz16(b) => *self.cells.bars.lock().unwrap() = Some(b),
            VizFrame::Spectral512(b) => *self.cells.spectral.lock().unwrap() = Some(b),
            VizFrame::Energy5(b) => *self.cells.energy.lock().unwrap() = Some(b),
            VizFrame::Wave256x2(b) => *self.cells.waveform.lock().unwrap() = Some(b),
            VizFrame::Transient1(x) => *self.cells.transient.lock().unwrap() = Some(x),
        }
    }
}
