//! Delivered-vs-catalog quality badge classification, shared by
//! `refresh_now_playing_meta` and the poll loop's per-tick UI push.

use qbz_models::{Quality, QualityLimit};

/// The #590 downgrade arithmetic (#638 fix 1). PRESERVED
/// EXACTLY: the 0.9 rate guard avoids flagging 44.1-vs-48 kHz family
/// mismatches (Tauri QualityBadge.svelte parity), and DSD (nominal 1-bit,
/// on either side) is exempt ENTIRELY, not just from the depth arm — past
/// DSD Phase 1 the DoP/native paths play DSD bit-perfect, and its
/// carrier/PCM rate vs the DSD "max" is apples-to-oranges, so any compare
/// would flag a false downgrade arrow on a bit-perfect DSD stream.
pub(crate) fn stream_downgraded(
    eff_rate_hz: u32,
    eff_bits: u32,
    max_rate_hz: u32,
    max_bits: u32,
) -> bool {
    let dsd = max_bits == 1 || eff_bits == 1;
    !dsd
        && ((eff_rate_hz > 0 && max_rate_hz > 0 && (eff_rate_hz as f64) < max_rate_hz as f64 * 0.9)
            || (eff_bits > 0 && max_bits > 0 && eff_bits < max_bits))
}

/// Display-time classification of WHY the delivered stream is below the
/// catalog max (#638 fix 1). `requested_id` / `request_cause` are the raw
/// values of `REQUESTED_QUALITY_ID` / `REQUESTED_CAUSE` (or the cast path's
/// request-time resolution); the return value is a `QualityLimit`
/// discriminant for the Slint `quality-limit-cause` property. Rules:
/// - not downgraded, or not a governed source (`requested_id` 0) → None;
/// - requested Hi-Res+ (no cap was in play) → Qobuz simply had no more
///   (CatalogAvailability) — this early-out is why no >96 kHz promise
///   check is needed here;
/// - delivered meets the requested tier's promise (F24-style: Hi-Res needs
///   24-bit, CD/MP3 are always met) → the request-time cause (the cap did
///   this);
/// - delivered below even the requested promise → Qobuz did not offer the
///   requested tier (CatalogAvailability).
pub(crate) fn classify_limit_cause(
    downgraded: bool,
    requested_id: u32,
    request_cause: i32,
    eff_bits: u32,
) -> i32 {
    if !downgraded {
        return QualityLimit::None as i32;
    }
    let Some(requested) = Quality::from_id(requested_id) else {
        return QualityLimit::None as i32;
    };
    let promise_met = match requested {
        Quality::UltraHiRes => return QualityLimit::CatalogAvailability as i32,
        Quality::HiRes => eff_bits >= 24,
        Quality::Lossless | Quality::Mp3 => true,
    };
    if promise_met {
        request_cause
    } else {
        QualityLimit::CatalogAvailability as i32
    }
}

/// Delivered-tier string for the badge's main line while downgraded
/// ("hires" | "cd" | "mp3"; "" = not downgraded → the badge shows the
/// catalog tier). Derived from the delivered bit depth PLUS the requested
/// tier so an MP3-capped stream (which decodes to 16-bit PCM) is labeled
/// MP3, not mislabeled CD.
pub(crate) fn delivered_tier_str(downgraded: bool, requested_id: u32, eff_bits: u32) -> &'static str {
    if !downgraded {
        return "";
    }
    if requested_id == Quality::Mp3.id() {
        return "mp3";
    }
    if eff_bits >= 24 {
        "hires"
    } else {
        "cd"
    }
}
