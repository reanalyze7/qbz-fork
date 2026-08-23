//! F25 quality hydration (#638 fix 1c): a search-queued Qobuz track has no
//! catalog quality fields, so the badge seed left them empty and
//! `TRACK_MAX_*` stayed 0 (the downgrade arrow could never fire). Fetch the
//! catalog track once per track id and re-seed the maxima + badge.

use super::statics::{HYDRATED_BITS, HYDRATED_RATE_HZ, HYDRATED_TRACK_ID};
use super::super::state::{FORCE_UI_REPUSH, TRACK_MAX_BITS, TRACK_MAX_RATE_HZ};
use super::super::Runtime;
use crate::{AppWindow, NowPlayingState};

/// Spawn the hydration fetch when `track_id_num` is governed by the
/// streaming-quality preference, carries no catalog quality fields, and has
/// not already been hydrated. No-op otherwise.
pub(super) fn spawn_quality_hydration(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    track_id_num: u64,
    governed: bool,
    missing_catalog_fields: bool,
) {
    if !(governed
        && missing_catalog_fields
        && HYDRATED_TRACK_ID.load(std::sync::atomic::Ordering::Relaxed) != track_id_num)
    {
        return;
    }
    let runtime = runtime.clone();
    let weak = weak.clone();
    tokio::spawn(async move {
        let fetched = match runtime.core().get_track(track_id_num).await {
            Ok(t) => t,
            Err(e) => {
                log::debug!("[qbz-slint] quality hydration: get_track {track_id_num} failed: {e}");
                return;
            }
        };
        if runtime.core().current_track().await.map(|t| t.id) != Some(track_id_num) {
            return;
        }
        let bits = fetched.maximum_bit_depth;
        let rate = fetched.maximum_sampling_rate;
        if bits.is_none() && rate.is_none() {
            return; // Nothing learned; leave the empty detail in place.
        }
        // Same Hz normalization as the meta seed (Qobuz reports kHz).
        let rate_hz = rate.map_or(0, |sr| if sr >= 1000.0 { sr as u32 } else { (sr * 1000.0) as u32 });
        // Values BEFORE the id, with the id store Released and paired
        // with the Acquire load in `hydrated_catalog_quality`, so a
        // reader keyed on the id never sees another track's params
        // (program order alone would not survive a weakly-ordered CPU —
        // macOS ARM is a shipped target).
        HYDRATED_RATE_HZ.store(rate_hz, std::sync::atomic::Ordering::Relaxed);
        HYDRATED_BITS.store(bits.unwrap_or(0), std::sync::atomic::Ordering::Relaxed);
        HYDRATED_TRACK_ID.store(track_id_num, std::sync::atomic::Ordering::Release);
        TRACK_MAX_RATE_HZ.store(rate_hz, std::sync::atomic::Ordering::Relaxed);
        TRACK_MAX_BITS.store(bits.unwrap_or(0), std::sync::atomic::Ordering::Relaxed);
        // Re-push the badge seed values the meta pass left empty. No DSD
        // arm: Qobuz catalog maxima are PCM (16/24-bit), never 1-bit.
        let quality_tier = match bits {
            Some(d) if d >= 24 => "hires",
            Some(_) => "cd",
            None if fetched.hires => "hires",
            None => "",
        };
        let quality_detail = if quality_tier.is_empty() {
            String::new()
        } else {
            crate::quality::detail(bits, rate)
        };
        let bits_push = bits.unwrap_or(0) as i32;
        let rate_hz_push = rate_hz as i32;
        let _ = weak.upgrade_in_event_loop(move |w| {
            let np = w.global::<NowPlayingState>();
            np.set_quality_tier(quality_tier.into());
            np.set_quality_detail(quality_detail.into());
            np.set_sample_rate_hz(rate_hz_push);
            np.set_bit_depth(bits_push);
        });
        // The maxima changed while the poll snapshot may be frozen
        // (paused): force the next tick to re-run the downgrade compare
        FORCE_UI_REPUSH.store(true, std::sync::atomic::Ordering::Relaxed);
    });
}
