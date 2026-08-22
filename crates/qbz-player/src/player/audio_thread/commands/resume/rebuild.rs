use super::super::super::*;

fn fetch_resume_audio_data(ctx: &mut ThreadCtx) -> Option<Vec<u8>> {
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

fn build_resume_engine(ctx: &mut ThreadCtx) -> Option<PlaybackEngine> {
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

/// Rebuild a `PlaybackEngine` from scratch (no active engine) and resume
/// from the last known position.
pub(crate) fn resume_from_scratch(ctx: &mut ThreadCtx) {
    let Some(audio_data) = fetch_resume_audio_data(ctx) else {
        return;
    };

    let Some(mut engine) = build_resume_engine(ctx) else {
        return;
    };

    let volume = f32::from_bits(ctx.state.volume.load(Ordering::SeqCst));
    apply_engine_volume(&ctx.stream_opt, &engine, volume);

    let source = match decode_with_fallback(&audio_data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to decode audio for resume: {}", e);
            return;
        }
    };

    // Backfill duration if a prior failed PlayStreaming never stored it
    // (#508); must read total_duration() before skip_duration consumes it.
    if ctx.state.duration.load(Ordering::SeqCst) == 0 {
        if let Some(d) = source.total_duration() {
            ctx.state.duration.store(d.as_secs(), Ordering::SeqCst);
        }
    }

    let resume_pos = ctx.state.position.load(Ordering::SeqCst);
    let skipped_source: Box<dyn Source<Item = f32> + Send> = if resume_pos > 0 {
        Box::new(source.skip_duration(Duration::from_secs(resume_pos)))
    } else {
        source
    };

    let skipped_source = crate::player::audio_thread::ctx_source::wrap_source(
        &ctx.diagnostic,
        &ctx.viz_tap,
        &ctx.analyzer_tx,
        &ctx.analyzer_enabled,
        skipped_source,
        ctx.current_normalization_gain,
        ctx.current_gain_atomic.clone(),
    );
    if let Err(e) = engine.append(skipped_source) {
        log::error!("Failed to append source for resume: {}", e);
        return;
    }
    ctx.state.start_playback_timer(resume_pos);
    ctx.state.is_playing.store(true, Ordering::SeqCst);
    ctx.current_engine = Some(engine);

    log::info!("Audio thread: resumed from {}s", resume_pos);
}
