use rustfft::num_complex::Complex32;
use std::time::Instant;

use super::SpectralAnalyzer;

impl SpectralAnalyzer {
    /// Process a mono frame and update spectral bands if cadence allows it.
    ///
    /// Returns true if `latest_bands` was refreshed on this call.
    pub fn process_audio_frame(&mut self, mono_samples: &[f32], sample_rate_hz: u32) -> bool {
        if mono_samples.len() < self.fft_size {
            return false;
        }

        let now = Instant::now();
        if now.duration_since(self.last_update) < self.frame_interval {
            return false;
        }
        self.last_update = now;

        if sample_rate_hz != self.sample_rate_hz {
            self.sample_rate_hz = sample_rate_hz;
            self.rebuild_band_ranges(sample_rate_hz);
        }

        for (i, sample) in mono_samples.iter().take(self.fft_size).enumerate() {
            self.fft_input[i] = Complex32::new(*sample * self.window[i], 0.0);
        }
        self.fft.process(&mut self.fft_input);

        // Magnitudes for Nyquist half, normalized by FFT size.
        let fft_norm = 1.0 / self.fft_size as f32;
        for (i, value) in self.fft_input.iter().take(self.fft_size / 2).enumerate() {
            self.magnitudes[i] = value.norm() * fft_norm;
        }

        for band_idx in 0..self.num_bands {
            let (start_bin, end_bin) = self.band_bin_ranges[band_idx];
            if end_bin <= start_bin {
                self.bands_raw[band_idx] = 0.0;
                continue;
            }

            let mut sum = 0.0f32;
            let mut count = 0u32;
            for bin in start_bin..end_bin {
                let m = self.magnitudes[bin];
                sum += m * m;
                count += 1;
            }

            // RMS energy with linear scaling — no power compression so the
            // frontend's exponential transform has full dynamic range to work with.
            let rms = if count > 0 {
                (sum / count as f32).sqrt()
            } else {
                0.0
            };
            let scaled = (rms * 25.0).clamp(0.0, 1.0);
            self.bands_raw[band_idx] = scaled;
        }

        // Exponential smoothing with fast attack / slower release.
        for i in 0..self.num_bands {
            let new_value = self.bands_raw[i];
            let prev = self.bands_smoothed[i];
            let alpha = if new_value > prev {
                1.0 - self.smoothing_factor * 0.5
            } else {
                1.0 - self.smoothing_factor
            };
            let smoothed = prev + alpha * (new_value - prev);
            self.bands_smoothed[i] = smoothed;
            self.latest_bands[i] = smoothed.clamp(0.0, 1.0);
        }

        true
    }
}
