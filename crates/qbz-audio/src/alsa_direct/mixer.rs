//! `AlsaDirectStream::set_hardware_volume`.

use super::AlsaDirectStream;

impl AlsaDirectStream {
    /// Try to set hardware volume via ALSA mixer
    ///
    /// Returns error if:
    /// - DAC doesn't have mixer controls (common for USB DACs)
    /// - Mixer API fails
    ///
    /// NOTE: Failure doesn't break playback, just means volume can't be controlled.
    pub fn set_hardware_volume(&self, volume: f32) -> Result<(), String> {
        use alsa::mixer::SelemChannelId::*;
        use alsa::mixer::{Mixer, SelemId};

        // Open mixer for device
        let mixer = Mixer::new(&self.device_id, false)
            .map_err(|e| format!("Failed to open mixer for {}: {}", self.device_id, e))?;

        // Try to find a volume control element
        // Common names: "Master", "PCM", "Speaker", "Headphone"
        let control_names = ["Master", "PCM", "Speaker", "Headphone", "Digital"];

        for name in &control_names {
            let selem_id = SelemId::new(name, 0);

            if let Some(selem) = mixer.find_selem(&selem_id) {
                // Check if this element has playback volume control
                if selem.has_playback_volume() {
                    let (min, max) = selem.get_playback_volume_range();
                    let target = min + ((max - min) as f32 * volume) as i64;

                    log::info!(
                        "[ALSA Direct] Setting hardware volume via '{}': {:.0}% (raw: {}/{})",
                        name,
                        volume * 100.0,
                        target,
                        max
                    );

                    // Set volume on all channels
                    for channel in &[FrontLeft, FrontRight, FrontCenter, RearLeft, RearRight] {
                        let _ = selem.set_playback_volume(*channel, target);
                    }

                    return Ok(());
                }
            }
        }

        Err(format!(
            "No volume control found for {}. DAC may not support hardware mixer.",
            self.device_id
        ))
    }
}
