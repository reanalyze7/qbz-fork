use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use super::{compute_gain_capped, AnalyzerMessage, AnalyzerState, LoudnessAnalyzer, LoudnessCache};

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

                    // Check cache first
                    if let Some(cached) = cache.get(track_id) {
                        let gain = compute_gain_capped(cached.gain_db);
                        log::info!(
                            "[LoudnessAnalyzer] Cache hit for track {}: {:.2} dB (source: {}), gain {:.4}",
                            track_id, cached.gain_db, cached.source, gain
                        );

                        // Set gain immediately via the atomic
                        gain_atomic.store(gain.to_bits(), Ordering::Relaxed);

                        // Create state marked as cached — still accept samples for refinement
                        let mut s =
                            AnalyzerState::new(track_id, sample_rate, channels, target_lufs);
                        s.gain_atomic = Some(gain_atomic);
                        s.initial_done = true;
                        state = Some(s);
                        continue;
                    }

                    // No cache — start fresh analysis
                    let mut s = AnalyzerState::new(track_id, sample_rate, channels, target_lufs);
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
                        log::info!("[LoudnessAnalyzer] Reset (seek) — keeping current gain");
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
