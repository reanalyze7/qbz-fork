//! Pure line-bed (mode 5) DSP chain: 512 backend bands -> 256 line heights,
//! plus the depth ring that stacks them into the line-bed lattice.

use std::cell::RefCell;

use super::{LINEBED_BANDS, LINEBED_LINES, SPECTRO_BANDS};

/// Line-bed (mode 5) reshaping + depth ring. Its own thread-local so
/// `render_frame` can mutate it independently of the immutable `STATE` borrow.
pub(super) struct LineBedState {
    smoothed: Vec<f32>,      // 512-band receive-IIR accumulator
    pub(super) ring: Vec<f32>, // LINEBED_LINES*LINEBED_BANDS, depth-ordered (row 0 = newest)
}
impl LineBedState {
    fn new() -> Self {
        Self {
            smoothed: vec![0.0; SPECTRO_BANDS as usize],
            ring: vec![0.0; (LINEBED_LINES * LINEBED_BANDS) as usize],
        }
    }
    /// Receive-IIR a 512-band frame, reshape to 256 heights, push at the near row.
    pub(super) fn push(&mut self, bins: &[f32]) {
        let n = self.smoothed.len().min(bins.len());
        for i in 0..n {
            self.smoothed[i] = self.smoothed[i] * 0.03 + bins[i] * 0.97;
        }
        let row = reshape_512_to_256(&self.smoothed);
        let bands = LINEBED_BANDS as usize;
        let lines = LINEBED_LINES as usize;
        // Shift every row one slot deeper, then write the newest at row 0.
        self.ring.copy_within(0..(lines - 1) * bands, bands);
        self.ring[0..bands].copy_from_slice(&row);
    }
}
thread_local! {
    pub(super) static LINEBED: RefCell<LineBedState> = RefCell::new(LineBedState::new());
}
thread_local! {
    /// Last spectrogram column written (spectral-ribbon gap-fill).
    pub(super) static SPECTRO_LAST_COL: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// Reused upload scratch for the spectral-ribbon column writes (mode 4):
    /// (one quantized 512-band row, the gap-fill repetition of it). Avoids two
    /// Vec allocations per spectral frame.
    pub(super) static SPECTRO_SCRATCH: RefCell<(Vec<u8>, Vec<u8>)> =
        const { RefCell::new((Vec::new(), Vec::new())) };
}

/// 512 backend bands → 256 line heights in [0.1, 84] — Tauri's LinebedPanel
/// chain (the backend bands are intentionally flat; this is what makes the
/// ridges): frequency-warp bin map → peak-preserving smoothing → low-end tail
/// roll-off → 3-point box → per-band gamma + soft clip.
fn reshape_512_to_256(data: &[f32]) -> [f32; 256] {
    let mut vis = [0.0f32; 256];
    for i in 0..256 {
        let seg_start = (i as f32 / 256.0).powf(1.32);
        let seg_end = ((i + 1) as f32 / 256.0).powf(1.32);
        let s = 4.0 + (460.0 - 4.0) * seg_start;
        let e = 4.0 + (460.0 - 4.0) * seg_end;
        let lower = (s.floor() as usize).max(4);
        let upper = (e.ceil() as usize).min(460);
        let (mut sum, mut peak, mut cnt) = (0.0f32, 0.0f32, 0u32);
        let mut j = lower;
        while j <= upper && j < data.len() {
            sum += data[j];
            if data[j] > peak {
                peak = data[j];
            }
            cnt += 1;
            j += 1;
        }
        let avg = if cnt > 0 { sum / cnt as f32 } else { 0.0 };
        vis[i] = (avg * 0.52 + peak * 0.48) * 770.0;
    }
    apply_average(&mut vis);
    // Low-end tail roll-off (first 7 bins).
    for i in 0..7usize {
        vis[i] *= 0.013_334_120_966_221_101 * ((i + 1) as f32).powf(1.6) + 0.7;
    }
    smooth3(&mut vis);
    // Per-band gamma + soft clip + cap → [0.1, 84].
    for i in 0..256 {
        let frac = i as f32 / 255.0;
        let exp = 1.35 + (0.9 - 1.35) * frac * frac;
        let norm = (vis[i] / 770.0).max(0.0);
        let shaped = norm.powf(exp);
        let comp = 1.0 - (-shaped * 3.25).exp();
        vis[i] = (comp * 84.0).clamp(0.1, 84.0);
    }
    vis
}

/// Two-pass peak-preserving smoothing (Tauri applyAverageTransform).
fn apply_average(d: &mut [f32; 256]) {
    let src = *d;
    for i in 0..256 {
        let prev = if i > 0 { src[i - 1] } else { src[i] };
        let next = if i < 255 { src[i + 1] } else { src[i] };
        let cur = src[i];
        d[i] = if cur >= prev && cur >= next {
            cur
        } else {
            (cur + prev.max(next)) / 2.0
        };
    }
    let src2 = *d;
    for i in 0..256 {
        let prev = if i > 0 { src2[i - 1] } else { src2[i] };
        let next = if i < 255 { src2[i + 1] } else { src2[i] };
        let cur = src2[i];
        d[i] = if cur >= prev && cur >= next {
            cur
        } else {
            cur / 2.0 + prev.max(next) / 3.0 + prev.min(next) / 6.0
        };
    }
}

/// 3-point box smooth, one pass (Tauri smoothSpectrum).
fn smooth3(d: &mut [f32; 256]) {
    let src = *d;
    for i in 0..256 {
        let prev = if i > 0 { src[i - 1] } else { src[i] };
        let next = if i < 255 { src[i + 1] } else { src[i] };
        d[i] = (prev + src[i] + next) / 3.0;
    }
}
