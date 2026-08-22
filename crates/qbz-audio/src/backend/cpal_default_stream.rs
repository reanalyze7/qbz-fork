//! `CpalDefaultBackend::create_output_stream_impl` — split out from
//! `cpal_default.rs` to keep that file under the line-count limit.

use super::cpal_default::CpalDefaultBackend;
use super::device_config::{BackendConfig, BackendResult};
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

impl CpalDefaultBackend {
    pub(super) fn create_output_stream_impl(
        &self,
        config: &BackendConfig,
    ) -> BackendResult<MixerDeviceSink> {
        #[cfg(target_os = "macos")]
        let macos_exclusive_device_name = if config.exclusive_mode && config.device_id.is_none() {
            match crate::coreaudio_direct::resolve_output_device_name(None) {
                Ok(name) => {
                    log::info!(
                        "[CoreAudio] Resolved System Default to '{}' for exclusive stream",
                        name
                    );
                    Some(name)
                }
                Err(e) => {
                    log::warn!(
                        "[CoreAudio] Could not resolve System Default device name: {}",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        // On macOS, only exclusive mode takes ownership of the device rate.
        // Shared mode must leave the user's current CoreAudio device rate alone.
        #[cfg(target_os = "macos")]
        {
            if config.exclusive_mode {
                if let Some(ref device_id) = config.device_id {
                    Self::switch_sample_rate_if_needed(device_id, config.sample_rate);
                } else if let Some(ref device_name) = macos_exclusive_device_name {
                    Self::switch_sample_rate_if_needed(device_name, config.sample_rate);
                } else {
                    Self::switch_default_device_rate_if_needed(config.sample_rate);
                }
            }
        }

        #[cfg(target_os = "macos")]
        let effective_device_id = config
            .device_id
            .as_ref()
            .or(macos_exclusive_device_name.as_ref());
        #[cfg(not(target_os = "macos"))]
        let effective_device_id = config.device_id.as_ref();

        #[cfg(target_os = "macos")]
        if !config.exclusive_mode {
            return self.open_macos_shared_stream_with_retry(
                effective_device_id.map(|name| name.as_str()),
                super::cpal_macos::MACOS_SHARED_OPEN_MAX_ATTEMPTS,
            );
        }

        let device = if let Some(device_id) = effective_device_id {
            self.host
                .output_devices()
                .map_err(|e| format!("Failed to enumerate devices: {}", e))?
                .find(|d| {
                    d.description()
                        .map(|desc| desc.name() == device_id.as_str())
                        .unwrap_or(false)
                })
                .ok_or_else(|| format!("Device '{}' not found", device_id))?
        } else {
            self.host
                .default_output_device()
                .ok_or_else(|| "No default output device found".to_string())?
        };

        let builder = DeviceSinkBuilder::from_device(device)
            .map_err(|e| format!("Failed to create device sink builder: {}", e))?;

        // MixerDeviceSink has zero internal buffering, so CPAL's buffer is the
        // ONLY buffer between the mixer and the hardware. With the bare CPAL/ALSA
        // default (no explicit size) the stream can underrun immediately on Linux:
        // the node links to the audio server but stays suspended and never feeds
        // audio (#470 — "System" shared output silent). Give it ~100ms, matching
        // the PipeWire/ALSA backends. We deliberately do NOT pin the sample rate
        // (no with_supported_config): the device keeps its negotiated rate and
        // rodio resamples, which is the whole point of shared "System" output.
        #[cfg(target_os = "linux")]
        let mixer_sink = builder
            .with_buffer_size(rodio::cpal::BufferSize::Fixed(
                (config.sample_rate / 10).clamp(1024, 19200),
            ))
            .open_stream()
            .map_err(|e| format!("Failed to create output stream: {}", e))?;
        #[cfg(not(target_os = "linux"))]
        let mixer_sink = builder
            .open_stream()
            .map_err(|e| format!("Failed to create output stream: {}", e))?;

        Ok(mixer_sink)
    }
}
