//! The cached cap + the rate→tier mapping.

use std::sync::RwLock;

use qbz_models::Quality;

/// The cached cap. `tier` is the coarse Qobuz-tier mapping of the detected
/// ceiling (`max_rate_hz`); `detected` false = the probe fell back to the
/// common rate set, so the Settings caveat must disclose that the cap may
/// not match the hardware (owner Decision B: it still applies).
#[derive(Clone)]
pub(super) struct CapState {
    pub tier: Quality,
    pub detected: bool,
    pub max_rate_hz: u32,
    pub description: String,
}

/// None = the cap is disabled (toggle off) or not refreshed yet.
pub(super) static CAP: RwLock<Option<CapState>> = RwLock::new(None);

/// Map a detected max rate to the tier we may REQUEST (spec C.3). Coarse by
/// design: Qobuz sells four discrete tiers and no 48 kHz tier exists, so a
/// 48 kHz (or 44.1 kHz) ceiling steps down to CD 16/44.1 — bit depth
/// included; the Settings summary says it plainly instead of letting the
/// user discover it. > 96 kHz keeps Hi-Res+ = no effective cap (still
/// cached so Settings can display what was detected).
pub(super) fn tier_for_max_rate_hz(max_hz: u32) -> Quality {
    if max_hz > 96_000 {
        Quality::UltraHiRes
    } else if max_hz >= 88_200 {
        Quality::HiRes
    } else {
        Quality::Lossless
    }
}

/// Cheap read for the request-time resolution: `(tier, detected)`.
/// None = no cap configured.
pub fn cap() -> Option<(Quality, bool)> {
    CAP.read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|c| (c.tier, c.detected))
}
