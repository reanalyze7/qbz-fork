use super::super::*;
use super::ctx::ThreadCtx;

/// True when a device has at least one supported output config.
pub(crate) fn is_device_valid(d: &rodio::cpal::Device) -> bool {
    d.supported_output_configs()
        .map(|configs| configs.count() > 0)
        .unwrap_or(false)
}

impl ThreadCtx {
    /// Find and initialize the audio device. Tries the backend system first,
    /// falls back to legacy CPAL. Takes the desired sample_rate/channels so
    /// DAC passthrough is preserved across reinitialization.
    pub(crate) fn init_device(
        &mut self,
        name: &Option<String>,
        sample_rate: u32,
        channels: u16,
    ) -> Option<StreamType> {
        if let Ok(settings) = self.settings.lock() {
            if settings.backend_type.is_some() || cfg!(target_os = "macos") {
                log::info!(
                    "Initializing backend system with {}Hz/{}ch",
                    sample_rate,
                    channels
                );
                match try_init_stream_with_backend(&settings, sample_rate, channels, &self.state) {
                    Some(Ok(stream_type)) => {
                        let device_name = settings
                            .output_device
                            .clone()
                            .unwrap_or_else(|| "Default".to_string());
                        log::info!(
                            "Audio output initialized via backend system at {}Hz (device: {})",
                            sample_rate,
                            device_name
                        );
                        self.state.set_current_device(Some(device_name));
                        return Some(stream_type);
                    }
                    Some(Err(e)) => {
                        #[cfg(target_os = "macos")]
                        if settings
                            .backend_type
                            .unwrap_or(AudioBackendType::SystemDefault)
                            == AudioBackendType::SystemDefault
                        {
                            log::error!(
                                "Could not start macOS audio output — {}. Not falling back to the legacy CPAL path because it would either play at the wrong speed (shared mode) or silently drop Exclusive Mode.",
                                e
                            );
                            self.state.set_current_device(None);
                            self.state.record_stream_error(e.clone());
                            return None;
                        }
                        log::warn!("Backend system init failed: {}, falling back to legacy", e);
                    }
                    None => {}
                }
            }
        }

        // Legacy CPAL path
        let device = self.find_legacy_device(name);

        let device = match device {
            Some(d) => {
                if let Some(name) = cpal_device_name(&d) {
                    log::info!("Using audio device: {}", name);
                    self.state.set_current_device(Some(name));
                }
                d
            }
            None => {
                log::error!("No audio output device available");
                self.state.set_current_device(None);
                return None;
            }
        };

        match DeviceSinkBuilder::from_device(device).and_then(|b| b.open_sink_or_fallback()) {
            Ok(mixer_sink) => {
                log::info!("Audio output initialized successfully");
                Some(StreamType::rodio(mixer_sink))
            }
            Err(e) => {
                log::error!(
                    "Failed to create audio output on device: {}. Trying default...",
                    e
                );
                match DeviceSinkBuilder::open_default_sink() {
                    Ok(mixer_sink) => {
                        log::info!("Fallback to default audio output succeeded");
                        Some(StreamType::rodio(mixer_sink))
                    }
                    Err(e2) => {
                        log::error!("Failed to create default audio output: {}", e2);
                        self.state.set_current_device(None);
                        None
                    }
                }
            }
        }
    }
}
