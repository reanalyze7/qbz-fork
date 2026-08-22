use super::super::*;

/// Handle `AudioCommand::Pause`.
pub(crate) fn handle_pause(ctx: &mut ThreadCtx) {
    if let Some(ref engine) = ctx.current_engine {
        engine.pause();
        ctx.state.pause_playback_timer();
        ctx.state.is_playing.store(false, Ordering::SeqCst);
        ctx.pause_suspend_deadline =
            Some(Instant::now() + Duration::from_millis(PAUSE_SUSPEND_DELAY_MS));
        log::info!(
            "Audio thread: paused at {}s",
            ctx.state.position.load(Ordering::SeqCst)
        );
    }
}

/// Handle `AudioCommand::Stop`.
pub(crate) fn handle_stop(ctx: &mut ThreadCtx) {
    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
    }
    ctx.current_audio_data = None;
    ctx.current_streaming_source = None;
    ctx.current_normalization_gain = None;
    ctx.current_gain_atomic = None;
    ctx.gapless_pending = None;
    ctx.gapless_request_armed = false;
    ctx.state.set_gapless_ready(false);
    ctx.state.set_gapless_next_track_id(0);
    ctx.analyzer_enabled.store(false, Ordering::SeqCst);
    ctx.state.set_normalization_gain(None);
    ctx.state.is_playing.store(false, Ordering::SeqCst);
    ctx.state.position.store(0, Ordering::SeqCst);
    ctx.state.set_loaded_audio(false);
    ctx.state.playback_start_millis.store(0, Ordering::SeqCst);
    ctx.state.position_at_start.store(0, Ordering::SeqCst);
    // Defer dropping the stream so a Play immediately following Stop (the
    // frontend's track-change pattern is Stop -> Play, not append) can reuse
    // the open device. The idle loop's pause-suspend handler drops the
    // stream when this deadline fires.
    ctx.pause_suspend_deadline =
        Some(Instant::now() + Duration::from_millis(PAUSE_SUSPEND_DELAY_MS));
    #[cfg(target_os = "linux")]
    qbz_audio::pipewire_backend::PipeWireBackend::reset_pipewire_clock();
    log::info!("Audio thread: stopped");
}

/// Handle `AudioCommand::SetVolume`.
pub(crate) fn handle_set_volume(ctx: &mut ThreadCtx, volume: f32) {
    ctx.state.volume.store(volume.to_bits(), Ordering::SeqCst);
    if let Some(ref engine) = ctx.current_engine {
        apply_engine_volume(&ctx.stream_opt, engine, volume);
    }
    log::debug!("Audio thread: volume set to {}", volume);
}
