use super::super::super::*;

pub(super) fn seek_abort(state: &SharedState, why: &str) {
    log::error!("Audio thread: seek aborted: {why}");
    state.is_playing.store(false, Ordering::SeqCst);
    state.set_stream_error(true);
}

/// Build a fresh `PlaybackEngine` from the current output stream for a seek.
/// Returns `None` when the failure was already logged and recorded.
pub(super) fn build_seek_engine(ctx: &mut ThreadCtx) -> Option<PlaybackEngine> {
    let Some(stream) = ctx.stream_opt.as_ref() else {
        seek_abort(&ctx.state, "no audio device available");
        return None;
    };

    match stream {
        StreamType::Rodio {
            sink: mixer_sink, ..
        } => match PlaybackEngine::new_rodio(&mixer_sink.mixer()) {
            Ok(e) => Some(e),
            Err(e) => {
                seek_abort(&ctx.state, &format!("rodio engine create failed: {e}"));
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
