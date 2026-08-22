//! `MixerDeviceSink` construction (config, exclusive-mode PipeWire suspend,
//! stream open) for `AlsaBackend::create_output_stream`, split out of that
//! function purely to stay under the per-file line budget — behavior is
//! unchanged.

use super::pipewire_suspend::suspend_default_sink_for_exclusive;
use crate::backend::BackendConfig;
use rodio::{
    cpal::{BufferSize, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig},
    DeviceSinkBuilder, MixerDeviceSink,
};

/// Build and open the `MixerDeviceSink` for `device` at `effective_rate`,
/// suspending PipeWire first when exclusive mode is requested.
pub(super) fn build_mixer_sink(
    device: rodio::cpal::Device,
    effective_rate: u32,
    config: &BackendConfig,
) -> Result<MixerDeviceSink, String> {
    // Rebuild StreamConfig with effective rate
    let stream_config = StreamConfig {
        channels: config.channels,
        sample_rate: effective_rate,
        buffer_size: if config.exclusive_mode {
            BufferSize::Fixed(512)
        } else {
            BufferSize::Fixed(effective_rate / 10)
        },
    };

    // Create SupportedStreamConfig
    let supported_config = SupportedStreamConfig::new(
        stream_config.channels,
        stream_config.sample_rate,
        SupportedBufferSize::Range { min: 64, max: 8192 },
        SampleFormat::F32,
    );

    // In exclusive mode, PipeWire may have re-acquired the device after the
    // previous ALSA Direct stream released it. Suspend PipeWire before opening.
    if config.exclusive_mode {
        log::info!("[ALSA Backend] Exclusive mode: suspending PipeWire sinks before CPAL stream");
        suspend_default_sink_for_exclusive();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    // Create MixerDeviceSink with custom config
    let mixer_sink = DeviceSinkBuilder::from_device(device)
        .map_err(|e| {
            if config.exclusive_mode {
                format!(
                    "Failed to create exclusive ALSA stream at {}Hz: {}. Device may be in use by another application.",
                    effective_rate, e
                )
            } else {
                format!("Failed to create ALSA device sink builder at {}Hz: {}", effective_rate, e)
            }
        })?
        .with_supported_config(&supported_config)
        .open_stream()
        .map_err(|e| {
            if config.exclusive_mode {
                format!(
                    "Failed to create exclusive ALSA stream at {}Hz: {}. Device may be in use by another application.",
                    effective_rate, e
                )
            } else {
                format!("Failed to create ALSA stream at {}Hz: {}", effective_rate, e)
            }
        })?;

    if effective_rate != config.sample_rate {
        log::info!(
            "[ALSA Backend] Output stream created at {}Hz (resampled from {}Hz, exclusive: {})",
            effective_rate,
            config.sample_rate,
            config.exclusive_mode
        );
    } else {
        log::info!(
            "[ALSA Backend] Output stream created successfully at {}Hz (exclusive: {})",
            config.sample_rate,
            config.exclusive_mode
        );
    }

    Ok(mixer_sink)
}
