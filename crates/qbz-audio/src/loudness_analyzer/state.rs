use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use ebur128::{EbuR128, Mode};

pub(super) struct AnalyzerState {
    pub(super) track_id: u64,
    pub(super) target_lufs: f32,
    pub(super) ebur128: EbuR128,
    pub(super) channels: u16,
    pub(super) sample_rate: u32,
    /// Shared gain atomic — written by us, read by DynamicAmplify
    pub(super) gain_atomic: Option<Arc<AtomicU32>>,
    /// Total samples fed since last reset
    pub(super) samples_fed: u64,
    /// Total samples fed at last measurement
    pub(super) samples_at_last_measure: u64,
    /// Whether initial measurement has been done
    pub(super) initial_done: bool,
    /// Dynamic thresholds based on actual sample rate and channels
    pub(super) initial_threshold: u64,
    pub(super) refinement_interval: u64,
}

impl AnalyzerState {
    pub(super) fn new(track_id: u64, sample_rate: u32, channels: u16, target_lufs: f32) -> Self {
        let ebur128 = EbuR128::new(channels as u32, sample_rate, Mode::I)
            .expect("Failed to create EbuR128 instance");

        // Scale thresholds to actual sample rate and channel count
        let samples_per_second = sample_rate as u64 * channels as u64;
        let initial_threshold = samples_per_second * 10; // 10 seconds
        let refinement_interval = samples_per_second * 5; // 5 seconds

        Self {
            track_id,
            target_lufs,
            ebur128,
            channels,
            sample_rate,
            gain_atomic: None,
            samples_fed: 0,
            samples_at_last_measure: 0,
            initial_done: false,
            initial_threshold,
            refinement_interval,
        }
    }

    /// Reset the EBU R128 analyzer (e.g., after seek) but keep the gain atomic.
    pub(super) fn reset_analyzer(&mut self) {
        self.ebur128 = EbuR128::new(self.channels as u32, self.sample_rate, Mode::I)
            .expect("Failed to create EbuR128 instance");
        self.samples_fed = 0;
        self.samples_at_last_measure = 0;
        self.initial_done = false;
    }
}
