//! append/append_dop for [`PlaybackEngine`] — methods that put a new source
//! into the queue (see `crossfade.rs` for the overlapping-player variant,
//! which needs its own file to stay under the line-count limit but is still
//! conceptually "put a new source into the engine", not a transport op).

use super::super::{BoxedSampleIter, PlaybackEngine};
use rodio::Source;
use std::sync::atomic::Ordering;

#[cfg(target_os = "linux")]
use super::super::BoxedDopIter;

impl PlaybackEngine {
    /// Queue a DoP word source (gapless when one is already playing).
    #[cfg(target_os = "linux")]
    pub fn append_dop(&mut self, source: BoxedDopIter) -> Result<(), String> {
        match self {
            Self::AlsaDop {
                is_playing,
                should_stop,
                position_frames,
                source_queue,
                source_transition,
                ..
            } => {
                let is_first = source_queue.is_empty() && !is_playing.load(Ordering::SeqCst);
                source_queue.push(source);
                if is_first {
                    position_frames.store(0, Ordering::SeqCst);
                    should_stop.store(false, Ordering::SeqCst);
                    source_transition.store(false, Ordering::SeqCst);
                    is_playing.store(true, Ordering::SeqCst);
                    log::info!("[DoP Engine] First source queued, playback starting");
                } else {
                    log::info!("[DoP Engine] Source queued for gapless DSD transition");
                }
                Ok(())
            }
            _ => Err("append_dop on a non-DoP engine".to_string()),
        }
    }

    /// Append audio source.
    /// For ALSA Direct: pushes to the source queue for gapless transition.
    /// For Rodio: delegates to Sink's built-in queue.
    pub fn append<S>(&mut self, source: S) -> Result<(), String>
    where
        S: Source<Item = f32> + Send + 'static,
    {
        match self {
            Self::Rodio { sink, .. } => {
                sink.append(source);
                Ok(())
            }
            Self::AlsaDirect {
                is_playing,
                should_stop,
                position_frames,
                source_queue,
                source_transition,
                ..
            } => {
                let is_first = source_queue.is_empty() && !is_playing.load(Ordering::SeqCst);

                // Box the source iterator and push to queue
                let boxed: BoxedSampleIter = Box::new(source.into_iter());
                source_queue.push(boxed);

                if is_first {
                    // First source: reset position, clear stop, start playing
                    position_frames.store(0, Ordering::SeqCst);
                    should_stop.store(false, Ordering::SeqCst);
                    source_transition.store(false, Ordering::SeqCst);
                    is_playing.store(true, Ordering::SeqCst);
                    log::info!("[ALSA Direct Engine] First source queued, playback starting");
                } else {
                    log::info!("[ALSA Direct Engine] Source queued for gapless transition");
                }

                Ok(())
            }
            #[cfg(target_os = "linux")]
            Self::Jack {
                is_playing,
                should_stop,
                position_frames,
                source_queue,
                source_transition,
                graph_rate,
                ..
            } => {
                let is_first = source_queue.is_empty() && !is_playing.load(Ordering::SeqCst);
                // Resample the track-native source to the JACK graph rate (stereo) so
                // the feeder/ring always carry graph-rate interleaved stereo f32.
                let resampled = rodio::source::UniformSourceIterator::new(
                    source,
                    std::num::NonZero::new(2u16).unwrap(),
                    std::num::NonZero::new(*graph_rate).unwrap(),
                );
                let boxed: BoxedSampleIter = Box::new(resampled);
                source_queue.push(boxed);
                if is_first {
                    position_frames.store(0, Ordering::SeqCst);
                    should_stop.store(false, Ordering::SeqCst);
                    source_transition.store(false, Ordering::SeqCst);
                    is_playing.store(true, Ordering::SeqCst);
                    log::info!("[JACK Engine] First source queued, playback starting");
                } else {
                    log::info!("[JACK Engine] Source queued for gapless transition");
                }
                Ok(())
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop { .. } => {
                Err("cannot append a PCM source to a DoP engine".to_string())
            }
        }
    }
}
