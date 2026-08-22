use super::super::super::*;
use super::normalization::compute_gain;
use super::stream_legacy::clear_streaming_state;

/// Build the incremental decoder, pre-skip to `start_position_secs` if
/// resuming, wrap + append the source, and flip shared state to "playing".
/// Returns `true` on success (caller should store `engine` back into
/// `ctx.current_engine`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn start_playback(
    ctx: &mut ThreadCtx,
    engine: &mut PlaybackEngine,
    source: Arc<BufferedMediaSource>,
    wait_elapsed: Duration,
    track_id: u64,
    sample_rate: u32,
    channels: u16,
    duration_secs: u64,
    start_position_secs: u64,
) -> bool {
    let incremental_source = match IncrementalStreamingSource::new(source.clone()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create incremental streaming source: {}", e);
            clear_streaming_state(ctx, "Failed to start the streaming decoder");
            return false;
        }
    };

    let actual_sr = incremental_source.get_sample_rate();
    let actual_ch = incremental_source.get_channels();
    if actual_sr != sample_rate || actual_ch != channels {
        log::warn!(
            "Streaming: detected format {}Hz/{}ch differs from expected {}Hz/{}ch",
            actual_sr, actual_ch, sample_rate, channels
        );
    }

    ctx.state.duration.store(duration_secs, Ordering::SeqCst);

    let (normalization, gain_atomic) = compute_gain(ctx, &source, track_id, sample_rate, channels);
    ctx.current_normalization_gain = normalization;
    ctx.current_gain_atomic = gain_atomic.clone();
    ctx.state.set_normalization_gain(normalization);

    let mut source_to_play: Box<dyn Source<Item = f32> + Send> = Box::new(incremental_source);

    // Eager pre-skip for session resume: decode-and-discard here so the
    // engine's first pull doesn't underrun for multi-second offsets.
    if start_position_secs > 0 {
        let target_samples: u64 = start_position_secs
            .saturating_mul(actual_sr as u64)
            .saturating_mul(actual_ch as u64);
        let skip_start = Instant::now();
        let mut skipped: u64 = 0;
        while skipped < target_samples {
            if source_to_play.next().is_none() {
                log::warn!(
                    "Resume: source ended before reaching {}s (pre-skipped {} samples)",
                    start_position_secs,
                    skipped
                );
                break;
            }
            skipped += 1;
        }
        log::info!(
            "Resume: pre-skipped {} samples ({}s) in {}ms",
            skipped,
            start_position_secs,
            skip_start.elapsed().as_millis()
        );
    }

    let source_to_play = crate::player::audio_thread::ctx_source::wrap_source(
        &ctx.diagnostic,
        &ctx.viz_tap,
        &ctx.analyzer_tx,
        &ctx.analyzer_enabled,
        source_to_play,
        normalization,
        gain_atomic,
    );
    if let Err(e) = engine.append(source_to_play) {
        log::error!("Failed to append streaming source to engine: {}", e);
        return false;
    }

    ctx.state.set_dsd_mode(0);
    ctx.state.is_playing.store(true, Ordering::SeqCst);
    ctx.state.position.store(start_position_secs, Ordering::SeqCst);
    ctx.state.current_track_id.store(track_id, Ordering::SeqCst);
    ctx.state.start_playback_timer(start_position_secs);

    log::info!(
        "Audio thread: streaming playback STARTED in {}ms at {}s (incremental decode active)",
        wait_elapsed.as_millis(),
        start_position_secs
    );
    true
}
