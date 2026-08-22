use super::super::super::*;
use super::rebuild_setup::{build_resume_engine, fetch_resume_audio_data};

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
