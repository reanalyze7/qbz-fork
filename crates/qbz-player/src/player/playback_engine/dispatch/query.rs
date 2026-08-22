//! Read-only queries for [`PlaybackEngine`]: emptiness, gapless-transition
//! flag, position/duration, and backend-kind checks.

use super::super::PlaybackEngine;
use std::sync::atomic::Ordering;

impl PlaybackEngine {
    /// True when this engine is the DoP (DSD over PCM) writer.
    pub fn is_dop(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            matches!(self, Self::AlsaDop { .. })
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// True only for the Rodio engine — the only variant with a `Mixer`
    /// handle to connect a second overlapping player to. Callers should
    /// check this before calling `crossfade_to` and fall back to `append`
    /// (strict gapless) on ALSA Direct/JACK/DoP.
    pub fn supports_crossfade(&self) -> bool {
        matches!(self, Self::Rodio { .. })
    }

    /// Check if playback queue is empty (all sources consumed, not playing)
    pub fn empty(&self) -> bool {
        match self {
            Self::Rodio { sink, .. } => sink.empty(),
            Self::AlsaDirect {
                is_playing,
                source_queue,
                ..
            } => !is_playing.load(Ordering::SeqCst) && source_queue.is_empty(),
            #[cfg(target_os = "linux")]
            Self::Jack {
                is_playing,
                source_queue,
                ..
            } => !is_playing.load(Ordering::SeqCst) && source_queue.is_empty(),
            #[cfg(target_os = "linux")]
            Self::AlsaDop {
                is_playing,
                source_queue,
                ..
            } => !is_playing.load(Ordering::SeqCst) && source_queue.is_empty(),
        }
    }

    /// Check if a gapless source transition just happened.
    /// Returns true once, then resets the flag.
    pub fn take_source_transition(&self) -> bool {
        match self {
            Self::Rodio { .. } => false,
            Self::AlsaDirect {
                source_transition, ..
            } => source_transition
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok(),
            #[cfg(target_os = "linux")]
            Self::Jack {
                source_transition, ..
            } => source_transition
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok(),
            #[cfg(target_os = "linux")]
            Self::AlsaDop {
                source_transition, ..
            } => source_transition
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok(),
        }
    }

}
