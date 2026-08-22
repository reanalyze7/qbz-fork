//! `PlaybackEngine::new_alsa_dop`.

use super::super::dop_writer::dop_writer_thread;
use super::super::{BoxedDopIter, PlaybackEngine, SourceQueue};
use qbz_audio::AlsaDirectStream;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::thread;

impl PlaybackEngine {
    /// Create a DoP engine over an S32 ALSA direct stream created with
    /// `AlsaDirectStream::new_dop`. Sources are queued via [`Self::append_dop`].
    #[cfg(target_os = "linux")]
    pub fn new_alsa_dop(stream: Arc<AlsaDirectStream>, native: bool) -> Self {
        let is_playing = Arc::new(AtomicBool::new(false));
        let should_stop = Arc::new(AtomicBool::new(false));
        let position_frames = Arc::new(AtomicU64::new(0));
        let source_queue: Arc<SourceQueue<BoxedDopIter>> = Arc::new(SourceQueue::new());
        let source_transition = Arc::new(AtomicBool::new(false));
        let handle = {
            let stream_c = stream.clone();
            let playing_c = is_playing.clone();
            let stop_c = should_stop.clone();
            let pos_c = position_frames.clone();
            let queue_c = source_queue.clone();
            let transition_c = source_transition.clone();
            let channels = stream.channels();
            thread::spawn(move || {
                dop_writer_thread(
                    stream_c, playing_c, stop_c, pos_c, queue_c, transition_c, channels, native,
                );
            })
        };
        Self::AlsaDop {
            stream,
            is_playing,
            should_stop,
            position_frames,
            source_queue,
            writer_thread: Some(handle),
            source_transition,
        }
    }
}
