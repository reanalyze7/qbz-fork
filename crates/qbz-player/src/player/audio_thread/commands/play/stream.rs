use super::super::super::*;

/// Ensure the output stream matches `sample_rate`/`channels`, recreating it
/// if needed. `Err(())` means the failure was already logged and recorded
/// in `ctx.state`; the caller should bail out of the whole `Play` command.
pub(crate) fn ensure_stream(ctx: &mut ThreadCtx, sample_rate: u32, channels: u16) -> Result<(), ()> {
    let StreamRecreateDecision {
        needs_new_stream,
        format_changed,
        dac_passthrough,
        using_alsa_direct,
        using_coreaudio_exclusive,
        coreaudio_shared_rate_mismatch,
    } = evaluate_stream_recreate(
        &ctx.settings,
        &ctx.stream_opt,
        ctx.current_track_sample_rate,
        ctx.current_track_channels,
        sample_rate,
        channels,
        "Play",
    );

    if !needs_new_stream {
        if format_changed {
            log::info!(
                "Audio format changed from {:?}Hz/{:?}ch to {}Hz/{}ch - reusing audio stream (DAC passthrough disabled, gapless enabled)",
                ctx.current_track_sample_rate,
                ctx.current_track_channels,
                sample_rate,
                channels
            );
        }
        return Ok(());
    }

    if ctx.stream_opt.is_some() {
        if coreaudio_shared_rate_mismatch.is_none()
            && (dac_passthrough || using_alsa_direct || using_coreaudio_exclusive)
            && format_changed
        {
            let mode = if using_coreaudio_exclusive {
                "CoreAudio exclusive"
            } else if using_alsa_direct {
                "ALSA Direct"
            } else {
                "DAC passthrough"
            };
            log::info!(
                "Sample rate/channels changed from {:?}Hz/{:?}ch to {}Hz/{}ch - recreating audio stream ({})",
                ctx.current_track_sample_rate,
                ctx.current_track_channels,
                sample_rate,
                channels,
                mode
            );
        }
        // Stop engine FIRST so its writer thread releases its
        // Arc<AlsaDirectStream> reference before we drop the stream.
        if let Some(engine) = ctx.current_engine.take() {
            engine.stop();
            std::thread::sleep(Duration::from_millis(50));
        }
        drop(ctx.stream_opt.take());
        std::thread::sleep(Duration::from_millis(50));
    }

    log::info!(
        "DAC passthrough: {}, ALSA Direct: {}, CoreAudio exclusive: {}",
        dac_passthrough,
        using_alsa_direct,
        using_coreaudio_exclusive
    );

    let stream_result = super::stream_legacy::create_stream(ctx, sample_rate, channels, dac_passthrough);

    match stream_result {
        Ok(stream) => {
            ctx.stream_opt = Some(stream);
            ctx.state.set_stream_error(false);

            if let Ok(settings) = ctx.settings.lock() {
                if let Some(ref device_name) = settings.output_device {
                    ctx.state.set_current_device(Some(device_name.clone()));
                    log::info!(
                        "Audio stream ready at {}Hz on device: {}",
                        sample_rate,
                        device_name
                    );
                } else {
                    ctx.state.set_current_device(Some("Default".to_string()));
                    log::info!("Audio stream ready at {}Hz on default device", sample_rate);
                }
            } else {
                log::info!("Audio stream ready at {}Hz", sample_rate);
            }

            // Delay to ensure stream is fully initialized before decoder starts
            std::thread::sleep(Duration::from_millis(150));
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to create stream at {}Hz: {}", sample_rate, e);
            ctx.state.set_stream_error(true);
            ctx.state.set_current_device(None);
            Err(())
        }
    }
}
