//! macOS nominal-sample-rate query helpers.

use super::super::cpal_default::CpalDefaultBackend;

impl CpalDefaultBackend {
    pub(in super::super) fn current_macos_nominal_rate(
        effective_device_name: Option<&str>,
    ) -> Option<u32> {
        use crate::coreaudio_direct;

        let device_id = match effective_device_name {
            Some(name) => coreaudio_direct::find_device_by_name(name).ok().flatten(),
            None => coreaudio_direct::get_default_output_device().ok(),
        }?;

        coreaudio_direct::get_nominal_sample_rate(device_id).ok()
    }

    /// In macOS shared mode, CPAL's default config can briefly report a stale
    /// sample rate after CoreAudio changes the device's nominal rate. If we
    /// trust the stale rate, playback can run at the wrong speed until the
    /// stream is recreated. Prefer opening the CPAL stream at CoreAudio's
    /// current nominal rate when the two disagree.
    pub(in super::super) fn shared_mode_nominal_stream_config(
        device: &rodio::cpal::Device,
        effective_device_name: Option<&str>,
    ) -> Option<rodio::cpal::SupportedStreamConfig> {
        use rodio::cpal::traits::DeviceTrait;

        let nominal_rate = Self::current_macos_nominal_rate(effective_device_name)?;
        let default_config = device.default_output_config().ok()?;
        let default_rate = default_config.sample_rate();
        if nominal_rate == default_rate {
            return None;
        }

        let supported_configs: Vec<_> = device.supported_output_configs().ok()?.collect();
        let matching_config = supported_configs
            .iter()
            .find_map(|range| {
                if range.channels() == default_config.channels()
                    && range.sample_format() == default_config.sample_format()
                {
                    (*range).try_with_sample_rate(nominal_rate)
                } else {
                    None
                }
            })
            .or_else(|| {
                supported_configs
                    .iter()
                    .find_map(|range| (*range).try_with_sample_rate(nominal_rate))
            });

        let device_label = effective_device_name.unwrap_or("System Default");
        if matching_config.is_some() {
            log::warn!(
                "[CoreAudio] Shared-mode rate mismatch on '{}': CPAL default {}Hz vs CoreAudio nominal {}Hz. Opening stream at the nominal rate to avoid wrong-speed playback.",
                device_label,
                default_rate,
                nominal_rate
            );
        } else {
            log::warn!(
                "[CoreAudio] Shared-mode rate mismatch on '{}': CPAL default {}Hz vs CoreAudio nominal {}Hz, but no supported CPAL config matched the nominal rate.",
                device_label,
                default_rate,
                nominal_rate
            );
        }

        matching_config
    }
}
