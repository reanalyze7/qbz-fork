use std::f32::consts::PI;

use super::{SpectralAnalyzer, MAX_FREQ_HZ, MIN_FREQ_HZ};

impl SpectralAnalyzer {
    pub(super) fn rebuild_window(&mut self) {
        // Hann window coefficients: w[n] = 0.5 * (1 - cos(2πn/(N-1))).
        let denom = (self.fft_size - 1) as f32;
        for n in 0..self.fft_size {
            self.window[n] = 0.5 * (1.0 - (2.0 * PI * (n as f32) / denom).cos());
        }
    }

    pub(super) fn rebuild_band_ranges(&mut self, sample_rate_hz: u32) {
        let nyquist = sample_rate_hz as f32 * 0.5;
        let max_freq = MAX_FREQ_HZ.min(nyquist.max(MIN_FREQ_HZ + 1.0));
        let min_log = MIN_FREQ_HZ.ln();
        let max_log = max_freq.ln();
        let bin_hz = sample_rate_hz as f32 / self.fft_size as f32;
        let max_bin = self.magnitudes.len().saturating_sub(1);

        for band_idx in 0..self.num_bands {
            let t0 = band_idx as f32 / self.num_bands as f32;
            let t1 = (band_idx + 1) as f32 / self.num_bands as f32;

            let low_hz = (min_log + (max_log - min_log) * t0).exp();
            let high_hz = (min_log + (max_log - min_log) * t1).exp();

            let mut start_bin = (low_hz / bin_hz).floor() as usize;
            let mut end_bin = (high_hz / bin_hz).ceil() as usize;

            start_bin = start_bin.min(max_bin);
            end_bin = end_bin.min(max_bin.saturating_add(1));
            if end_bin <= start_bin {
                end_bin = (start_bin + 1).min(max_bin.saturating_add(1));
            }

            self.band_bin_ranges[band_idx] = (start_bin, end_bin);
        }
    }
}
