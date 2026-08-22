use super::super::*;

mod engine;
mod finish;
mod stream;
mod stream_legacy;

/// Handle `AudioCommand::Play`: (re)create the output stream if the format
/// changed, build a fresh `PlaybackEngine`, decode + normalize the data, and
/// start playback.
pub(crate) fn handle(
    ctx: &mut ThreadCtx,
    data: Vec<u8>,
    track_id: u64,
    duration_secs: u64,
    sample_rate: u32,
    channels: u16,
) {
    log::info!(
        "Audio thread: playing track {} ({}Hz, {} channels)",
        track_id,
        sample_rate,
        channels
    );
    ctx.pause_suspend_deadline = None;
    ctx.state.set_dsd_mode(0);
    // Clear any pending gapless state (new Play supersedes queued gapless)
    ctx.gapless_pending = None;
    ctx.gapless_request_armed = false;
    ctx.state.set_gapless_ready(false);
    ctx.state.set_gapless_next_track_id(0);

    if stream::ensure_stream(ctx, sample_rate, channels).is_err() {
        return;
    }

    ctx.current_track_sample_rate = Some(sample_rate);
    ctx.current_track_channels = Some(channels);

    // Stop previous engine and wait for sink to release resources.
    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
        #[cfg(target_os = "linux")]
        std::thread::sleep(Duration::from_millis(50));
    }

    ctx.current_audio_data = Some(data.clone());
    ctx.current_streaming_source = None;
    ctx.state.set_loaded_audio(true);

    let Some(engine) = engine::build_engine(ctx) else {
        return;
    };

    let volume = f32::from_bits(ctx.state.volume.load(Ordering::SeqCst));
    apply_engine_volume(&ctx.stream_opt, &engine, volume);

    finish::decode_and_start(ctx, engine, data, track_id, duration_secs, sample_rate, channels);
}
