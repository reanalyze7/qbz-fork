//! macOS sample-rate switching helpers, invoked before stream creation.

use super::super::cpal_default::CpalDefaultBackend;

impl CpalDefaultBackend {
    /// Switch device sample rate before stream creation (if device supports the target rate).
    pub(in super::super) fn switch_sample_rate_if_needed(device_name: &str, target_rate: u32) {
        use crate::coreaudio_direct;

        log::info!(
            "[CoreAudio] Rate switch requested: device='{}' target={}Hz",
            device_name,
            target_rate
        );

        let device_id = match coreaudio_direct::find_device_by_name(device_name) {
            Ok(Some(id)) => id,
            Ok(None) => {
                log::warn!(
                    "[CoreAudio] Cannot switch rate: device '{}' not found",
                    device_name
                );
                return;
            }
            Err(e) => {
                log::warn!("[CoreAudio] Cannot switch rate: {}", e);
                return;
            }
        };

        // Check if device supports the target rate
        if let Ok(rates) = coreaudio_direct::query_supported_sample_rates(device_id) {
            if !rates.contains(&target_rate) {
                log::debug!(
                    "[CoreAudio] Device '{}' does not support {}Hz, skipping rate switch",
                    device_name,
                    target_rate
                );
                return;
            }
        }

        if let Err(e) = coreaudio_direct::set_nominal_sample_rate(device_id, target_rate) {
            log::warn!("[CoreAudio] Failed to switch sample rate: {}", e);
        }
    }

    /// Switch the default output device's sample rate.
    pub(in super::super) fn switch_default_device_rate_if_needed(target_rate: u32) {
        use crate::coreaudio_direct;

        let device_id = match coreaudio_direct::get_default_output_device() {
            Ok(id) => id,
            Err(e) => {
                log::debug!(
                    "[CoreAudio] Could not get default device for rate switch: {}",
                    e
                );
                return;
            }
        };

        if let Ok(rates) = coreaudio_direct::query_supported_sample_rates(device_id) {
            if !rates.contains(&target_rate) {
                log::debug!(
                    "[CoreAudio] Default device does not support {}Hz, skipping rate switch",
                    target_rate
                );
                return;
            }
        }

        if let Err(e) = coreaudio_direct::set_nominal_sample_rate(device_id, target_rate) {
            log::warn!(
                "[CoreAudio] Failed to switch default device sample rate: {}",
                e
            );
        }
    }
}
