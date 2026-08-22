use std::sync::Arc;
use std::time::{Duration, Instant};

use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};

mod bands;
mod process;
#[cfg(test)]
mod tests;

const MIN_FREQ_HZ: f32 = 20.0;
const MAX_FREQ_HZ: f32 = 20_000.0;

/// Progressive spectral analyzer for the immersive Spectral Ribbon visualizer.
///
/// Design choices:
/// - FFT size defaults to 1024: better low-frequency detail than 512, while still
///   remaining lightweight for a 20-30Hz UI update cadence.
/// - No allocation in hot path: all buffers are pre-allocated.
/// - Output is compact and normalized (Vec<f32> bands), suitable for Tauri events.
pub struct SpectralAnalyzer {
    pub update_rate_hz: u32,
    pub fft_size: usize,
    pub smoothing_factor: f32,

    sample_rate_hz: u32,
    num_bands: usize,
    frame_interval: Duration,
    last_update: Instant,

    window: Vec<f32>,
    fft: Arc<dyn Fft<f32>>,
    fft_input: Vec<Complex32>,
    magnitudes: Vec<f32>,
    band_bin_ranges: Vec<(usize, usize)>,
    bands_raw: Vec<f32>,
    bands_smoothed: Vec<f32>,
    latest_bands: Vec<f32>,
}

impl SpectralAnalyzer {
    pub fn new(
        sample_rate_hz: u32,
        fft_size: usize,
        num_bands: usize,
        update_rate_hz: u32,
        smoothing_factor: f32,
    ) -> Self {
        let clamped_fft = match fft_size {
            512 | 1024 | 2048 | 4096 | 8192 => fft_size,
            _ => 1024,
        };
        let clamped_bands = num_bands.clamp(48, 1024);
        let clamped_rate = update_rate_hz.clamp(20, 60);
        let clamped_smoothing = smoothing_factor.clamp(0.0, 0.98);

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(clamped_fft);

        let mut analyzer = Self {
            update_rate_hz: clamped_rate,
            fft_size: clamped_fft,
            smoothing_factor: clamped_smoothing,
            sample_rate_hz,
            num_bands: clamped_bands,
            frame_interval: Duration::from_secs_f32(1.0 / clamped_rate as f32),
            last_update: Instant::now() - Duration::from_secs(1),
            window: vec![0.0; clamped_fft],
            fft,
            fft_input: vec![Complex32::default(); clamped_fft],
            magnitudes: vec![0.0; clamped_fft / 2],
            band_bin_ranges: vec![(0, 0); clamped_bands],
            bands_raw: vec![0.0; clamped_bands],
            bands_smoothed: vec![0.0; clamped_bands],
            latest_bands: vec![0.0; clamped_bands],
        };

        analyzer.rebuild_window();
        analyzer.rebuild_band_ranges(sample_rate_hz);
        analyzer
    }

    pub fn get_latest_bands(&self) -> &[f32] {
        &self.latest_bands
    }
}
