//! `CpalDefaultBackend::enumerate_devices_impl` — split out from
//! `cpal_default.rs` to keep that file under the line-count limit.

use super::cpal_default::CpalDefaultBackend;
use super::types::{AudioDevice, BackendResult};
use rodio::cpal::traits::{DeviceTrait, HostTrait};

impl CpalDefaultBackend {
    pub(super) fn enumerate_devices_impl(&self) -> BackendResult<Vec<AudioDevice>> {
        let default_device = self
            .host
            .default_output_device()
            .ok_or_else(|| "No default output device found".to_string())?;

        let default_name = default_device
            .description()
            .map(|desc| desc.name().to_string())
            .unwrap_or_else(|_| "Default Output".to_string());

        let mut devices = Vec::new();
        for device in self
            .host
            .output_devices()
            .map_err(|e| format!("Failed to enumerate output devices: {}", e))?
        {
            let name = device
                .description()
                .map(|desc| desc.name().to_string())
                .unwrap_or_else(|_| "Unknown Device".to_string());
            let is_default = name == default_name;

            // On macOS, probe device capabilities via CoreAudio
            #[cfg(target_os = "macos")]
            let (supported_rates, max_rate, bus_type, is_hw) = { Self::probe_macos_device(&name) };
            #[cfg(not(target_os = "macos"))]
            let (supported_rates, max_rate, bus_type, is_hw): (
                Option<Vec<u32>>,
                Option<u32>,
                Option<String>,
                bool,
            ) = (None, None, None, false);

            devices.push(AudioDevice {
                id: name.clone(),
                name,
                description: None,
                is_default,
                max_sample_rate: max_rate,
                supported_sample_rates: supported_rates,
                device_bus: bus_type,
                is_hardware: is_hw,
            });
        }

        // CPAL's raw enumeration repeats one output across many ALSA PCM
        // plugins (all sharing a display name) and includes the `null` discard
        // sink. Collapse the duplicates and push `null` to the end so the
        // System/JACK picker shows one entry per real output. See device_filter.
        let devices = crate::device_filter::retain_real_outputs(
            devices,
            |d| d.id.as_str(),
            |d| d.name.as_str(),
        );

        Ok(devices)
    }
}
