//! `AudioBackend` trait implementation for `AlsaBackend`.

use super::output_stream_device::resolve_output_device;
use super::output_stream_rate::determine_effective_rate;
use super::output_stream_sink::build_mixer_sink;
use super::AlsaBackend;
use crate::backend::{AudioBackend, AudioBackendType, AudioDevice, BackendConfig, BackendResult};
use rodio::{
    cpal::traits::{DeviceTrait, HostTrait},
    MixerDeviceSink,
};

impl AudioBackend for AlsaBackend {
    fn backend_type(&self) -> AudioBackendType {
        AudioBackendType::Alsa
    }

    fn enumerate_devices(&self) -> BackendResult<Vec<AudioDevice>> {
        self.enumerate_with_proc_descriptions()
    }

    fn create_output_stream(&self, config: &BackendConfig) -> BackendResult<MixerDeviceSink> {
        log::info!(
            "[ALSA Backend] Creating stream: {}Hz, {} channels, exclusive: {}, plugin: {:?}",
            config.sample_rate,
            config.channels,
            config.exclusive_mode,
            config.alsa_plugin
        );

        let device = resolve_output_device(&self.host, config)?;

        let device_name = device
            .description()
            .map(|desc| desc.name().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        log::info!("[ALSA Backend] Using device: {}", device_name);

        let effective_rate = determine_effective_rate(&device, config)?;

        build_mixer_sink(device, effective_rate, config)
    }

    fn is_available(&self) -> bool {
        // Check if we can enumerate devices (ALSA is working)
        self.host.output_devices().is_ok()
    }

    fn description(&self) -> &'static str {
        "ALSA Direct - Bit-perfect with optional exclusive hardware access"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
