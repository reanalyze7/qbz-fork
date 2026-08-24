//! The dirty-guarded per-tick `NowPlayingState` push: position/progress plus
//! the delivered-vs-catalog quality-downgrade badge fields.
use slint::ComponentHandle;

use super::state::PollLoopState;
use super::super::meta::{classify_limit_cause, delivered_tier_str, stream_downgraded};
use super::super::quality::{fmt_elapsed, fmt_remaining, set_viz_paused};
use super::super::state::{REQUESTED_CAUSE, REQUESTED_QUALITY_ID, TRACK_MAX_BITS, TRACK_MAX_RATE_HZ};
use super::super::Runtime;
use crate::{AppWindow, NowPlayingState};

/// Push the live values onto NowPlayingState — but only when something the
/// push depends on actually changed (the dirty-guard in `state.last_ui_push`).
/// While playing, `position` advances, so pushes proceed; fully idle
/// (track_id == 0, nothing playing) the UI hop is skipped entirely and the
/// window stays clean.
#[allow(clippy::too_many_arguments)]
pub(super) fn maybe_push(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    state: &mut PollLoopState,
    track_id: u64,
    position: u64,
    duration: u64,
    is_playing: bool,
    volume: f32,
    cache: f32,
    seekable_max: f32,
    eff_rate_hz: u32,
    eff_bits: u32,
) {
    let ui_snapshot = (
        track_id,
        position,
        duration,
        is_playing,
        volume.to_bits(),
        cache.to_bits(),
        seekable_max.to_bits(),
        eff_rate_hz,
        eff_bits,
    );
    if state.last_ui_push == Some(ui_snapshot) {
        return;
    }
    state.last_ui_push = Some(ui_snapshot);
    // Effective-vs-max quality (#590 follow-up, reshaped by #638
    // fix 1): when the DELIVERED stream is below the catalog max,
    // the badge's main line flips to the delivered tier/detail
    // (owner decision — restores the Tauri behavior; the catalog
    // max moves to the tooltip's "Source" line), the amber arrow
    // turns on, and the tooltip names the CAUSE. The downgrade
    // arithmetic (0.9 rate-family guard + full DSD exemption)
    // lives in `stream_downgraded`.
    let max_rate_hz = TRACK_MAX_RATE_HZ.load(std::sync::atomic::Ordering::Relaxed);
    let max_bits = TRACK_MAX_BITS.load(std::sync::atomic::Ordering::Relaxed);
    let downgraded = stream_downgraded(eff_rate_hz, eff_bits, max_rate_hz, max_bits);
    let requested_id = REQUESTED_QUALITY_ID.load(std::sync::atomic::Ordering::Relaxed);
    let request_cause = REQUESTED_CAUSE.load(std::sync::atomic::Ordering::Relaxed);
    let limit_cause = classify_limit_cause(downgraded, requested_id, request_cause, eff_bits);
    let delivered_tier = delivered_tier_str(downgraded, requested_id, eff_bits);
    // True delivered line, via the shared formatter so it matches
    // the badge style ("16-bit / 44.1 kHz"). Native DSD streams
    // (1-bit) go through the DSD label instead — the generic
    // detail would read "1-bit / 2822.4 kHz".
    let true_detail = if eff_bits == 1 {
        crate::quality::dsd_multiple_label((eff_rate_hz > 0).then_some(eff_rate_hz as f64))
    } else if eff_rate_hz > 0 || eff_bits > 0 {
        crate::quality::detail((eff_bits > 0).then_some(eff_bits), (eff_rate_hz > 0).then_some(eff_rate_hz as f64))
    } else {
        String::new()
    };
    // Mirror engine truth onto the visualizer tap alongside the
    // set_playing push below. This is the catch-all: EVERY local
    // transition (pause/resume from any surface — MPRIS, tray,
    // hotkey, QConnect renderer command — plus stop, track end,
    // seek-while-paused snapshots) lands here within one 450ms
    // tick; the direct edge sites above only shave latency.
    set_viz_paused(runtime, !is_playing);
    let progress = if duration > 0 {
        (position as f32 / duration as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let elapsed = fmt_elapsed(position);
    let remaining = fmt_remaining(position, duration);
    let _ = weak.upgrade_in_event_loop(move |w| {
        let np = w.global::<NowPlayingState>();
        np.set_position_secs(position as i32);
        if duration > 0 {
            np.set_duration_secs(duration as i32);
        }
        np.set_progress(progress);
        np.set_cache(cache);
        np.set_seekable_max(seekable_max);
        np.set_elapsed(elapsed.into());
        np.set_remaining(remaining.into());
        np.set_playing(is_playing);
        np.set_volume(volume.clamp(0.0, 1.0));
        // Effective quality for the delivered-first badge line,
        // the downgrade arrow and the tooltip cause (#638 fix 1).
        np.set_effective_sample_rate_hz(eff_rate_hz as i32);
        np.set_effective_bit_depth(eff_bits as i32);
        np.set_quality_downgraded(downgraded);
        np.set_quality_true_detail(true_detail.into());
        np.set_quality_limit_cause(limit_cause);
        np.set_quality_effective_tier(delivered_tier.into());
    });
}
