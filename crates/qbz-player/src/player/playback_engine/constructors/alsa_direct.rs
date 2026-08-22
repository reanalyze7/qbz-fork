//! `PlaybackEngine::new_alsa_direct`.

use super::super::alsa_writer::alsa_writer_thread;
use super::super::{PlaybackEngine, SourceQueue};
use qbz_audio::AlsaDirectStream;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::thread;

impl PlaybackEngine {
    /// Create ALSA Direct engine with gapless source queue.
    /// Spawns a single writer thread that lives for the engine's lifetime.
    pub fn new_alsa_direct(stream: Arc<AlsaDirectStream>, hardware_volume: bool) -> Self {
        let is_playing = Arc::new(AtomicBool::new(false));
        let should_stop = Arc::new(AtomicBool::new(false));
        let position_frames = Arc::new(AtomicU64::new(0));
        let duration_frames = Arc::new(AtomicU64::new(0));
        let source_queue = Arc::new(SourceQueue::new());
        let source_transition = Arc::new(AtomicBool::new(false));

        // Spawn the single long-lived writer thread
        let handle = {
            let stream_c = stream.clone();
            let playing_c = is_playing.clone();
            let stop_c = should_stop.clone();
            let pos_c = position_frames.clone();
            let dur_c = duration_frames.clone();
            let queue_c = source_queue.clone();
            let transition_c = source_transition.clone();
            let channels = stream.channels();

            thread::spawn(move || {
                alsa_writer_thread(
                    stream_c,
                    playing_c,
                    stop_c,
                    pos_c,
                    dur_c,
                    queue_c,
                    transition_c,
                    channels,
                );
            })
        };

        Self::AlsaDirect {
            stream,
            is_playing,
            should_stop,
            position_frames,
            duration_frames,
            source_queue,
            playback_thread: Some(handle),
            source_transition,
            hardware_volume,
        }
    }
}
