use super::super::*;

mod buffer_wait;
mod engine;
mod normalization;
mod start;
mod stream;
mod stream_legacy;

/// Handle `AudioCommand::PlayStreaming`: (re)create the output stream, wait
/// for the initial buffer, build an incremental decoder, and start playback.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle(
    ctx: &mut ThreadCtx,
    source: Arc<BufferedMediaSource>,
    track_id: u64,
    sample_rate: u32,
    channels: u16,
    duration_secs: u64,
    start_position_secs: u64,
    content_length: u64,
    play_gen: u64,
) {
    log::info!(
        "Audio thread: starting streaming playback for track {} ({}Hz, {} channels, {}s, start={}s)",
        track_id, sample_rate, channels, duration_secs, start_position_secs
    );
    ctx.pause_suspend_deadline = None;
    ctx.current_streaming_source = Some(source.clone());
    ctx.current_audio_data = None;
    ctx.state.set_loaded_audio(true);
    ctx.state.duration.store(duration_secs, Ordering::SeqCst);

    if stream::ensure_stream(ctx, sample_rate, channels).is_err() {
        return;
    }

    ctx.current_track_sample_rate = Some(sample_rate);
    ctx.current_track_channels = Some(channels);

    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
        #[cfg(target_os = "linux")]
        std::thread::sleep(Duration::from_millis(50));
    }

    let Some(mut engine) = engine::build_engine(ctx) else {
        return;
    };

    let volume = f32::from_bits(ctx.state.volume.load(Ordering::SeqCst));
    apply_engine_volume(&ctx.stream_opt, &engine, volume);

    let Some(buffered) = buffer_wait::wait_for_buffer(
        ctx,
        &source,
        duration_secs,
        content_length,
        start_position_secs,
        play_gen,
        track_id,
    ) else {
        return;
    };

    let ok = start::start_playback(
        ctx,
        &mut engine,
        source,
        buffered,
        track_id,
        sample_rate,
        channels,
        duration_secs,
        start_position_secs,
    );
    if ok {
        ctx.current_engine = Some(engine);
    }
}
