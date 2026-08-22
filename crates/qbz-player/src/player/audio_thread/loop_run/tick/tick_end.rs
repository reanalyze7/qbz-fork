use super::super::super::*;

/// Lead time before track end to request the next track (gapless
/// readiness). 10s covers most HiRes tracks even on the software-AES
/// fallback path for offline-cache v2 bundles (decrypt-bound, ~10 MB/s
/// without AES-NI). Crossfade duration widens the lead time further: the
/// overlap needs the next track's bytes decoded and ready BY the moment
/// the fade should start, not just by natural end.
const GAPLESS_LEAD_SECS: u64 = 10;

/// Signal the frontend to prepare the next track once we're within
/// `lead_secs` of the current track's end.
pub(super) fn maybe_request_next(ctx: &mut ThreadCtx, pos: u64, dur: u64, transition_consumed: bool) {
    let (gapless_enabled, crossfade_lead_secs) = ctx
        .settings
        .lock()
        .ok()
        .map(|s| (s.gapless_enabled, s.crossfade_seconds.ceil() as u64))
        .unwrap_or((false, 0));
    let lead_secs = GAPLESS_LEAD_SECS.max(crossfade_lead_secs);

    if gapless_enabled
        && !transition_consumed
        && dur > 0
        && pos + lead_secs >= dur
        && ctx.gapless_pending.is_none()
        && !ctx.gapless_request_armed
        && !ctx.state.is_gapless_ready()
        && ctx.state.get_gapless_next_track_id() == 0
        && ctx.current_streaming_source.is_none()
    {
        log::info!(
            "Gapless: approaching end of track ({}s/{}s), requesting next",
            pos,
            dur
        );
        ctx.state.set_gapless_ready(true);
        ctx.gapless_request_armed = true;
    }
}

/// Check whether the engine has drained all sources (track finished
/// naturally) and reset transport/gapless state if so.
pub(super) fn check_track_finished(ctx: &mut ThreadCtx) {
    let finished = ctx
        .current_engine
        .as_ref()
        .map(|e| e.empty() && ctx.state.is_playing.load(Ordering::SeqCst))
        .unwrap_or(false);
    if !finished {
        return;
    }

    log::info!("Audio thread: track finished (engine empty)");
    ctx.state.is_playing.store(false, Ordering::SeqCst);
    let duration = ctx.state.duration.load(Ordering::SeqCst);
    ctx.state.position.store(duration, Ordering::SeqCst);
    ctx.state.playback_start_millis.store(0, Ordering::SeqCst);
    ctx.state.set_gapless_ready(false);
    ctx.state.set_gapless_next_track_id(0);
    ctx.gapless_pending = None;
    ctx.gapless_request_armed = false;
}
