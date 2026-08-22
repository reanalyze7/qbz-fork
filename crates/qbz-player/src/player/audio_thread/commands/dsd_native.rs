use super::super::*;

/// Handle `AudioCommand::PlayDsdNative`: play a local DSD file NATIVELY
/// (ALSA DSD_U32) — Linux only, requires kernel quirk support.
pub(crate) fn handle(ctx: &mut ThreadCtx, path: std::path::PathBuf, track_id: u64) {
    #[cfg(target_os = "linux")]
    {
        log::info!(
            "Audio thread: native DSD playback for track {} ({})",
            track_id,
            path.display()
        );
        ctx.pause_suspend_deadline = None;
        ctx.gapless_pending = None;
        ctx.gapless_request_armed = false;
        ctx.state.set_gapless_ready(false);
        ctx.state.set_gapless_next_track_id(0);

        let info = match qbz_dsd::open_dsd(&path) {
            Ok(d) => d.info().clone(),
            Err(e) => {
                log::error!("Native DSD: cannot open file: {}", e);
                ctx.state.set_stream_error(true);
                return;
            }
        };
        if info.channels != 2 {
            log::error!("Native DSD: stereo only");
            ctx.state.set_stream_error(true);
            return;
        }
        let rate = qbz_dsd::native_u32_rate(info.dsd_rate);
        let duration = (info.sample_count / 32) / (rate.max(1) as u64);

        if let Some(engine) = ctx.current_engine.take() {
            engine.stop();
            std::thread::sleep(Duration::from_millis(50));
        }
        drop(ctx.stream_opt.take());
        std::thread::sleep(Duration::from_millis(50));

        let device = ctx
            .settings
            .lock()
            .ok()
            .and_then(|s| s.output_device.clone());
        let Some(device) = device else {
            log::error!("Native DSD: no output device configured");
            ctx.state.set_stream_error(true);
            return;
        };
        let (stream, little_endian) =
            match qbz_audio::alsa_backend::create_native_dsd_stream(&device, info.dsd_rate, 2) {
                Ok(pair) => pair,
                Err(e) => {
                    log::error!("Native DSD: stream open failed: {}", e);
                    ctx.state.set_stream_error(true);
                    ctx.state.set_current_device(None);
                    return;
                }
            };
        let stream = Arc::new(stream);
        let native_src = match qbz_dsd::open_dsd(&path)
            .map_err(|e| e.to_string())
            .and_then(|d| qbz_dsd::NativeDsdStream::new(d, little_endian).map_err(|e| e.to_string()))
        {
            Ok(n) => n,
            Err(e) => {
                log::error!("Native DSD: source build failed: {}", e);
                ctx.state.set_stream_error(true);
                return;
            }
        };
        log::info!(
            "Native DSD: {} locked at {} Hz U32 ({}) on {}",
            qbz_dsd::dsd_label(info.dsd_rate),
            rate,
            if little_endian { "LE" } else { "BE" },
            device
        );
        ctx.stream_opt = Some(StreamType::AlsaDirect(stream.clone()));
        ctx.state.set_current_device(Some(device));
        ctx.current_track_sample_rate = Some(rate);
        ctx.current_track_channels = Some(2);
        ctx.current_audio_data = None;
        ctx.current_streaming_source = None;

        let mut engine = PlaybackEngine::new_alsa_dop(stream, true);
        if let Err(e) =
            engine.append_dop(Box::new(DsdErrorReport::new(native_src, ctx.state.clone())))
        {
            log::error!("Native DSD: append failed: {}", e);
            ctx.state.set_stream_error(true);
            return;
        }
        ctx.current_engine = Some(engine);
        ctx.state.set_loaded_audio(true);
        ctx.state.set_stream_error(false);
        ctx.state.set_stream_quality(info.dsd_rate, 1);
        ctx.state.duration.store(duration, Ordering::SeqCst);
        ctx.state.set_dsd_mode(if little_endian { 3 } else { 2 });
        ctx.state.is_playing.store(true, Ordering::SeqCst);
        ctx.state.position.store(0, Ordering::SeqCst);
        ctx.state.current_track_id.store(track_id, Ordering::SeqCst);
        ctx.state.start_playback_timer(0);
        log::info!("Audio thread: native DSD playback STARTED");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (path, track_id);
        log::error!("Native DSD playback is Linux-only");
        ctx.state.set_stream_error(true);
    }
}
