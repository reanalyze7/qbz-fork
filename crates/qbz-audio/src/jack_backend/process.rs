use jack::{AudioOut, Client, Control, Port, ProcessScope};
use ringbuf::traits::Consumer;
use ringbuf::HeapCons;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::MAX_NFRAMES;

/// JACK `process` handler (RT thread). Pops interleaved stereo f32 from the ring
/// and de-interleaves into the two output ports. Allocation- and lock-free.
pub(super) struct JackProcess {
    pub(super) consumer: HeapCons<f32>,
    pub(super) out_l: Port<AudioOut>,
    pub(super) out_r: Port<AudioOut>,
    /// Reusable interleaved scratch (pre-sized; no RT allocation).
    pub(super) scratch: Vec<f32>,
    pub(super) underruns: Arc<AtomicU64>,
}

impl jack::ProcessHandler for JackProcess {
    fn process(&mut self, _client: &Client, ps: &ProcessScope) -> Control {
        let nframes = (ps.n_frames() as usize).min(MAX_NFRAMES);
        let need = nframes * 2; // stereo interleaved
        let got = self.consumer.pop_slice(&mut self.scratch[..need]);

        let l = self.out_l.as_mut_slice(ps);
        let r = self.out_r.as_mut_slice(ps);
        for i in 0..nframes {
            let li = i * 2;
            let ri = li + 1;
            l[i] = if li < got { self.scratch[li] } else { 0.0 };
            r[i] = if ri < got { self.scratch[ri] } else { 0.0 };
        }
        if got < need {
            self.underruns.fetch_add(1, Ordering::Relaxed);
        }
        Control::Continue
    }
}
