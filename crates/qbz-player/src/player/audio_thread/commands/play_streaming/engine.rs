use super::super::super::*;
use super::stream_legacy::clear_streaming_state;

/// Create a `PlaybackEngine` from the current output stream for a streaming
/// play. Returns `None` when the caller should bail (already logged and
/// recorded in `ctx.state`).
pub(crate) fn build_engine(ctx: &mut ThreadCtx) -> Option<PlaybackEngine> {
    let Some(stream) = ctx.stream_opt.as_ref() else {
        log::error!("Audio thread: no audio device available for streaming");
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
                log::error!("Failed to create engine for streaming: {}", e);
                clear_streaming_state(ctx, "Failed to create the playback engine for streaming");
                None
            }
        },
        #[cfg(target_os = "linux")]
        StreamType::AlsaDirect(alsa_stream) => {
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
        StreamType::Jack(jack_stream) => Some(PlaybackEngine::new_jack(jack_stream.clone())),
    }
}
