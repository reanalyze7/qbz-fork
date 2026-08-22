use super::super::super::*;
use super::stream_legacy::clear_streaming_state;

/// Wait for the initial download buffer (and, on session resume, enough
/// buffer to cover the resume offset). Returns the elapsed wait time on
/// success, or `None` when the caller should bail (already logged/recorded,
/// or the play was superseded and should quietly abandon).
#[allow(clippy::too_many_arguments)]
pub(crate) fn wait_for_buffer(
    ctx: &mut ThreadCtx,
    source: &Arc<BufferedMediaSource>,
    duration_secs: u64,
    content_length: u64,
    start_position_secs: u64,
    play_gen: u64,
    track_id: u64,
) -> Option<Duration> {
    log::info!("Streaming: waiting for initial buffer...");
    let start_wait = Instant::now();
    let max_wait = Duration::from_secs(60);

    let bytes_per_sec_estimate: u64 = if duration_secs > 0 && content_length > 0 {
        content_length / duration_secs
    } else {
        200_000
    };
    let resume_buffer_target: u64 = if start_position_secs > 0 {
        bytes_per_sec_estimate.saturating_mul(start_position_secs.saturating_add(8))
    } else {
        0
    };

    let buffer_sufficient = |src: &Arc<BufferedMediaSource>| -> bool {
        if !src.has_min_buffer() {
            return false;
        }
        if resume_buffer_target == 0 {
            return true;
        }
        (src.buffer_size() as u64) >= resume_buffer_target
    };

    while !buffer_sufficient(source)
        && source.download_error().is_none()
        && start_wait.elapsed() < max_wait
        && ctx.state.is_current_play(play_gen)
    {
        std::thread::sleep(Duration::from_millis(50));
    }

    // A newer play intent superseded this one — bail quietly (#591).
    if !ctx.state.is_current_play(play_gen) {
        log::info!(
            "Streaming: play of track {} superseded after {}ms buffer wait — abandoning",
            track_id,
            start_wait.elapsed().as_millis()
        );
        return None;
    }

    if !source.has_min_buffer() {
        let err_msg = match source.download_error() {
            Some(err) => {
                log::error!("Streaming: feeder failed before initial buffer: {}", err);
                format!("Stream feeder failed before buffering: {err}")
            }
            None => {
                log::error!("Streaming: timeout waiting for initial buffer");
                "Timed out waiting for the stream buffer to fill".to_string()
            }
        };
        clear_streaming_state(ctx, err_msg);
        return None;
    }
    if resume_buffer_target > 0 && (source.buffer_size() as u64) < resume_buffer_target {
        log::warn!(
            "Streaming: timed out waiting for resume buffer (got {} bytes, wanted {}); pre-skip may underrun briefly",
            source.buffer_size(),
            resume_buffer_target
        );
    }

    log::info!(
        "Streaming: buffer ready in {}ms ({} bytes, target {}), creating incremental decoder...",
        start_wait.elapsed().as_millis(),
        source.buffer_size(),
        resume_buffer_target
    );

    Some(start_wait.elapsed())
}
