use super::*;

pub(crate) fn create_output_stream_with_config(
    device: rodio::cpal::Device,
    sample_rate: u32,
    channels: u16,
    exclusive_mode: bool,
) -> Result<MixerDeviceSink, String> {
    log::info!(
        "Creating MixerDeviceSink: {}Hz, {} channels, exclusive: {}",
        sample_rate,
        channels,
        exclusive_mode
    );

    // Create StreamConfig with desired sample rate
    // Note: buffer_size here is unused — with_supported_config() resets it.
    // The actual buffer size is set via with_buffer_size() below.
    let config = StreamConfig {
        channels,
        sample_rate,
        buffer_size: BufferSize::Default,
    };

    // Check if device supports this configuration
    let supported_configs = device
        .supported_output_configs()
        .map_err(|e| format!("Failed to get supported configs: {}", e))?;

    let mut found_matching = false;
    for range in supported_configs {
        if range.channels() == channels
            && sample_rate >= range.min_sample_rate()
            && sample_rate <= range.max_sample_rate()
        {
            found_matching = true;
            log::info!(
                "Device supports {}Hz (range: {}-{}Hz)",
                sample_rate,
                range.min_sample_rate(),
                range.max_sample_rate()
            );
            break;
        }
    }

    if !found_matching {
        log::warn!(
            "Device may not support {}Hz, attempting anyway",
            sample_rate
        );
    }

    // Create SupportedStreamConfig
    let supported_config = SupportedStreamConfig::new(
        config.channels,
        config.sample_rate,
        SupportedBufferSize::Range { min: 64, max: 8192 },
        SampleFormat::F32,
    );

    // Compute buffer size — must be applied AFTER with_supported_config()
    // because that method resets buffer_size to Default via ..Default::default().
    // MixerDeviceSink has zero internal buffering, so CPAL's buffer is the
    // ONLY buffer between the mixer and audio hardware.
    let cpal_buffer_size = if exclusive_mode {
        BufferSize::Fixed(512) // Low latency for exclusive mode
    } else {
        // ~100ms buffer, matching old vendored cpal period size.
        // Prevents underruns at high sample rates (192kHz = 19200 frames).
        BufferSize::Fixed(sample_rate / 10)
    };
    log::info!("Buffer size: {:?}", cpal_buffer_size);

    // Create MixerDeviceSink with custom config
    match DeviceSinkBuilder::from_device(device) {
        Ok(builder) => {
            match builder
                .with_supported_config(&supported_config)
                .with_buffer_size(cpal_buffer_size)
                .open_stream()
            {
                Ok(mixer_sink) => {
                    log::info!("MixerDeviceSink created successfully at {}Hz", sample_rate);
                    Ok(mixer_sink)
                }
                Err(e) => {
                    log::error!("Failed to open stream at {}Hz: {}", sample_rate, e);
                    Err(format!("Failed to create output stream: {}", e))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create device sink builder: {}", e);
            Err(format!("Failed to create output stream: {}", e))
        }
    }
}
