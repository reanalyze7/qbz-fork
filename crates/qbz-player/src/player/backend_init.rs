use super::*;

/// Try to create output stream using the backend system (if configured)
/// Returns None if backend system is not configured (backend_type = None)
///
/// For ALSA backend with hw: devices, may return AlsaDirect instead of Rodio stream.
pub(crate) fn try_init_stream_with_backend(
    audio_settings: &AudioSettings,
    sample_rate: u32,
    channels: u16,
    state: &SharedState,
) -> Option<Result<StreamType, String>> {
    // A None backend_type means "Auto" / unset. Resolve it to SystemDefault on
    // every platform instead of returning None — returning None made the caller
    // fall through to the legacy CPAL path, which forced the track rate onto the
    // shared default device: that froze the seekbar with no audio AND left a
    // process-wide stuck audio handle that survived Reset (#470). "Auto" is
    // resolved to a concrete backend in the UI; this is the backend-side safety
    // net for any remaining None (legacy installs, headless callers).
    let backend_type = audio_settings
        .backend_type
        .unwrap_or(qbz_audio::AudioBackendType::SystemDefault);

    log::info!(
        "Using backend system: {:?} (device: {:?}, plugin: {:?})",
        backend_type,
        audio_settings.output_device,
        audio_settings.alsa_plugin
    );

    // Create backend
    let backend = match BackendManager::create_backend(backend_type) {
        Ok(b) => b,
        Err(e) => {
            log::error!("Failed to create backend {:?}: {}", backend_type, e);
            return Some(Err(e));
        }
    };

    // Check availability
    if !backend.is_available() {
        let msg = format!("Backend {:?} is not available on this system", backend_type);
        log::error!("{}", msg);
        return Some(Err(msg));
    }

    // Build backend config
    let config = BackendConfig {
        backend_type,
        device_id: audio_settings.output_device.clone(),
        sample_rate,
        channels,
        exclusive_mode: audio_settings.exclusive_mode,
        alsa_plugin: audio_settings.alsa_plugin,
        pw_force_bitperfect: audio_settings.pw_force_bitperfect,
        skip_sink_switch: audio_settings.skip_sink_switch,
    };

    // For ALSA backend with hw: devices, try direct ALSA first (Linux only)
    #[cfg(target_os = "linux")]
    if backend_type == AudioBackendType::Alsa {
        // Check if device is hw: or plughw:
        if let Some(ref device_id) = config.device_id {
            if qbz_audio::AlsaDirectStream::is_hw_device(device_id) {
                log::info!("Detected hw: device, using ALSA Direct for bit-perfect playback");

                // Downcast backend to AlsaBackend to access try_create_direct_stream
                if let Some(alsa_backend) = backend
                    .as_any()
                    .downcast_ref::<qbz_audio::alsa_backend::AlsaBackend>()
                {
                    if let Some(result) = alsa_backend.try_create_direct_stream(&config) {
                        return Some(result.map(|(stream, mode)| {
                            log::info!("ALSA Direct stream created with mode: {:?}", mode);
                            state.set_bit_perfect_mode(Some(mode));
                            StreamType::AlsaDirect(Arc::new(stream))
                        }));
                    }
                }
            }
        }
    }

    // JACK (#263 Tier 3): create the JACK client/stream directly (not via the
    // MixerDeviceSink trait). Opt-in routing-freedom mode, NOT bit-perfect.
    #[cfg(target_os = "linux")]
    if backend_type == AudioBackendType::Jack {
        match qbz_audio::JackStream::new(config.channels) {
            Ok(stream) => {
                state.set_bit_perfect_mode(Some(qbz_audio::BitPerfectMode::Disabled));
                return Some(Ok(StreamType::Jack(Arc::new(stream))));
            }
            Err(e) => return Some(Err(format!("JACK backend unavailable: {e}"))),
        }
    }

    // Fallback to regular rodio stream (PipeWire, Pulse, ALSA via CPAL)
    match backend.create_output_stream_with_exclusive_guard(&config) {
        Ok((mixer_sink, _exclusive_guard)) => {
            let output_sample_rate = mixer_sink.config().sample_rate().get();
            log::info!(
                "Stream created via {:?} backend (requested {}Hz, output {}Hz)",
                backend_type,
                sample_rate,
                output_sample_rate
            );
            state.set_bit_perfect_mode(Some(BitPerfectMode::Disabled));
            #[cfg(target_os = "macos")]
            let stream = if backend_type == AudioBackendType::SystemDefault {
                StreamType::Rodio {
                    sink: mixer_sink,
                    exclusive_guard: _exclusive_guard,
                }
            } else {
                StreamType::rodio(mixer_sink)
            };
            #[cfg(not(target_os = "macos"))]
            let stream = StreamType::rodio(mixer_sink);
            Some(Ok(stream))
        }
        Err(e) => {
            log::error!("Backend stream creation failed: {}", e);
            Some(Err(e))
        }
    }
}

