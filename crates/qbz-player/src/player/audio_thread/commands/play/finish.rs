use super::super::super::*;

/// Decode `data`, apply normalization, append to `engine`, and flip the
/// shared state over to "playing". Consumes `engine`, storing it back into
/// `ctx.current_engine` on success.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decode_and_start(
    ctx: &mut ThreadCtx,
    mut engine: PlaybackEngine,
    data: Vec<u8>,
    track_id: u64,
    duration_secs: u64,
    sample_rate: u32,
    channels: u16,
) {
    let source = match decode_with_fallback(&data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to decode audio: {}", e);
            return;
        }
    };

    let actual_duration = source
        .total_duration()
        .map(|d| d.as_secs())
        .unwrap_or(duration_secs);
    ctx.state.duration.store(actual_duration, Ordering::SeqCst);

    let norm_settings = ctx
        .settings
        .lock()
        .ok()
        .filter(|s| s.normalization_enabled)
        .map(|s| s.normalization_target_lufs);

    let (normalization, gain_atomic) = if let Some(target_lufs) = norm_settings {
        let rg_gain =
            extract_replaygain(&data).map(|rg| calculate_gain_factor(&rg, target_lufs));
        let atomic = Arc::new(AtomicU32::new(rg_gain.unwrap_or(1.0).to_bits()));

        if let Some(cached) = ctx.loudness_cache.get(track_id) {
            let cached_gain = db_to_linear(cached.gain_db.min(6.0));
            atomic.store(cached_gain.to_bits(), Ordering::Relaxed);
            log::info!(
                "Normalization: cache hit for track {}, gain {:.4}",
                track_id,
                cached_gain
            );
        }

        let _ = ctx.analyzer_tx.try_send(AnalyzerMessage::NewTrack {
            track_id,
            sample_rate,
            channels,
            target_lufs,
            gain_atomic: atomic.clone(),
        });

        (rg_gain, Some(atomic))
    } else {
        (None, None)
    };

    ctx.current_normalization_gain = normalization;
    ctx.current_gain_atomic = gain_atomic.clone();
    ctx.state.set_normalization_gain(normalization);

    let source = crate::player::audio_thread::ctx_source::wrap_source(
        &ctx.diagnostic,
        &ctx.viz_tap,
        &ctx.analyzer_tx,
        &ctx.analyzer_enabled,
        source,
        normalization,
        gain_atomic,
    );
    if let Err(e) = engine.append(source) {
        log::error!("Failed to append source to engine: {}", e);
        return;
    }

    ctx.state.is_playing.store(true, Ordering::SeqCst);
    ctx.state.position.store(0, Ordering::SeqCst);
    ctx.state.current_track_id.store(track_id, Ordering::SeqCst);
    ctx.state.start_playback_timer(0);

    ctx.current_engine = Some(engine);
    log::info!(
        "Audio thread: playback started, duration: {}s, normalization: {}",
        actual_duration,
        normalization
            .map(|g| format!("{:.4}x", g))
            .unwrap_or_else(|| "off".to_string())
    );
}
