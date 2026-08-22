use super::super::super::*;

/// Legacy CPAL device lookup shared by both branches below.
fn find_device(ctx: &ThreadCtx, name: &Option<String>) -> Option<rodio::cpal::Device> {
    if let Some(ref name) = name {
        log::info!("Looking for audio device: {}", name);
        let found = ctx.host.output_devices().ok().and_then(|mut devices| {
            devices.find(|d| cpal_device_name(d).as_deref() == Some(name.as_str()))
        });
        match found {
            Some(d) if crate::player::audio_thread::ctx_device::is_device_valid(&d) => {
                log::info!("Found and validated device: {}", name);
                Some(d)
            }
            Some(_) => {
                log::warn!(
                    "Device '{}' found but has no valid output configs, using default",
                    name
                );
                ctx.host.default_output_device()
            }
            None => {
                log::warn!("Device '{}' not found, using default", name);
                ctx.host.default_output_device()
            }
        }
    } else {
        log::info!("Using default audio device");
        ctx.host.default_output_device()
    }
}

fn legacy_stream(
    ctx: &ThreadCtx,
    sample_rate: u32,
    channels: u16,
    dac_passthrough: bool,
) -> Result<StreamType, String> {
    let Some(device) = find_device(ctx, &ctx.current_device_name) else {
        return Err("No audio output device available".to_string());
    };

    if let Some(name) = cpal_device_name(&device) {
        log::info!("Using audio device: {}", name);
        ctx.state.set_current_device(Some(name));
    }

    create_output_stream_with_config(device, sample_rate, channels, dac_passthrough)
        .map(StreamType::rodio)
}

/// Try the backend system first (if configured), then fall back to legacy
/// CPAL. Shared by the "backend not configured" and "backend init failed"
/// paths (#591/#592 follow-up — same shape as the streaming handler). Uses
/// the track's native rate/channels unchanged — no resampling introduced.
pub(crate) fn create_stream(
    ctx: &mut ThreadCtx,
    sample_rate: u32,
    channels: u16,
    dac_passthrough: bool,
) -> Result<StreamType, String> {
    let settings = match ctx.settings.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => {
            // Failed to lock settings, use legacy path with CPAL device search.
            return legacy_stream(ctx, sample_rate, channels, dac_passthrough);
        }
    };

    match try_init_stream_with_backend(&settings, sample_rate, channels, &ctx.state) {
        Some(Ok(stream)) => {
            let device_name = settings
                .output_device
                .clone()
                .unwrap_or_else(|| "Default".to_string());
            log::info!("Backend system using device: {}", device_name);
            ctx.state.set_current_device(Some(device_name));
            Ok(stream)
        }
        // macOS: the backend path is authoritative (CoreAudio ownership +
        // nominal-rate handling); surface the failure unchanged.
        #[cfg(target_os = "macos")]
        Some(Err(e)) => Err(e),
        // Linux/other: a backend init failure must not dead-end the Play
        // (#591/#592 follow-up). Fall back to the legacy CPAL path.
        #[cfg(not(target_os = "macos"))]
        Some(Err(e)) => {
            log::warn!(
                "Backend system init failed for Play: {}, falling back to legacy",
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
