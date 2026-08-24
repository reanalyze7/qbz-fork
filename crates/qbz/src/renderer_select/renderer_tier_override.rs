use crate::*;

// Settings-override renderer tier resolution: check whether the previous
// start with this override left the sentinel armed (never reached a usable
// state) and revert to auto if so, otherwise apply + arm the override.
// Split out of `renderer_tier_from_prefs` (part8.rs) to stay under the
// 130-line file cap — pure extraction of the non-"auto" branch.
pub(crate) fn renderer_tier_override(
    mut prefs: crate::ui_prefs::UiPrefs,
    key: String,
) -> (RendererTier, String) {
    if renderer_sentinel_armed() {
        let attempted = renderer_sentinel_value();
        clear_renderer_sentinel();
        // Ladder continuation (rung 4 via the persisted-"gl" route): rung 3
        // wrote prefs.renderer="gl" and armed "auto-gl", so its failure
        // surfaces HERE, not in the auto branch. A USER-picked gl arms the
        // literal "gl", never "auto-gl" — no ambiguity.
        if attempted.as_deref() == Some("auto-gl") && !cfg!(target_os = "macos") {
            log::warn!(
                "[renderer] the ladder-persisted GL renderer also failed to start -> \
                 persisting the software renderer"
            );
            prefs.renderer = "software".to_string();
            prefs.renderer_auto_degraded = env!("CARGO_PKG_VERSION").to_string();
            crate::ui_prefs::save(&prefs);
            RENDERER_DEGRADED.store(true, std::sync::atomic::Ordering::Relaxed);
            return (
                RendererTier::Software,
                "Settings (software — wgpu and GL failed to start)".to_string(),
            );
        }
        log::warn!(
            "[renderer] previous start with renderer='{key}' never reached a usable state \
             -> reverting the setting to auto"
        );
        prefs.renderer = "auto".to_string();
        prefs.renderer_auto_degraded.clear();
        crate::ui_prefs::save(&prefs);
        RENDERER_REVERTED.store(true, std::sync::atomic::Ordering::Relaxed);
        let tier = detect_hardware_gpu();
        // Arm the re-detected tier too — if IT also fails, the next start
        // continues down the ladder instead of looping.
        arm_auto_tier(tier, &prefs);
        return (tier, format!("auto-detect (reverted: '{key}' failed to start)"));
    }
    let tier = match key.as_str() {
        "wgpu" => RendererTier::Wgpu,
        "gl" => RendererTier::FemtovgGl,
        "software" => RendererTier::Software,
        other => {
            log::warn!("[renderer] unknown persisted renderer '{other}' -> auto-detecting");
            return (detect_hardware_gpu(), "auto-detect".to_string());
        }
    };
    // Software cannot fail to start — arming it would only invite false
    // reverts from no-input quick sessions.
    if tier != RendererTier::Software {
        log::info!("[renderer] Settings renderer override '{key}' (sentinel armed)");
        arm_renderer_sentinel(&key);
    } else {
        log::info!("[renderer] Settings renderer override 'software'");
    }
    (tier, format!("Settings ({key})"))
}
