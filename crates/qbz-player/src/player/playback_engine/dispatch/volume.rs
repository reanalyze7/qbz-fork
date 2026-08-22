//! `set_volume` for [`PlaybackEngine`].

use super::super::PlaybackEngine;

impl PlaybackEngine {
    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        match self {
            Self::Rodio { sink, .. } => sink.set_volume(volume),
            Self::AlsaDirect {
                stream,
                hardware_volume,
                ..
            } => {
                if *hardware_volume {
                    #[cfg(target_os = "linux")]
                    {
                        if let Err(e) = stream.set_hardware_volume(volume) {
                            log::warn!("[ALSA Direct Engine] Hardware volume failed: {}", e);
                        }
                    }
                } else {
                    log::debug!(
                        "[ALSA Direct Engine] Hardware volume control disabled (use DAC/amplifier)"
                    );
                }
            }
            #[cfg(target_os = "linux")]
            Self::Jack { .. } => {
                // JACK output volume is controlled in the JACK graph / DAW; the
                // feeder writes unattenuated f32. (Software volume could later be
                // applied by scaling in the feeder.)
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop { .. } => {
                // ANY gain applied to DoP words breaks the marker sequence —
                // volume must be controlled at the DAC/amplifier.
                log::debug!("[DoP Engine] Volume is fixed during DoP playback");
            }
        }
    }
}
