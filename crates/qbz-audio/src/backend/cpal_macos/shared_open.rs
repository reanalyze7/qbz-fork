//! Shared-mode open-with-retry path, for macOS only. Split out from
//! `exclusive.rs` to keep both files under the line-count limit.

use super::super::cpal_default::CpalDefaultBackend;
use super::super::device_config::BackendResult;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

impl CpalDefaultBackend {
    /// Open a macOS shared-mode CPAL stream and verify it actually opened at
    /// CoreAudio's current nominal rate. CPAL caches device configs and can
    /// return a stale rate when the OS rate has just changed; passing the
    /// stale rate to CPAL produces a stream that runs at the wrong speed.
    ///
    /// Retries up to `max_attempts` times because the mismatch is racy — a
    /// fresh resolve + reopen usually lands on the correct rate.
    pub(in super::super) fn open_macos_shared_stream_with_retry(
        &self,
        effective_device_name: Option<&str>,
        max_attempts: usize,
    ) -> BackendResult<MixerDeviceSink> {
        debug_assert!(max_attempts >= 1, "max_attempts must be at least 1");

        let mut last_err: Option<String> = None;
        for attempt in 0..max_attempts {
            let device = if let Some(device_name) = effective_device_name {
                self.host
                    .output_devices()
                    .map_err(|e| format!("Failed to enumerate devices: {}", e))?
                    .find(|d| {
                        d.description()
                            .map(|desc| desc.name() == device_name)
                            .unwrap_or(false)
                    })
                    .ok_or_else(|| format!("Device '{}' not found", device_name))?
            } else {
                self.host
                    .default_output_device()
                    .ok_or_else(|| "No default output device found".to_string())?
            };

            let override_config =
                Self::shared_mode_nominal_stream_config(&device, effective_device_name);

            let builder = DeviceSinkBuilder::from_device(device)
                .map_err(|e| format!("Failed to create device sink builder: {}", e))?;

            let open_result = if let Some(ref cfg) = override_config {
                let buffer_size = rodio::cpal::BufferSize::Fixed((cfg.sample_rate() / 20).max(64));
                builder
                    .with_supported_config(cfg)
                    .with_buffer_size(buffer_size)
                    .open_stream()
            } else {
                builder.open_stream()
            };

            let mixer_sink = match open_result {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(format!("Failed to create output stream: {}", e));
                    break;
                }
            };

            let nominal_rate = Self::current_macos_nominal_rate(effective_device_name);
            let output_rate = mixer_sink.config().sample_rate().get();
            match nominal_rate {
                Some(nominal) if nominal != output_rate => {
                    drop(mixer_sink);
                    if attempt + 1 < max_attempts {
                        log::warn!(
                            "[CoreAudio] System audio rate changed during stream open (stream {}Hz, device now {}Hz). Retrying open ({}/{})",
                            output_rate,
                            nominal,
                            attempt + 2,
                            max_attempts
                        );
                        std::thread::sleep(super::MACOS_SHARED_OPEN_RETRY_DELAY);
                    }
                    last_err = Some(format!(
                        "macOS audio device changed its sample rate during stream open (opened at {}Hz, device is now {}Hz). This usually self-corrects — try playing again.",
                        output_rate, nominal
                    ));
                    continue;
                }
                _ => return Ok(mixer_sink),
            }
        }

        Err(last_err.unwrap_or_else(|| {
            "Could not open the macOS audio output stream after multiple attempts. Try selecting the device again or restarting playback.".to_string()
        }))
    }
}
