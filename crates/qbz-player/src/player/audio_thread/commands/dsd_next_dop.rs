use super::super::*;

/// Handle `AudioCommand::PlayNextDsdDop`: queue the next DSD track on the
/// ACTIVE DoP/native engine for a gapless transition — Linux only.
pub(crate) fn handle(ctx: &mut ThreadCtx, path: std::path::PathBuf, track_id: u64) {
    #[cfg(target_os = "linux")]
    {
        let Some(engine) = ctx.current_engine.as_mut() else {
            ctx.state.set_gapless_ready(false);
            return;
        };
        if !engine.is_dop() {
            log::info!("Gapless DoP: engine is not DoP, ignoring");
            ctx.state.set_gapless_ready(false);
            return;
        }
        // Build the packing matching the ACTIVE direct mode (1 = DoP, 2/3 =
        // native BE/LE).
        let mode = ctx.state.dsd_mode();
        let built: Result<(Box<dyn Iterator<Item = i32> + Send>, u32, u64), String> =
            qbz_dsd::open_dsd(&path)
                .map_err(|e| e.to_string())
                .and_then(|d| match mode {
                    1 => qbz_dsd::DopStream::new(d).map_err(|e| e.to_string()).map(|st| {
                        let rate = st.carrier_rate();
                        let frames = st.total_frames();
                        (
                            Box::new(DsdErrorReport::new(st, ctx.state.clone()))
                                as Box<dyn Iterator<Item = i32> + Send>,
                            rate,
                            frames,
                        )
                    }),
                    2 | 3 => qbz_dsd::NativeDsdStream::new(d, mode == 3)
                        .map_err(|e| e.to_string())
                        .map(|st| {
                            let rate = st.rate();
                            let frames = st.total_frames();
                            (
                                Box::new(DsdErrorReport::new(st, ctx.state.clone()))
                                    as Box<dyn Iterator<Item = i32> + Send>,
                                rate,
                                frames,
                            )
                        }),
                    _ => Err("no DSD-direct mode active".to_string()),
                });
        let (src, rate, total_frames) = match built {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Gapless DSD: cannot open next track: {}", e);
                ctx.state.set_gapless_ready(false);
                return;
            }
        };
        if ctx.current_track_sample_rate != Some(rate) {
            log::info!(
                "Gapless DSD: rate mismatch ({:?} vs {}), ignoring",
                ctx.current_track_sample_rate,
                rate
            );
            ctx.state.set_gapless_ready(false);
            return;
        }
        let duration = total_frames / (rate.max(1) as u64);
        match engine.append_dop(src) {
            Ok(()) => {
                // data stays empty: the DoP engine never resumes from
                // ctx.current_audio_data (pause-suspend teardown is gated
                // off in DoP mode).
                ctx.gapless_pending = Some(GaplessPending {
                    track_id,
                    duration_secs: duration,
                    data: Vec::new(),
                    normalization_gain: None,
                });
                ctx.state.set_gapless_next_track_id(track_id);
                ctx.state.set_gapless_ready(false);
                log::info!(
                    "Gapless DoP: queued track {} for seamless DSD transition",
                    track_id
                );
            }
            Err(e) => {
                log::warn!("Gapless DoP: append failed: {}", e);
                ctx.state.set_gapless_ready(false);
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (path, track_id);
        ctx.state.set_gapless_ready(false);
    }
}
