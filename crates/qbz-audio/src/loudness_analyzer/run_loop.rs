use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use super::{AnalyzerMessage, AnalyzerState, LoudnessAnalyzer, LoudnessCache};
use crate::loudness::gain::gain_factor_for;

impl LoudnessAnalyzer {
    pub(super) fn run(rx: Receiver<AnalyzerMessage>, cache: Arc<LoudnessCache>) {
        let mut state: Option<AnalyzerState> = None;

        loop {
            let msg = match rx.recv() {
                Ok(msg) => msg,
                Err(_) => {
                    log::info!("[LoudnessAnalyzer] Channel closed, shutting down");
                    break;
                }
            };

            match msg {
                AnalyzerMessage::NewTrack {
                    track_id,
                    sample_rate,
                    channels,
                    target_lufs,
                    gain_atomic,
                } => {
                    log::info!(
                        "[LoudnessAnalyzer] New track {} ({}Hz, {}ch, target {:.1} LUFS)",
                        track_id,
                        sample_rate,
                        channels,
                        target_lufs
                    );

                    let Some(mut s) =
                        AnalyzerState::new(track_id, sample_rate, channels, target_lufs)
                    else {
                        log::warn!(
                            "[LoudnessAnalyzer] Format {}Hz/{}ch non mesurable, piste {} laissee telle quelle",
                            sample_rate, channels, track_id
                        );
                        state = None;
                        continue;
                    };

                    // Mesure deja connue (ecoute precedente ou pre-analyse
                    // hors-ligne) : le gain est pose des la premiere note, et
                    // plus rien ne bougera pendant le morceau.
                    if let Some(cached) = cache.get(track_id) {
                        let gain = gain_factor_for(cached.measured_lufs, target_lufs);
                        gain_atomic.store(gain.to_bits(), Ordering::Relaxed);
                        s.gain_applied = true;
                        log::info!(
                            "[LoudnessAnalyzer] Cache hit piste {}: {:.1} LUFS ({}), gain {:.4}",
                            track_id,
                            cached.measured_lufs,
                            cached.source,
                            gain
                        );
                    }

                    s.gain_atomic = Some(gain_atomic);
                    state = Some(s);
                }
                AnalyzerMessage::Samples(samples) => {
                    if let Some(ref mut s) = state {
                        s.feed_samples(&samples, &cache);
                    }
                }
                AnalyzerMessage::Reset => {
                    if let Some(ref mut s) = state {
                        log::info!("[LoudnessAnalyzer] Reset (seek) — gain conserve");
                        s.reset_analyzer();
                    }
                }
                AnalyzerMessage::Shutdown => {
                    log::info!("[LoudnessAnalyzer] Shutdown requested");
                    break;
                }
            }
        }
    }
}
