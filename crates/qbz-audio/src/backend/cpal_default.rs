//! CPAL default backend ("System"): plays through the OS default output via
//! CPAL's default host — CoreAudio (macOS), WASAPI (Windows), and the ALSA
//! "default" PCM (Linux, which routes to PipeWire/Pulse/dmix for shared mixing).
//! No platform-specific commands (no `pactl`); opens at the device's negotiated
//! rate with rodio resampling, so it mixes with other apps like any normal player.
//!
//! The heavy method bodies live in sibling files (`cpal_default_enum.rs`,
//! `cpal_default_stream.rs`, and — macOS only — `cpal_macos/`) as plain
//! inherent methods on `CpalDefaultBackend`; this file holds the struct and
//! the single `impl AudioBackend for CpalDefaultBackend` block (a trait impl
//! cannot be split across files) whose methods just delegate to them.

use super::trait_def::AudioBackend;
use super::device_config::{AudioDevice, BackendConfig, BackendResult};
use super::types::AudioBackendType;
use rodio::cpal::traits::HostTrait;
use rodio::MixerDeviceSink;

pub struct CpalDefaultBackend {
    pub(super) host: rodio::cpal::Host,
}

impl CpalDefaultBackend {
    pub fn new() -> BackendResult<Self> {
        Ok(Self {
            host: rodio::cpal::default_host(),
        })
    }
}

impl AudioBackend for CpalDefaultBackend {
    fn backend_type(&self) -> AudioBackendType {
        AudioBackendType::SystemDefault
    }

    fn enumerate_devices(&self) -> BackendResult<Vec<AudioDevice>> {
        self.enumerate_devices_impl()
    }

    fn create_output_stream(&self, config: &BackendConfig) -> BackendResult<MixerDeviceSink> {
        self.create_output_stream_impl(config)
    }

    #[cfg(target_os = "macos")]
    fn create_output_stream_with_exclusive_guard(
        &self,
        config: &BackendConfig,
    ) -> BackendResult<(
        MixerDeviceSink,
        Option<crate::coreaudio_direct::CoreAudioExclusiveGuard>,
    )> {
        self.create_output_stream_with_exclusive_guard_impl(config)
    }

    fn is_available(&self) -> bool {
        self.host.default_output_device().is_some()
    }

    fn description(&self) -> &'static str {
        "System Audio - Default audio output"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
