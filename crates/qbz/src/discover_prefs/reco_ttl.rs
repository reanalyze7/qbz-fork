//! Recommendations cache-window (TTL) setting, plus the "show recommendations"
//! opt-out toggle.

use super::{current, persist, PREFS};
use crate::{AppWindow, ExternalRecoState};
use slint::ComponentHandle;

/// Opt-out toggle for the Discover "Recommendations" tab (Settings > Integrations).
/// The Slint `SettingsState.show-recommendations` global is updated by the toggle
/// itself; this writes the choice through to the persisted prefs.
pub fn set_show_recommendations(value: bool) {
    if let Some(p) = PREFS.lock().unwrap().as_mut() {
        p.show_recommendations = value;
    }
    persist();
}

/// The offered cache-window options, in hours. Index <-> hours mapping is the
/// contract with the `ExternalRecoState.cache-ttl-index` Slint global and the
/// QbzSelect option order (24 / 36 / 48 / 72 hours).
const TTL_HOURS: [i64; 4] = [24, 36, 48, 72];

/// Map a stored hour value to its select index (default index 2 == 48h).
pub(super) fn ttl_index_from_hours(h: i64) -> i32 {
    TTL_HOURS.iter().position(|&x| x == h).unwrap_or(2) as i32
}

/// Map a select index back to its hour value (clamped; default 48h).
fn ttl_hours_from_index(i: i32) -> i64 {
    *TTL_HOURS.get(i.clamp(0, 3) as usize).unwrap_or(&48)
}

/// Set the Recommendations cache window from the select index (0..=3). Mirrors
/// `set_show_recommendations`: mutate the in-memory authoritative prefs, persist,
/// then push the resolved index back into the `ExternalRecoState` global.
pub fn set_reco_cache_ttl_index(window: &AppWindow, index: i32) {
    let hours = ttl_hours_from_index(index);
    if let Some(p) = PREFS.lock().unwrap().as_mut() {
        p.reco_cache_ttl_hours = hours;
    }
    persist();
    window
        .global::<ExternalRecoState>()
        .set_cache_ttl_index(ttl_index_from_hours(hours));
}

/// The current Recommendations cache window expressed in SECONDS — consumed by
/// the external-reco loader's `get_results` TTL check.
pub fn reco_cache_ttl_secs() -> i64 {
    current().reco_cache_ttl_hours * 3600
}
