use super::super::*;

mod gapless_transition;
mod tick_end;

/// Periodic housekeeping run at most once per 500ms while playing (gated by
/// `ctx.last_empty_check`): buffer progress, streaming->cached promotion,
/// gapless transition detection, and the "track finished" check.
pub(super) fn on_timeout(ctx: &mut ThreadCtx) {
    let now = Instant::now();
    if now.duration_since(ctx.last_empty_check) < Duration::from_millis(500) {
        return;
    }
    ctx.last_empty_check = now;

    update_buffer_progress(ctx);
    promote_completed_streaming(ctx);

    let pos = ctx.state.current_position();
    let dur = ctx.state.duration.load(Ordering::SeqCst);

    // Track whether a gapless transition fired this tick so the
    // "approaching end" check below can skip itself — otherwise the stale
    // pos/dur snapshot just taken would immediately re-arm the request for
    // the NEW track and the real trigger would never fire again.
    let transition_consumed_pending = gapless_transition::check(ctx, pos, dur);

    tick_end::maybe_request_next(ctx, pos, dur, transition_consumed_pending);
    tick_end::check_track_finished(ctx);
}

fn update_buffer_progress(ctx: &mut ThreadCtx) {
    if let Some(streaming_src) = ctx.current_streaming_source.as_ref() {
        let progress = streaming_src.progress().unwrap_or(1.0);
        ctx.state.set_buffer_progress(progress);
    } else {
        ctx.state.set_buffer_progress(0.0);
    }
}

/// Once a streaming download completes, persist the full data and clear the
/// streaming marker — unlocks normal gapless pre-queue for the track's tail.
fn promote_completed_streaming(ctx: &mut ThreadCtx) {
    let mut clear_streaming_source = false;
    if let Some(streaming_src) = ctx.current_streaming_source.as_ref() {
        if streaming_src.is_complete() {
            if ctx.current_audio_data.is_none() {
                if let Some(full_data) = streaming_src.take_complete_data() {
                    log::info!(
                        "Streaming promotion: full track buffered ({} bytes), enabling cached transition path",
                        full_data.len()
                    );
                    ctx.current_audio_data = Some(full_data);
                }
            }
            clear_streaming_source = true;
        }
    }
    if clear_streaming_source {
        ctx.current_streaming_source = None;
    }
}
