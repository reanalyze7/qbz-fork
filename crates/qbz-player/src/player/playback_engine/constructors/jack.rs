//! `PlaybackEngine::new_jack` (#263 Tier 3).

use super::super::jack_feeder::jack_feeder_thread;
use super::super::{PlaybackEngine, SourceQueue};
use qbz_audio::JackStream;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::thread;

impl PlaybackEngine {
    /// Create a JACK engine with a gapless source queue (#263 Tier 3). Spawns one
    /// long-lived feeder thread that resamples each source to the JACK graph rate
    /// and writes it to the client's ring buffer.
    #[cfg(target_os = "linux")]
    pub fn new_jack(stream: Arc<JackStream>) -> Self {
        let is_playing = Arc::new(AtomicBool::new(false));
        let should_stop = Arc::new(AtomicBool::new(false));
        let position_frames = Arc::new(AtomicU64::new(0));
        let duration_frames = Arc::new(AtomicU64::new(0));
        let source_queue = Arc::new(SourceQueue::new());
        let source_transition = Arc::new(AtomicBool::new(false));
        let graph_rate = stream.sample_rate();

        let handle = {
            let stream_c = stream.clone();
            let playing_c = is_playing.clone();
            let stop_c = should_stop.clone();
            let pos_c = position_frames.clone();
            let dur_c = duration_frames.clone();
            let queue_c = source_queue.clone();
            let transition_c = source_transition.clone();
            thread::spawn(move || {
                jack_feeder_thread(
                    stream_c, playing_c, stop_c, pos_c, dur_c, queue_c, transition_c,
                );
            })
        };

        Self::Jack {
            is_playing,
            should_stop,
            position_frames,
            duration_frames,
            source_queue,
            feeder_thread: Some(handle),
            source_transition,
            graph_rate,
        }
    }
}
