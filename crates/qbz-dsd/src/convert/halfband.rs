//! Half-band ÷2 decimation stage used by the [`super::converter`] chain.

use std::sync::OnceLock;

pub(super) const HALFBAND_TAPS: usize = 63;

/// Symmetric half-band low-pass (cutoff fs/4) for ÷2 decimation, generated
/// once: windowed sinc (Blackman), odd length, even taps (except center)
/// exactly zero by construction of sinc(n/2).
pub(super) fn halfband_taps() -> &'static [f32; HALFBAND_TAPS] {
    static TAPS: OnceLock<[f32; HALFBAND_TAPS]> = OnceLock::new();
    TAPS.get_or_init(|| {
        let m = (HALFBAND_TAPS - 1) as f64 / 2.0; // 31
        let mut taps = [0.0f32; HALFBAND_TAPS];
        let mut sum = 0.0f64;
        for (n, t) in taps.iter_mut().enumerate() {
            let x = n as f64 - m;
            let sinc = if x == 0.0 {
                0.5
            } else {
                (std::f64::consts::PI * 0.5 * x).sin() / (std::f64::consts::PI * x)
            };
            let w = 0.42 - 0.5 * (2.0 * std::f64::consts::PI * n as f64 / (HALFBAND_TAPS - 1) as f64).cos()
                + 0.08 * (4.0 * std::f64::consts::PI * n as f64 / (HALFBAND_TAPS - 1) as f64).cos();
            let v = sinc * w;
            *t = v as f32;
            sum += v;
        }
        // Normalize to unity DC gain.
        let scale = (1.0 / sum) as f32;
        for t in taps.iter_mut() {
            *t *= scale;
        }
        taps
    })
}

/// Streaming FIR decimate-by-2 stage with history carry-over.
pub(super) struct HalfBand {
    /// Pending input: last TAPS-1 samples of the previous call + new input.
    carry: Vec<f32>,
    /// Read cursor parity is preserved across calls via drain bookkeeping.
    next_center: usize,
}

impl HalfBand {
    pub(super) fn new() -> Self {
        Self {
            // Prime with zeros so the first outputs are filter warm-up, not
            // garbage; keeps output counting deterministic (len/2 per input).
            carry: vec![0.0; HALFBAND_TAPS - 1],
            next_center: HALFBAND_TAPS - 1,
        }
    }

    /// Feed `input`, append decimated output to `out`.
    pub(super) fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        let taps = halfband_taps();
        self.carry.extend_from_slice(input);
        let mut i = self.next_center;
        while i < self.carry.len() {
            let window = &self.carry[i + 1 - HALFBAND_TAPS..=i];
            let mut acc = 0.0f32;
            // Half-band: even-indexed taps are zero except the center.
            let mut j = 0;
            while j < HALFBAND_TAPS {
                acc += taps[j] * window[HALFBAND_TAPS - 1 - j];
                j += 2;
            }
            acc += taps[HALFBAND_TAPS / 2] * window[HALFBAND_TAPS / 2];
            out.push(acc);
            i += 2;
        }
        // Keep the last TAPS-1 samples; remember cursor parity.
        let keep_from = self.carry.len().saturating_sub(HALFBAND_TAPS - 1);
        let overshoot = i - self.carry.len(); // 0 or 1
        self.carry.drain(..keep_from);
        self.next_center = (HALFBAND_TAPS - 1) + overshoot;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halfband_dc_gain_is_unity() {
        let mut hb = HalfBand::new();
        let mut out = Vec::new();
        hb.process(&vec![0.5f32; 8192], &mut out);
        assert_eq!(out.len(), 8192 / 2);
        let tail = &out[256..];
        for &s in tail {
            assert!((s - 0.5).abs() < 1e-3, "DC not preserved: {s}");
        }
    }

    #[test]
    fn halfband_output_count_is_half_input_across_calls() {
        let mut hb = HalfBand::new();
        let mut out = Vec::new();
        // Odd-sized chunks exercise the parity carry-over.
        for chunk in [333usize, 1000, 77, 4096, 1] {
            hb.process(&vec![0.1f32; chunk], &mut out);
        }
        let total: usize = 333 + 1000 + 77 + 4096 + 1;
        // Centers sit on fixed parity from the zero prefix → ceil(total/2).
        assert_eq!(out.len(), total.div_ceil(2));
    }
}
