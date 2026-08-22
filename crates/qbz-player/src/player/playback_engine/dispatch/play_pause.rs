//! play/pause for [`PlaybackEngine`].

use super::super::PlaybackEngine;
use std::sync::atomic::Ordering;

impl PlaybackEngine {
    /// Play (unpause)
    pub fn play(&self) {
        match self {
            Self::Rodio { sink, .. } => sink.play(),
            Self::AlsaDirect { is_playing, .. } => {
                log::info!("[ALSA Direct Engine] Resume requested");
                is_playing.store(true, Ordering::SeqCst);
            }
            #[cfg(target_os = "linux")]
            Self::Jack { is_playing, .. } => {
                log::info!("[JACK Engine] Resume requested");
                is_playing.store(true, Ordering::SeqCst);
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop { is_playing, .. } => {
                log::info!("[DoP Engine] Resume requested");
                is_playing.store(true, Ordering::SeqCst);
            }
        }
    }

    /// Pause
    pub fn pause(&self) {
        match self {
            Self::Rodio { sink, .. } => sink.pause(),
            Self::AlsaDirect { is_playing, .. } => {
                log::info!("[ALSA Direct Engine] Pause requested");
                is_playing.store(false, Ordering::SeqCst);
            }
            #[cfg(target_os = "linux")]
            Self::Jack { is_playing, .. } => {
                log::info!("[JACK Engine] Pause requested");
                is_playing.store(false, Ordering::SeqCst);
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop { is_playing, .. } => {
                // The writer keeps feeding 0x69 DSD silence while paused so
                // the DAC stays locked in DSD mode (no pop on resume).
                log::info!("[DoP Engine] Pause requested (DSD silence keeps flowing)");
                is_playing.store(false, Ordering::SeqCst);
            }
        }
    }
}
