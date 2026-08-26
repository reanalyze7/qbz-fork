use std::sync::atomic::Ordering;

use super::{AnalyzerState, LoudnessCache};
use crate::loudness::gain::{gain_db_for, gain_factor_for};

impl AnalyzerState {
    /// Nourrit le meter et, le cas echeant, pose le gain ou met a jour le cache.
    pub(super) fn feed_samples(&mut self, samples: &[f32], cache: &LoudnessCache) {
        if samples.len() < self.channels as usize {
            return;
        }
        if !self.meter.feed(samples) {
            log::warn!("[LoudnessAnalyzer] Echantillons refuses par le meter");
            return;
        }

        let frames = self.meter.frames_fed();

        // Gain provisoire : seulement si rien n'a encore ete pose (ni cache,
        // ni pre-analyse hors-ligne). C'est le seul changement de volume
        // autorise en cours de lecture, et il tombe dans les 2 premieres s.
        if !self.gain_applied && frames >= self.provisional_frames {
            self.apply_provisional();
        }

        let due = if self.frames_at_last_measure == 0 {
            frames >= self.integrated_frames
        } else {
            frames - self.frames_at_last_measure >= self.refinement_frames
        };
        if due {
            self.cache_integrated(cache);
        }
    }

    /// Pose un gain approche a partir de la loudness court-terme.
    fn apply_provisional(&mut self) {
        let Some(lufs) = self.meter.shortterm_lufs() else {
            return; // intro silencieuse : on attend plutot que de deformer
        };
        let gain = gain_factor_for(lufs, self.target_lufs);
        if let Some(ref atomic) = self.gain_atomic {
            atomic.store(gain.to_bits(), Ordering::Relaxed);
        }
        self.gain_applied = true;
        log::info!(
            "[LoudnessAnalyzer] Piste {} (provisoire): {:.1} LUFS court-terme, cible {:.1}, {:.2} dB, gain {:.4}",
            self.track_id,
            lufs,
            self.target_lufs,
            gain_db_for(lufs, self.target_lufs),
            gain
        );
    }

    /// Mesure integree -> cache uniquement, jamais appliquee au morceau en
    /// cours (c'est ce saut qui s'entendait au milieu des titres).
    fn cache_integrated(&mut self, cache: &LoudnessCache) {
        let Some(lufs) = self.meter.integrated_lufs() else {
            return;
        };
        self.frames_at_last_measure = self.meter.frames_fed();
        log::info!(
            "[LoudnessAnalyzer] Piste {} (cache): {:.1} LUFS integres, {:.2} dB pour la cible {:.1}",
            self.track_id,
            lufs,
            gain_db_for(lufs, self.target_lufs),
            self.target_lufs
        );
        cache.set(self.track_id, lufs, self.meter.peak(), "ebur128");
    }
}
