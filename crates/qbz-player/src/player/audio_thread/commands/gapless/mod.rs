use super::super::*;

mod normalization;

/// Handle `AudioCommand::PlayNext`: append (or crossfade to) the next
/// track's decoded audio on the existing engine for a gapless transition.
pub(crate) fn handle(ctx: &mut ThreadCtx, data: Vec<u8>, track_id: u64, sample_rate: u32, channels: u16) {
    if ctx.current_engine.is_none() {
        normalization::preanalyze_only(ctx, &data, track_id);
        log::warn!(
            "Gapless: no engine, ignoring PlayNext for track {}",
            track_id
        );
        ctx.state.set_gapless_ready(false);
        return;
    }

    if let (Some(cur_sr), Some(cur_ch)) =
        (ctx.current_track_sample_rate, ctx.current_track_channels)
    {
        if sample_rate != cur_sr || channels != cur_ch {
            // Le titre sera demarre par le chemin streaming : on le mesure
            // quand meme maintenant, pour qu'il parte au bon volume.
            normalization::preanalyze_only(ctx, &data, track_id);
            log::info!(
                "Gapless: format mismatch (current {}Hz/{}ch vs next {}Hz/{}ch), ignoring PlayNext for track {}",
                cur_sr, cur_ch, sample_rate, channels, track_id
            );
            ctx.state.set_gapless_ready(false);
            return;
        }
    }

    if ctx.current_streaming_source.is_some() {
        normalization::preanalyze_only(ctx, &data, track_id);
        log::info!(
            "Gapless: streaming source active, ignoring PlayNext for track {}",
            track_id
        );
        ctx.state.set_gapless_ready(false);
        return;
    }

    let source = match decode_with_fallback(&data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Gapless: failed to decode track {}: {}", track_id, e);
            ctx.state.set_gapless_ready(false);
            return;
        }
    };
    let actual_duration = source.total_duration().map(|d| d.as_secs()).unwrap_or(0);

    // Crossfade duration (0 = off -> strict gapless append). Rodio-only —
    // engines without a `Mixer` silently keep gapless regardless of this
    // setting (see PlaybackEngine::supports_crossfade).
    let crossfade_secs = ctx
        .settings
        .lock()
        .ok()
        .map(|s| s.crossfade_seconds)
        .unwrap_or(0.0);

    let (normalization_gain, gain_atomic, pending_normalization) =
        normalization::prepare(ctx, &data, track_id, sample_rate, channels);
    let source = crate::player::audio_thread::ctx_source::wrap_source(
        &ctx.diagnostic,
        &ctx.viz_tap,
        &ctx.analyzer_tx,
        &ctx.analyzer_enabled,
        source,
        normalization_gain,
        gain_atomic,
    );

    // Capture emptiness BEFORE the append: a user-paused engine still holds
    // its current source (NOT empty), so a paused session is never resumed
    // by this (late-gapless race — see append_next for the full story).
    let engine = ctx.current_engine.as_mut().expect("checked Some above");
    let engine_was_empty = engine.empty();
    let do_crossfade = crossfade_secs > 0.0 && !engine_was_empty && engine.supports_crossfade();
    let append_result = if do_crossfade {
        engine.crossfade_to(source, std::time::Duration::from_secs_f32(crossfade_secs))
    } else {
        engine.append(source)
    };
    if let Err(e) = append_result {
        log::error!(
            "Gapless: failed to {} track {} to engine: {}",
            if do_crossfade { "crossfade" } else { "append" },
            track_id,
            e
        );
        ctx.state.set_gapless_ready(false);
        return;
    }
    if engine_was_empty && !ctx.state.is_playing.load(Ordering::SeqCst) {
        log::info!(
            "Gapless: PlayNext landed after track finished — resuming playback-state tracking"
        );
        ctx.state.is_playing.store(true, Ordering::SeqCst);
    }

    ctx.gapless_pending = Some(GaplessPending {
        track_id,
        duration_secs: actual_duration,
        data,
        normalization_gain,
        normalization: pending_normalization,
    });
    ctx.state.set_gapless_next_track_id(track_id);
    ctx.state.set_gapless_ready(false);

    log::info!(
        "Gapless: queued track {} (duration: {}s) for seamless transition",
        track_id,
        actual_duration
    );
}
