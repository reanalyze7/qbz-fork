//! Backend manager - factory for creating backends. The detection layer:
//! shells out to `pw-cli`/`pactl`, reads `XDG_RUNTIME_DIR`.

use super::cpal_default::CpalDefaultBackend;
use super::trait_def::AudioBackend;
use super::device_config::BackendResult;
use super::types::AudioBackendType;

/// Backend manager - factory for creating backends
pub struct BackendManager;

impl BackendManager {
    /// Get all available backends on this system
    pub fn available_backends() -> Vec<AudioBackendType> {
        let mut backends = Vec::new();

        #[cfg(target_os = "linux")]
        {
            // System default (always available): shared OS default output via the
            // ALSA "default" PCM. The app-like OOTB choice; listed first.
            backends.push(AudioBackendType::SystemDefault);

            // PipeWire (check if running)
            if Self::is_pipewire_available() {
                backends.push(AudioBackendType::PipeWire);
            }

            // ALSA (always available on Linux)
            backends.push(AudioBackendType::Alsa);

            // PulseAudio (check if running)
            if Self::is_pulse_available() {
                backends.push(AudioBackendType::Pulse);
            }

            // JACK (#263 Tier 3): offered now that the player wiring is in place
            // (StreamType::Jack + dispatch + PlaybackEngine::Jack feeder/resampler).
            // The binary links libjack, so reaching here means it is present;
            // opening the client fails gracefully if no JACK server is reachable.
            backends.push(AudioBackendType::Jack);
        }

        #[cfg(not(target_os = "linux"))]
        {
            backends.push(AudioBackendType::SystemDefault);
        }

        backends
    }

    /// Create a backend instance
    pub fn create_backend(backend_type: AudioBackendType) -> BackendResult<Box<dyn AudioBackend>> {
        // Install the custom ALSA error handler once per process, before any
        // CPAL/ALSA enumeration fires. Idempotent via std::sync::Once.
        #[cfg(target_os = "linux")]
        crate::alsa_error_handler::install_once();

        match backend_type {
            AudioBackendType::PipeWire => {
                #[cfg(target_os = "linux")]
                {
                    let backend = crate::pipewire_backend::PipeWireBackend::new()?;
                    Ok(Box::new(backend))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    log::info!(
                        "PipeWire not available on this platform, using system default audio"
                    );
                    Ok(Box::new(CpalDefaultBackend::new()?))
                }
            }
            AudioBackendType::SystemDefault => {
                // "System": play through the OS default output via CPAL's default
                // host — CoreAudio/WASAPI off-Linux, the ALSA "default" PCM on Linux
                // (routes to PipeWire/Pulse/dmix for shared mixing). Opens at the
                // device's negotiated rate (rodio resamples); shared, no exclusivity,
                // no `pactl`. Available on every platform.
                Ok(Box::new(CpalDefaultBackend::new()?))
            }
            AudioBackendType::Alsa => {
                #[cfg(target_os = "linux")]
                {
                    let backend = crate::alsa_backend::AlsaBackend::new()?;
                    Ok(Box::new(backend))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    Err("ALSA backend only available on Linux".to_string())
                }
            }
            AudioBackendType::Pulse => {
                #[cfg(target_os = "linux")]
                {
                    let backend = crate::pulse_backend::PulseBackend::new()?;
                    Ok(Box::new(backend))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    Err("PulseAudio backend only available on Linux".to_string())
                }
            }
            AudioBackendType::Jack => {
                // JACK streams are created directly in the player dispatch
                // (qbz-player), NOT via the MixerDeviceSink AudioBackend trait.
                // This arm exists only so the factory stays exhaustive; the
                // returned backend is never used to open a JACK stream.
                #[cfg(target_os = "linux")]
                {
                    Ok(Box::new(CpalDefaultBackend::new()?))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    Err("JACK backend only available on Linux".to_string())
                }
            }
        }
    }
}
