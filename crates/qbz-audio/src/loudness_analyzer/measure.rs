use std::sync::atomic::Ordering;

use super::{compute_gain_capped, AnalyzerState, LoudnessCache};

impl AnalyzerState {
    /// Feed samples to the EBU R128 analyzer and possibly update gain.
    pub(super) fn feed_samples(&mut self, samples: &[f32], cache: &LoudnessCache) {
        // Feed interleaved samples as frames
        let frame_count = samples.len() / self.channels as usize;
        if frame_count == 0 {
            return;
        }

        if let Err(e) = self.ebur128.add_frames_f32(samples) {
            log::warn!("[LoudnessAnalyzer] Error feeding samples: {}", e);
            return;
        }

        self.samples_fed += samples.len() as u64;

        // Check if it's time to measure
        let should_measure = if !self.initial_done {
            self.samples_fed >= self.initial_threshold
        } else {
            self.samples_fed - self.samples_at_last_measure >= self.refinement_interval
        };

        if should_measure {
            self.measure_and_update(cache);
        }
    }

    pub(super) fn measure_and_update(&mut self, cache: &LoudnessCache) {
        let loudness = match self.ebur128.loudness_global() {
            Ok(l) => l,
            Err(e) => {
                log::warn!("[LoudnessAnalyzer] Failed to get loudness: {}", e);
                return;
            }
        };

        // -inf means silence — don't adjust
        if loudness.is_infinite() || loudness.is_nan() {
            log::debug!(
                "[LoudnessAnalyzer] Track {}: loudness is {:?}, skipping",
                self.track_id,
                loudness
            );
            return;
        }

        let measured_lufs = loudness as f32;
        let adjustment_db = self.target_lufs - measured_lufs;
        let gain = compute_gain_capped(adjustment_db);

        let phase = if self.initial_done {
            "refine"
        } else {
            "initial"
        };
        log::info!(
            "[LoudnessAnalyzer] Track {} ({}): measured {:.1} LUFS, target {:.1}, adjustment {:.2} dB, gain {:.4}",
            self.track_id, phase, measured_lufs, self.target_lufs, adjustment_db, gain
        );

        // Only update the live gain on the FIRST measurement.
        // Refinements update the cache only — applying gain changes mid-song
        // causes audible volume fluctuations within a single track.
        if !self.initial_done {
            if let Some(ref atomic) = self.gain_atomic {
                atomic.store(gain.to_bits(), Ordering::Relaxed);
            }
        }

        self.samples_at_last_measure = self.samples_fed;
        self.initial_done = true;

        // Always cache the latest measurement for next playback
        cache.set(self.track_id, adjustment_db, 0.0, "ebur128");
    }
}
