use super::super::*;

use crate::player::offline_loudness::OfflineJob;

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

/// Mesure hors-ligne du morceau courant, pour le cache uniquement.
fn preanalyze_current(ctx: &ThreadCtx, data: &[u8]) {
    let Some(target) = ctx
        .settings
        .lock()
        .ok()
        .filter(|s| s.normalization_enabled)
        .map(|s| s.normalization_target_lufs)
    else {
        return;
    };
    let track_id = ctx.state.current_track_id.load(Ordering::SeqCst);
    if track_id == 0 || ctx.loudness_cache.has(track_id) {
        return;
    }
    ctx.offline_loudness.submit(OfflineJob::cache_only(
        track_id,
        std::sync::Arc::new(data.to_vec()),
        target,
    ));
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
                    // Morceau entier disponible : mesure hors-ligne pour le
                    // cache. Elle n'est pas appliquee a la lecture en cours
                    // (pas de changement de volume en plein morceau), mais la
                    // prochaine ecoute partira au bon niveau.
                    preanalyze_current(ctx, &full_data);
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
