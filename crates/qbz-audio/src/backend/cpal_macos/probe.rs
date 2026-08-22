//! macOS device-capability probing (sample rates, bus type, hardware flag).

use super::super::cpal_default::CpalDefaultBackend;

impl CpalDefaultBackend {
    /// Probe a macOS audio device for capabilities via CoreAudio APIs.
    /// Returns (supported_rates, max_rate, bus_type, is_hardware).
    pub(in super::super) fn probe_macos_device(
        device_name: &str,
    ) -> (Option<Vec<u32>>, Option<u32>, Option<String>, bool) {
        use crate::coreaudio_direct;

        let device_id = match coreaudio_direct::find_device_by_name(device_name) {
            Ok(Some(id)) => {
                log::info!("[CoreAudio] Found device '{}' with ID {}", device_name, id);
                id
            }
            Ok(None) => {
                log::debug!(
                    "[CoreAudio] Device '{}' not found via CoreAudio",
                    device_name
                );
                return (None, None, None, false);
            }
            Err(e) => {
                log::debug!("[CoreAudio] Error finding device '{}': {}", device_name, e);
                return (None, None, None, false);
            }
        };

        let supported_rates = coreaudio_direct::query_supported_sample_rates(device_id)
            .inspect(|rates| {
                log::info!(
                    "[CoreAudio] Device '{}' supported rates: {:?}",
                    device_name,
                    rates
                )
            })
            .inspect_err(|e| {
                log::warn!(
                    "[CoreAudio] Failed to query rates for '{}': {}",
                    device_name,
                    e
                )
            })
            .ok()
            .filter(|r| !r.is_empty());
        let max_rate = supported_rates
            .as_ref()
            .and_then(|rates| rates.iter().max().copied());
        let bus_type = coreaudio_direct::get_device_transport_type(device_id);
        let is_hardware = bus_type.as_deref().is_some_and(|t| {
            t == "usb" || t == "built-in" || t == "thunderbolt" || t == "firewire"
        });

        (supported_rates, max_rate, bus_type, is_hardware)
    }
}
