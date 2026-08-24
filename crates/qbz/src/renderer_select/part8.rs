use crate::*;

/// Resolve the persisted Settings>Appearance renderer key ("auto" | "wgpu" |
/// "gl" | "software"). A non-auto override is wrapped in the startup sentinel:
/// armed here (before the risky backend/window init), disarmed on the first
/// real user input (or a fallback timer). If we find it still armed from the
/// PREVIOUS run, that run never reached a usable state with this override —
/// revert to "auto" and auto-detect, so users can't lock themselves out.
///
/// AUTO runs a degradation LADDER instead of a single fallback: each failed
/// start moves one rung down, and the first rung that survives wins.
///
///   wgpu (default adapter)  -> wgpu (opposite PowerPreference) -> GL -> software
///
/// The alternate-adapter rung exists because the #542 family is an ADAPTER
/// mixup, not a wgpu failure: on hybrid machines (mux laptops, desktops with
/// the monitor on the discrete card) the heuristically-preferred adapter may
/// be unable to present while the other one works perfectly — surrendering
/// the whole GPU tier over that would be wrong. macOS stays out of the
/// ladder by design (no GL tier there; it degrades back to wgpu).
///
/// Split into `renderer_tier_auto` / `renderer_tier_override` (this dir's
/// `renderer_tier_auto.rs` / `renderer_tier_override.rs`) for the two
/// mutually-exclusive branches (`key == "auto"` vs. a Settings override).
pub(crate) fn renderer_tier_from_prefs() -> (RendererTier, String) {
    let mut prefs = crate::ui_prefs::load();
    let mut key = prefs.renderer.clone();
    // A persisted AUTO-degradation is version-keyed: a new build re-probes
    // "auto" once (vendored renderer fixes / driver updates are likely since
    // the rung was recorded); the ladder re-degrades within one start if the
    // stack is still broken. User-chosen overrides carry no version marker
    // and are never re-probed.
    if key != "auto"
        && !prefs.renderer_auto_degraded.is_empty()
        && prefs.renderer_auto_degraded != env!("CARGO_PKG_VERSION")
    {
        log::info!(
            "[renderer] '{key}' was auto-degraded by version {} — re-probing auto once on {}",
            prefs.renderer_auto_degraded,
            env!("CARGO_PKG_VERSION")
        );
        prefs.renderer = "auto".to_string();
        prefs.renderer_auto_degraded.clear();
        crate::ui_prefs::save(&prefs);
        key = "auto".to_string();
    }
    if key == "auto" {
        return renderer_tier_auto(prefs);
    }
    renderer_tier_override(prefs, key)
}
