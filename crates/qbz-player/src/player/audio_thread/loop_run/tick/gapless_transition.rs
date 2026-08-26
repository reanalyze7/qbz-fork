use super::super::super::*;

fn swap_to_pending(ctx: &mut ThreadCtx) {
    let Some(pending) = ctx.gapless_pending.take() else {
        return;
    };
    // Clear gapless slot markers FIRST so a racing reader (the frontend
    // polling loop reads track_id / gapless_next_track_id as separate
    // atomics) never sees the inconsistent track_id-changed +
    // slot-still-set combination.
    ctx.state.set_gapless_next_track_id(0);
    ctx.state.set_gapless_ready(false);
    ctx.state
        .current_track_id
        .store(pending.track_id, Ordering::SeqCst);
    ctx.state
        .duration
        .store(pending.duration_secs, Ordering::SeqCst);
    ctx.state.start_playback_timer(0);
    ctx.current_audio_data = Some(pending.data);
    ctx.current_normalization_gain = pending.normalization_gain;
    ctx.state.set_normalization_gain(pending.normalization_gain);
    ctx.gapless_request_armed = false;

    // L'analyseur ne bascule que MAINTENANT. L'armer au prefetch revenait a
    // mesurer ce morceau sur la fin du precedent (souvent un fondu), donc a
    // lui attribuer un volume calcule sur du quasi-silence.
    if let Some(norm) = pending.normalization {
        norm.started.store(true, Ordering::SeqCst);
        let _ = ctx.analyzer_tx.try_send(AnalyzerMessage::NewTrack {
            track_id: pending.track_id,
            sample_rate: norm.sample_rate,
            channels: norm.channels,
            target_lufs: norm.target_lufs,
            gain_atomic: norm.gain_atomic.clone(),
        });
        ctx.current_gain_atomic = Some(norm.gain_atomic);
    }
}

/// Detect and apply a gapless transition, either by position (pos >= dur)
/// or, for ALSA Direct, via the writer thread's atomic transition flag.
/// Returns `true` if a transition was consumed this tick.
pub(super) fn check(ctx: &mut ThreadCtx, pos: u64, dur: u64) -> bool {
    if dur > 0 && pos >= dur {
        if let Some(ref pending) = ctx.gapless_pending {
            log::info!(
                "Gapless transition: track {} -> {} (pos {}s >= dur {}s)",
                ctx.state.current_track_id.load(Ordering::SeqCst),
                pending.track_id,
                pos,
                dur
            );
            swap_to_pending(ctx);
            return true;
        }
    }

    let alsa_transition = ctx
        .current_engine
        .as_ref()
        .map(|e| e.take_source_transition())
        .unwrap_or(false);
    if alsa_transition {
        if let Some(ref pending) = ctx.gapless_pending {
            log::info!(
                "ALSA Direct gapless transition: track {} -> {}",
                ctx.state.current_track_id.load(Ordering::SeqCst),
                pending.track_id
            );
            swap_to_pending(ctx);
            return true;
        }
    }

    false
}
