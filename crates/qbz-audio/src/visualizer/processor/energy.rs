use std::sync::Arc;

use super::{VizFrame, VizSink, ENERGY_BAND_RANGES, NUM_ENERGY_BANDS};

/// RMS jump threshold for transient detection (sensitive).
const TRANSIENT_THRESHOLD: f32 = 0.04;
/// Frames remaining in transient cooldown (~100ms at 30fps).
const TRANSIENT_COOLDOWN_FRAMES: u32 = 3;

/// Per-frame mutable state for the energy-band / transient detector, carried
/// across loop iterations by `run_fft_loop`.
pub(super) struct EnergyState {
    pub(super) energy_bands: [f32; NUM_ENERGY_BANDS],
    pub(super) smoothed_energy: [f32; NUM_ENERGY_BANDS],
    pub(super) prev_rms: f32,
    pub(super) transient_cooldown: u32,
}

impl EnergyState {
    pub(super) fn new() -> Self {
        Self {
            energy_bands: [0.0; NUM_ENERGY_BANDS],
            smoothed_energy: [0.0; NUM_ENERGY_BANDS],
            prev_rms: 0.0,
            transient_cooldown: 0,
        }
    }

    /// Compute RMS per frequency band from the spectrum, submit
    /// `VizFrame::Energy5`, then feed the same per-band values into transient
    /// detection and submit `VizFrame::Transient1` on a sharp RMS jump.
    pub(super) fn process(
        &mut self,
        spectrum: &spectrum_analyzer::FrequencySpectrum,
        sink: &Arc<dyn VizSink>,
    ) {
        let data = spectrum.data();
        let mut raw_sum = 0.0f32;
        for (band_idx, &(lo, hi)) in ENERGY_BAND_RANGES.iter().enumerate() {
            let mut sum_sq = 0.0f32;
            let mut count = 0u32;
            for (freq, magnitude) in data.iter() {
                let f = freq.val();
                if f >= lo && f < hi {
                    let mag = magnitude.val();
                    sum_sq += mag * mag;
                    count += 1;
                }
            }
            let rms = if count > 0 {
                (sum_sq / count as f32).sqrt()
            } else {
                0.0
            };
            // Compress and normalize
            let compressed = (rms * 6.0).powf(0.5).clamp(0.0, 1.0);
            // Smooth: fast attack, slow decay
            if compressed > self.smoothed_energy[band_idx] {
                self.smoothed_energy[band_idx] =
                    self.smoothed_energy[band_idx] * 0.2 + compressed * 0.8;
            } else {
                self.smoothed_energy[band_idx] =
                    self.smoothed_energy[band_idx] * 0.85 + compressed * 0.15;
            }
            self.energy_bands[band_idx] = self.smoothed_energy[band_idx];
            // Transient feed: raw (pre-smoothed) value, bass/sub-bass
            // weighted 2x for beat detection.
            let weight = if band_idx < 2 { 2.0 } else { 1.0 };
            raw_sum += compressed * weight;
        }
        sink.submit(VizFrame::Energy5(self.energy_bands));

        // --- Transient Detection: detect sharp RMS jumps ---
        let raw_rms = raw_sum / (NUM_ENERGY_BANDS as f32 + 2.0); // account for extra bass weight
        let rms_delta = raw_rms - self.prev_rms;

        if self.transient_cooldown > 0 {
            self.transient_cooldown -= 1;
        }

        if rms_delta > TRANSIENT_THRESHOLD && self.transient_cooldown == 0 {
            // Transient detected! Submit intensity (0.0 - 1.0)
            let intensity = (rms_delta * 5.0).clamp(0.0, 1.0);
            sink.submit(VizFrame::Transient1(intensity));
            self.transient_cooldown = TRANSIENT_COOLDOWN_FRAMES;
        }

        self.prev_rms = raw_rms;
    }
}
