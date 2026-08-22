use super::super::super::*;

/// Create a `PlaybackEngine` from the current output stream. On repeated
/// sink-creation failure this auto-reinitializes the device. Returns `None`
/// when the caller should bail out of the whole `Play` command (already
/// logged and recorded in `ctx.state`).
pub(crate) fn build_engine(ctx: &mut ThreadCtx) -> Option<PlaybackEngine> {
    let Some(stream) = ctx.stream_opt.as_ref() else {
        log::error!("Audio thread: no audio device available");
        return None;
    };

    match stream {
        StreamType::Rodio {
            sink: mixer_sink, ..
        } => match PlaybackEngine::new_rodio(&mixer_sink.mixer()) {
            Ok(e) => {
                ctx.consecutive_sink_failures = 0;
                ctx.state.set_stream_error(false);
                Some(e)
            }
            Err(e) => {
                ctx.consecutive_sink_failures += 1;
                log::error!(
                    "Failed to create engine (attempt {}): {}",
                    ctx.consecutive_sink_failures,
                    e
                );

                if ctx.consecutive_sink_failures >= MAX_SINK_FAILURES {
                    log::warn!(
                        "Audio stream appears broken after {} failures. Auto-reinitializing...",
                        ctx.consecutive_sink_failures
                    );
                    ctx.state.set_stream_error(true);

                    drop(ctx.stream_opt.take());
                    std::thread::sleep(Duration::from_millis(200));

                    let sr = ctx.current_track_sample_rate.unwrap_or(48000);
                    let ch = ctx.current_track_channels.unwrap_or(2);
                    let device_name = ctx.current_device_name.clone();
                    ctx.stream_opt = ctx.init_device(&device_name, sr, ch);
                    if ctx.stream_opt.is_some() {
                        log::info!("Audio stream auto-reinitialized successfully at {}Hz", sr);
                        ctx.consecutive_sink_failures = 0;
                        ctx.state.set_stream_error(false);
                    } else {
                        log::error!("Auto-reinit failed. Audio device unavailable.");
                        ctx.state.is_playing.store(false, Ordering::SeqCst);
                        ctx.state.set_current_device(None);
                    }
                }
                None
            }
        },
        #[cfg(target_os = "linux")]
        StreamType::AlsaDirect(alsa_stream) => {
            ctx.consecutive_sink_failures = 0;
            ctx.state.set_stream_error(false);
            let hardware_volume = ctx
                .settings
                .lock()
                .ok()
                .map(|s| s.alsa_hardware_volume)
                .unwrap_or(false);
            Some(PlaybackEngine::new_alsa_direct(
                alsa_stream.clone(),
                hardware_volume,
            ))
        }
        #[cfg(target_os = "linux")]
        StreamType::Jack(jack_stream) => {
            ctx.consecutive_sink_failures = 0;
            ctx.state.set_stream_error(false);
            Some(PlaybackEngine::new_jack(jack_stream.clone()))
        }
    }
}
