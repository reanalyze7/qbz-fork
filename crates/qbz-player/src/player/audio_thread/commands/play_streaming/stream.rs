use super::super::super::*;
use super::stream_legacy::{clear_streaming_state, create_stream};

/// Ensure the output stream matches `sample_rate`/`channels` for a streaming
/// play. `Err(())` means failure was already logged/recorded; caller bails.
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
        "Streaming",
    );

    if !needs_new_stream {
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
                "Streaming: Sample rate/channels changed to {}Hz/{}ch - recreating audio stream ({})",
                sample_rate, channels, mode
            );
        }
        if let Some(engine) = ctx.current_engine.take() {
            engine.stop();
            std::thread::sleep(Duration::from_millis(50));
        }
        drop(ctx.stream_opt.take());
        std::thread::sleep(Duration::from_millis(50));
    }

    match create_stream(ctx, sample_rate, channels, dac_passthrough) {
        Ok(stream) => {
            ctx.stream_opt = Some(stream);
            ctx.state.set_stream_error(false);
            log::info!("Streaming audio stream ready at {}Hz", sample_rate);
            std::thread::sleep(Duration::from_millis(150));
            Ok(())
        }
        Err(e) => {
            if !e.is_empty() {
                log::error!(
                    "Failed to create stream for streaming at {}Hz: {}",
                    sample_rate,
                    e
                );
                clear_streaming_state(ctx, e);
            }
            Err(())
        }
    }
}
