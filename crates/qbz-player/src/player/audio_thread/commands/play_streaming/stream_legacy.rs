use super::super::super::*;

pub(super) fn clear_streaming_state(ctx: &mut ThreadCtx, message: impl Into<String>) {
    ctx.current_streaming_source = None;
    ctx.current_audio_data = None;
    ctx.state.set_loaded_audio(false);
    ctx.state.is_playing.store(false, Ordering::SeqCst);
    ctx.state.record_stream_error(message);
}

fn legacy_stream(
    ctx: &ThreadCtx,
    sample_rate: u32,
    channels: u16,
    dac_passthrough: bool,
) -> Result<StreamType, String> {
    let device = if let Some(ref name) = ctx.current_device_name {
        ctx.host
            .output_devices()
            .ok()
            .and_then(|mut devices| {
                devices.find(|d| cpal_device_name(d).as_deref() == Some(name.as_str()))
            })
            .or_else(|| ctx.host.default_output_device())
    } else {
        ctx.host.default_output_device()
    };

    let Some(device) = device else {
        return Err("No audio output device available for streaming".to_string());
    };

    if let Some(name) = cpal_device_name(&device) {
        ctx.state.set_current_device(Some(name));
    }

    create_output_stream_with_config(device, sample_rate, channels, dac_passthrough)
        .map(StreamType::rodio)
}

pub(super) fn create_stream(
    ctx: &mut ThreadCtx,
    sample_rate: u32,
    channels: u16,
    dac_passthrough: bool,
) -> Result<StreamType, String> {
    let Ok(settings) = ctx.settings.lock() else {
        let device = ctx.host.default_output_device();
        let Some(device) = device else {
            log::error!("No audio output device available for streaming");
            clear_streaming_state(ctx, "No audio output device available for streaming");
            return Err(String::new());
        };
        return create_output_stream_with_config(device, sample_rate, channels, dac_passthrough)
            .map(StreamType::rodio);
    };
    let settings = settings.clone();

    match try_init_stream_with_backend(&settings, sample_rate, channels, &ctx.state) {
        Some(Ok(stream)) => {
            let device_name = settings
                .output_device
                .clone()
                .unwrap_or_else(|| "Default".to_string());
            log::info!("Streaming backend using device: {}", device_name);
            ctx.state.set_current_device(Some(device_name));
            Ok(stream)
        }
        #[cfg(target_os = "macos")]
        Some(Err(e)) => Err(e),
        #[cfg(not(target_os = "macos"))]
        Some(Err(e)) => {
            log::warn!(
                "Backend system init failed for streaming: {}, falling back to legacy",
                e
            );
            legacy_stream(ctx, sample_rate, channels, dac_passthrough)
        }
        None => {
            log::info!("Backend system not configured, using legacy CPAL path");
            legacy_stream(ctx, sample_rate, channels, dac_passthrough)
        }
    }
}
