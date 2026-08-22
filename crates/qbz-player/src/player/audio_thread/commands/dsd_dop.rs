use super::super::*;

/// Handle `AudioCommand::PlayDsdDop`: play a local DSD file via DoP (DSD
/// over PCM) on ALSA direct — Linux only.
pub(crate) fn handle(ctx: &mut ThreadCtx, path: std::path::PathBuf, track_id: u64) {
    #[cfg(target_os = "linux")]
    {
        log::info!(
            "Audio thread: DoP playback for track {} ({})",
            track_id,
            path.display()
        );
        ctx.pause_suspend_deadline = None;
        ctx.gapless_pending = None;
        ctx.gapless_request_armed = false;
        ctx.state.set_gapless_ready(false);
        ctx.state.set_gapless_next_track_id(0);

        let demux = match qbz_dsd::open_dsd(&path) {
            Ok(d) => d,
            Err(e) => {
                log::error!("DoP: cannot open DSD file: {}", e);
                ctx.state.set_stream_error(true);
                return;
            }
        };
        let dop = match qbz_dsd::DopStream::new(demux) {
            Ok(d) => d,
            Err(e) => {
                log::error!("DoP: cannot build DoP stream: {}", e);
                ctx.state.set_stream_error(true);
                return;
            }
        };
        let carrier = dop.carrier_rate();
        let dsd_rate = dop.dsd_rate();
        let duration = dop.total_frames() / (carrier.max(1) as u64);

        // DoP always needs a fresh S32 stream at the carrier rate — tear
        // down whatever is open.
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
            log::error!("DoP: no output device configured");
            ctx.state.set_stream_error(true);
            return;
        };
        let stream = match qbz_audio::alsa_backend::create_dop_stream(&device, carrier, 2) {
            Ok(st) => Arc::new(st),
            Err(e) => {
                log::error!("DoP: stream open failed: {}", e);
                ctx.state.set_stream_error(true);
                ctx.state.set_current_device(None);
                return;
            }
        };
        log::info!(
            "DoP: {} locked at {} Hz carrier on {}",
            qbz_dsd::dsd_label(dsd_rate),
            carrier,
            device
        );
        ctx.stream_opt = Some(StreamType::AlsaDirect(stream.clone()));
        ctx.state.set_current_device(Some(device));
        ctx.current_track_sample_rate = Some(carrier);
        ctx.current_track_channels = Some(2);
        ctx.current_audio_data = None;
        ctx.current_streaming_source = None;

        let mut engine = PlaybackEngine::new_alsa_dop(stream, false);
        if let Err(e) = engine.append_dop(Box::new(DsdErrorReport::new(dop, ctx.state.clone()))) {
            log::error!("DoP: append failed: {}", e);
            ctx.state.set_stream_error(true);
            return;
        }
        ctx.current_engine = Some(engine);
        ctx.state.set_loaded_audio(true);
        ctx.state.set_stream_error(false);
        ctx.state.set_stream_quality(dsd_rate, 1);
        ctx.state.duration.store(duration, Ordering::SeqCst);
        ctx.state.set_dsd_mode(1);
        ctx.state.is_playing.store(true, Ordering::SeqCst);
        ctx.state.position.store(0, Ordering::SeqCst);
        ctx.state.current_track_id.store(track_id, Ordering::SeqCst);
        ctx.state.start_playback_timer(0);
        log::info!("Audio thread: DoP playback STARTED");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (path, track_id);
        log::error!("DoP playback is Linux-only");
        ctx.state.set_stream_error(true);
    }
}
