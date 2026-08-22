//! Final CPAL stream construction: supported-config check, buffer sizing,
//! and opening the device sink.

use crate::backend::BackendConfig;
use rodio::{
    cpal::{
        traits::DeviceTrait, BufferSize, SampleFormat, StreamConfig, SupportedBufferSize,
        SupportedStreamConfig,
    },
    DeviceSinkBuilder, MixerDeviceSink,
};

pub(super) fn build_stream(
    device: rodio::cpal::Device,
    config: &BackendConfig,
    effective_rate: u32,
) -> Result<MixerDeviceSink, String> {
    // Create output stream with custom sample rate configuration
    log::info!(
        "[PipeWire Backend] Creating stream: {}Hz (track: {}Hz), {} channels, exclusive: {}",
        effective_rate,
        config.sample_rate,
        config.channels,
        config.exclusive_mode
    );

    // Create StreamConfig with effective sample rate
    // Note: buffer_size here is unused — with_supported_config() resets it.
    // The actual buffer size is set via with_buffer_size() below.
    let stream_config = StreamConfig {
        channels: config.channels,
        sample_rate: effective_rate,
        buffer_size: BufferSize::Default,
    };

    // Check if CPAL device supports this configuration
    let supported_configs = device
        .supported_output_configs()
        .map_err(|e| format!("Failed to get supported configs: {}", e))?;

    let mut found_matching = false;
    for range in supported_configs {
        if range.channels() == config.channels
            && effective_rate >= range.min_sample_rate()
            && effective_rate <= range.max_sample_rate()
        {
            found_matching = true;
            log::info!(
                "[PipeWire Backend] CPAL device supports {}Hz (range: {}-{}Hz)",
                effective_rate,
                range.min_sample_rate(),
                range.max_sample_rate()
            );
            break;
        }
    }

    if !found_matching {
        log::warn!(
            "[PipeWire Backend] CPAL device may not support {}Hz, attempting anyway",
            effective_rate
        );
    }

    // Create SupportedStreamConfig
    let supported_config = SupportedStreamConfig::new(
        stream_config.channels,
        stream_config.sample_rate,
        SupportedBufferSize::Range { min: 64, max: 8192 },
        SampleFormat::F32,
    );

    // Compute buffer size — must be applied AFTER with_supported_config()
    // because that method resets buffer_size to Default via ..Default::default().
    // MixerDeviceSink has zero internal buffering, so CPAL's buffer is the
    // ONLY buffer between the mixer and audio hardware.
    let cpal_buffer_size = if config.exclusive_mode {
        BufferSize::Fixed(512) // Low latency for exclusive mode
    } else {
        // ~100ms buffer, matching old vendored cpal period size.
        // Prevents underruns at high sample rates (192kHz = 19200 frames).
        BufferSize::Fixed(effective_rate / 10)
    };
    log::info!("[PipeWire Backend] Buffer size: {:?}", cpal_buffer_size);

    // Create MixerDeviceSink with custom config
    let mixer_sink = DeviceSinkBuilder::from_device(device)
        .map_err(|e| format!("Failed to create device sink builder: {}", e))?
        .with_supported_config(&supported_config)
        .with_buffer_size(cpal_buffer_size)
        .open_stream()
        .map_err(|e| {
            format!(
                "Failed to create output stream at {}Hz: {}",
                effective_rate, e
            )
        })?;

    log::info!(
        "[PipeWire Backend] Output stream created successfully at {}Hz",
        effective_rate
    );

    Ok(mixer_sink)
}
