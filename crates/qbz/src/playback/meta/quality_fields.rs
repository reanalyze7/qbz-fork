//! Catalog quality resolution for the current track: bit depth / sample
//! rate (with F25 hydration fallback), the badge tier/detail strings, and
//! the REQUESTED-tier bookkeeping the poll loop's downgrade badge reads.

use super::statics::hydrated_catalog_quality;
use super::super::quality::local_playback_quality;
use super::super::state::{REQUESTED_CAUSE, REQUESTED_QUALITY_ID, TRACK_MAX_BITS, TRACK_MAX_RATE_HZ};
use qbz_models::{QualityLimit, QueueTrack};

/// Resolved catalog quality fields for one track, plus whether the
/// streaming-quality preference governs it (Qobuz-sourced, non-local).
pub(super) struct QualityFields {
    pub(super) bit_depth: Option<u32>,
    pub(super) sample_rate: Option<f64>,
    pub(super) quality_tier: &'static str,
    pub(super) quality_detail: String,
    pub(super) governed: bool,
}

/// Resolve the badge fields for `track` and cache the catalog max +
/// REQUESTED tier into the cross-cutting statics the poll loop compares
/// the DELIVERED stream against every tick (#590 follow-up, #638 fix 1).
pub(super) fn resolve_quality(track: &QueueTrack) -> QualityFields {
    // F25 (#638 fix 1c): search-queued tracks carry NO catalog params
    // (`track_item_to_queue` leaves both fields None), which used to render
    // the formatter DEFAULTS — "16-bit / 44.1 kHz" under a HI-RES tier — and
    // zeroed TRACK_MAX_* so the downgrade arrow could never fire there.
    // Adopt the async hydration's cached values when it already ran for this
    // track; until it lands the detail stays EMPTY (never a guess).
    let (bit_depth, sample_rate) = if track.bit_depth.is_none() && track.sample_rate.is_none() {
        hydrated_catalog_quality(track.id)
    } else {
        (track.bit_depth, track.sample_rate)
    };
    let quality_tier = match bit_depth {
        Some(1) => "hires",
        Some(d) if d >= 24 => "hires",
        Some(_) => "cd",
        None if track.hires => "hires",
        None => "",
    };
    let quality_detail = if quality_tier.is_empty() || (bit_depth.is_none() && sample_rate.is_none()) {
        // Empty tier, or a known tier with unknown params (pre-hydration
        // search play): an empty detail beats the guessed one (F25).
        String::new()
    } else if bit_depth == Some(1) {
        crate::quality::dsd_multiple_label(sample_rate)
    } else {
        crate::quality::detail(bit_depth, sample_rate)
    };
    // Cache the catalog max for the poll loop's downgrade detection (#590
    // follow-up). Rate normalized to Hz exactly like the sample-rate-hz push
    // below (`sample_rate` is Hz when >= 1000, else kHz); 0 = unknown.
    TRACK_MAX_RATE_HZ.store(
        sample_rate.map_or(0, |sr| if sr >= 1000.0 { sr as u32 } else { (sr * 1000.0) as u32 }),
        std::sync::atomic::Ordering::Relaxed,
    );
    TRACK_MAX_BITS.store(bit_depth.unwrap_or(0), std::sync::atomic::Ordering::Relaxed);

    // REQUESTED tier + request-time cause for the badge's WHY line (#638
    // fix 1), resolved ONCE per track change beside the TRACK_MAX stores
    // (ui_prefs::load is a disk read + JSON parse — never per poll tick).
    // Only Qobuz-sourced tracks are governed by the streaming-quality
    // preference; local / ephemeral sources store 0 = not governed,
    // which keeps the cause line off for them. The device-capped resolve
    // (#638 fix 3) names the output device when its cap — not the
    // preference — shaped the request, so the tooltip can say which.
    // Mirrors the normalized `source` string built alongside the basic
    // fields (a None source defaults to "local"/"qobuz" by `is_local`).
    let source_default = track
        .source
        .as_deref()
        .unwrap_or(if track.is_local { "local" } else { "qobuz" });
    let governed = !track.is_local && matches!(source_default, "qobuz" | "qobuz_download");
    if governed {
        let (requested, cause) = local_playback_quality();
        REQUESTED_QUALITY_ID.store(requested.id(), std::sync::atomic::Ordering::Relaxed);
        REQUESTED_CAUSE.store(cause as i32, std::sync::atomic::Ordering::Relaxed);
    } else {
        REQUESTED_QUALITY_ID.store(0, std::sync::atomic::Ordering::Relaxed);
        REQUESTED_CAUSE.store(QualityLimit::None as i32, std::sync::atomic::Ordering::Relaxed);
    }

    QualityFields {
        bit_depth,
        sample_rate,
        quality_tier,
        quality_detail,
        governed,
    }
}
