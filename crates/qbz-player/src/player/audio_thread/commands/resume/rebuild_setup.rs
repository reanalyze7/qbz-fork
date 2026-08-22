use super::super::super::*;

/// Recover the audio bytes to resume from: already-stored data, or a
/// completed streaming source promoted to cached data.
pub(super) fn fetch_resume_audio_data(ctx: &mut ThreadCtx) -> Option<Vec<u8>> {
    if let Some(ref data) = ctx.current_audio_data {
        return Some(data.clone());
    }
    let streaming_src = ctx.current_streaming_source.clone()?;
    if streaming_src.is_complete() {
        match streaming_src.take_complete_data() {
            Some(data) => {
                log::info!(
                    "Resume: using complete streaming data ({} bytes)",
                    data.len()
                );
                ctx.current_audio_data = Some(data.clone());
                Some(data)
            }
            None => {
                log::warn!(
                    "Audio thread: cannot resume - streaming source complete but data unavailable"
                );
                None
            }
        }
    } else {
        log::warn!(
            "Audio thread: cannot resume - streaming not complete yet ({} bytes buffered)",
            streaming_src.buffer_size()
        );
        None
    }
}

/// Reinitialize the output stream if needed and build a fresh
/// `PlaybackEngine` from it.
pub(super) fn build_resume_engine(ctx: &mut ThreadCtx) -> Option<PlaybackEngine> {
    if ctx.stream_opt.is_none() {
        let sr = ctx.current_track_sample_rate.unwrap_or(48000);
        let ch = ctx.current_track_channels.unwrap_or(2);
        log::info!("Resume: reinitializing stream at {}Hz/{}ch", sr, ch);
        let device_name = ctx.current_device_name.clone();
        ctx.stream_opt = ctx.init_device(&device_name, sr, ch);
    }

    let Some(stream) = ctx.stream_opt.as_ref() else {
        log::error!("Audio thread: cannot resume - no audio device available");
        return None;
    };

    match stream {
        StreamType::Rodio {
            sink: mixer_sink, ..
        } => match PlaybackEngine::new_rodio(&mixer_sink.mixer()) {
            Ok(e) => Some(e),
            Err(e) => {
                log::error!("Failed to create engine for resume: {}", e);
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
