//! position_secs/duration_secs/is_alsa_direct for [`PlaybackEngine`].

use super::super::PlaybackEngine;
use std::sync::atomic::Ordering;

impl PlaybackEngine {
    /// Get current position in seconds (for ALSA Direct only)
    #[allow(dead_code)]
    pub fn position_secs(&self) -> Option<u64> {
        match self {
            Self::Rodio { .. } => None,
            Self::AlsaDirect {
                position_frames,
                stream,
                ..
            } => {
                let frames = position_frames.load(Ordering::SeqCst);
                let sample_rate = stream.sample_rate() as u64;
                Some(frames / sample_rate)
            }
            #[cfg(target_os = "linux")]
            Self::Jack {
                position_frames,
                graph_rate,
                ..
            } => {
                let frames = position_frames.load(Ordering::SeqCst);
                Some(frames / (*graph_rate as u64).max(1))
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop {
                position_frames,
                stream,
                ..
            } => {
                let frames = position_frames.load(Ordering::SeqCst);
                Some(frames / (stream.sample_rate() as u64).max(1))
            }
        }
    }

    /// Get duration in seconds (for ALSA Direct only)
    #[allow(dead_code)]
    pub fn duration_secs(&self) -> Option<u64> {
        match self {
            Self::Rodio { .. } => None,
            Self::AlsaDirect {
                duration_frames,
                stream,
                ..
            } => {
                let frames = duration_frames.load(Ordering::SeqCst);
                let sample_rate = stream.sample_rate() as u64;
                Some(frames / sample_rate)
            }
            #[cfg(target_os = "linux")]
            Self::Jack {
                duration_frames,
                graph_rate,
                ..
            } => {
                let frames = duration_frames.load(Ordering::SeqCst);
                Some(frames / (*graph_rate as u64).max(1))
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop { .. } => None,
        }
    }

    /// Check if using ALSA Direct engine
    #[allow(dead_code)]
    pub fn is_alsa_direct(&self) -> bool {
        matches!(self, Self::AlsaDirect { .. })
    }
}
