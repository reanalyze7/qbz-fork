use super::super::*;

/// Handle `AudioCommand::ReinitDevice`.
pub(crate) fn handle_reinit_device(ctx: &mut ThreadCtx, new_device: Option<String>) {
    log::info!(
        "Audio thread: reinitializing device (new: {:?})",
        new_device
    );
    ctx.pause_suspend_deadline = None;

    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
    }

    drop(ctx.stream_opt.take());
    log::info!("Audio thread: previous stream dropped, device released");

    std::thread::sleep(Duration::from_millis(100));

    ctx.current_device_name = new_device;
    // Use last known sample rate/channels to maintain DAC passthrough
    let sr = ctx.current_track_sample_rate.unwrap_or(48000);
    let ch = ctx.current_track_channels.unwrap_or(2);
    log::info!("ReinitDevice: reinitializing at {}Hz/{}ch", sr, ch);
    let device_name = ctx.current_device_name.clone();
    ctx.stream_opt = ctx.init_device(&device_name, sr, ch);

    if ctx.stream_opt.is_some() {
        log::info!("Audio thread: device reinitialized successfully");
        ctx.consecutive_sink_failures = 0;
    } else {
        log::error!("Audio thread: failed to reinitialize device");
    }

    // Preserve position so Resume can seek back to it.
    ctx.state.pause_playback_timer();
    ctx.state.is_playing.store(false, Ordering::SeqCst);
    // Keep current_audio_data / current_streaming_source intact so Resume
    // can recreate the engine and seek.
}

/// Handle `AudioCommand::ReleaseDevice`.
pub(crate) fn handle_release_device(ctx: &mut ThreadCtx) {
    log::info!("Audio thread: releasing output device (user-requested)");
    ctx.pause_suspend_deadline = None;
    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
    }
    drop(ctx.stream_opt.take());
    #[cfg(target_os = "linux")]
    {
        qbz_audio::alsa_backend::resume_suspended_sink();
        qbz_audio::pipewire_backend::PipeWireBackend::reset_pipewire_clock();
    }
    ctx.state.pause_playback_timer();
    ctx.state.is_playing.store(false, Ordering::SeqCst);
    // Keep current_audio_data / current_streaming_source intact so a later
    // Play / Resume reopens and continues.
    log::info!("Audio thread: output device released");
}
