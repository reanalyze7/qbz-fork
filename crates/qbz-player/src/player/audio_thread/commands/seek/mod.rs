use super::super::*;

mod engine;
mod source;

/// True when a seek to `position_secs` should be allowed given how much of
/// a streaming source has been buffered so far.
fn streaming_watermark_ok(ctx: &ThreadCtx, position_secs: u64) -> bool {
    let Some(ref stream_src) = ctx.current_streaming_source else {
        return true;
    };
    if stream_src.is_complete() {
        return true;
    }
    // Approximate bytes-to-seconds mapping via download fraction x total
    // duration. Exact for CBR, close-enough for FLAC/VBR; the 0.90 margin
    // covers the error band so the decoder never reads past the watermark.
    let duration_secs = ctx.state.duration();
    let progress = stream_src.progress().unwrap_or(0.0);
    if duration_secs == 0 || progress <= 0.0 {
        log::warn!(
            "Audio thread: seek to {}s ignored — streaming progress unknown",
            position_secs
        );
        return false;
    }
    let max_seekable_secs = (progress * 0.90 * duration_secs as f32) as u64;
    if position_secs > max_seekable_secs {
        log::warn!(
            "Audio thread: seek to {}s ignored — past buffered watermark ({}s, progress {:.1}%)",
            position_secs,
            max_seekable_secs,
            progress * 100.0
        );
        return false;
    }
    log::info!(
        "Audio thread: seek to {}s within buffered zone (watermark {}s, progress {:.1}%)",
        position_secs,
        max_seekable_secs,
        progress * 100.0
    );
    true
}

/// Handle `AudioCommand::Seek`.
pub(crate) fn handle(ctx: &mut ThreadCtx, position_secs: u64) {
    if ctx.current_engine.as_ref().map(|e| e.is_dop()).unwrap_or(false) {
        // v1 limitation: no seek inside a DoP stream.
        log::info!("Seek ignored during DoP playback ({}s)", position_secs);
        return;
    }
    ctx.pause_suspend_deadline = None;
    ctx.gapless_pending = None;
    ctx.gapless_request_armed = false;
    ctx.state.set_gapless_ready(false);
    ctx.state.set_gapless_next_track_id(0);

    if ctx.current_audio_data.is_none() && ctx.current_streaming_source.is_none() {
        log::warn!("Audio thread: cannot seek - no audio data available");
        return;
    }
    if !streaming_watermark_ok(ctx, position_secs) {
        return;
    }
    if ctx.stream_opt.is_none() {
        log::error!("Audio thread: cannot seek - no audio device available");
        return;
    }

    log::info!("Audio thread: seeking to {}s", position_secs);

    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
    }

    let Some(mut new_engine) = engine::build_seek_engine(ctx) else {
        return;
    };

    let volume = f32::from_bits(ctx.state.volume.load(Ordering::SeqCst));
    apply_engine_volume(&ctx.stream_opt, &new_engine, volume);

    let Some(skipped_source) = source::build_skipped_source(ctx, position_secs) else {
        return;
    };

    let _ = ctx.analyzer_tx.try_send(AnalyzerMessage::Reset);
    let skipped_source = crate::player::audio_thread::ctx_source::wrap_source(
        &ctx.diagnostic,
        &ctx.viz_tap,
        &ctx.analyzer_tx,
        &ctx.analyzer_enabled,
        skipped_source,
        ctx.current_normalization_gain,
        ctx.current_gain_atomic.clone(),
    );
    if let Err(e) = new_engine.append(skipped_source) {
        engine::seek_abort(&ctx.state, &format!("append source for seek failed: {e}"));
        return;
    }

    let was_playing = ctx.state.is_playing.load(Ordering::SeqCst);
    if !was_playing {
        new_engine.pause();
    }

    ctx.state.position.store(position_secs, Ordering::SeqCst);
    if was_playing {
        ctx.state.start_playback_timer(position_secs);
    }

    ctx.current_engine = Some(new_engine);
    ctx.state.set_stream_error(false);
    log::info!(
        "Audio thread: seeked to {}s (was_playing: {})",
        position_secs,
        was_playing
    );
}
