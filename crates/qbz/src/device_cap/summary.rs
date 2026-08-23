//! Settings summary-line formatting.

use qbz_models::Quality;

use super::state::CAP;

/// The Settings "Detected device limit" value line: `(summary, detected)`,
/// e.g. `("192 kHz · Hi-Res+", true)`. Untranslated data composition — the
/// same convention as the quality badge's "24-bit / 96 kHz" (tier names are
/// product names). `("", true)` when no cap is active: the row hides on the
/// empty summary, and `true` keeps the fallback caveat from flashing before
/// the first refresh lands.
pub fn summary() -> (String, bool) {
    match CAP.read().unwrap_or_else(|e| e.into_inner()).as_ref() {
        Some(c) => (
            format!(
                "{} · {}",
                rate_khz_label(c.max_rate_hz),
                tier_display(c.tier)
            ),
            c.detected,
        ),
        None => (String::new(), true),
    }
}

/// Product-name tier label for the summary line. The CD entry spells out the
/// bit-depth cost (spec C.3: no 48 kHz tier exists, so the step below Hi-Res
/// loses depth too — say it, don't let the user discover it).
pub(super) fn tier_display(tier: Quality) -> &'static str {
    match tier {
        Quality::UltraHiRes => "Hi-Res+",
        Quality::HiRes => "Hi-Res",
        Quality::Lossless => "CD 16-bit / 44.1 kHz",
        // Unreachable from tier_for_max_rate_hz; total match for safety.
        Quality::Mp3 => "MP3 320",
    }
}

/// "192 kHz" / "44.1 kHz" from Hz — integer when whole, one decimal
/// otherwise (matches `crate::quality::detail`'s rate formatting).
pub(super) fn rate_khz_label(hz: u32) -> String {
    let khz = hz as f64 / 1000.0;
    if khz.fract().abs() < f64::EPSILON {
        format!("{} kHz", khz as i64)
    } else {
        format!("{khz} kHz")
    }
}
