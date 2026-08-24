use crate::*;

// Auto-detect renderer tier ladder: check whether the previous start with
// "auto" left the sentinel armed (a rung failed to reach a usable state)
// and continue down the ladder, otherwise detect fresh. Split out of
// `renderer_tier_from_prefs` (part8.rs) to stay under the 130-line file
// cap — pure extraction of the `key == "auto"` branch.
pub(crate) fn renderer_tier_auto(mut prefs: crate::ui_prefs::UiPrefs) -> (RendererTier, String) {
        if renderer_sentinel_armed() {
            let attempted = renderer_sentinel_value();
            clear_renderer_sentinel();
            if !cfg!(target_os = "macos") {
                match attempted.as_deref() {
                    // Rung 2: the default-preference adapter died before the
                    // app became usable. Retry wgpu on the OPPOSITE
                    // PowerPreference before surrendering the GPU tier.
                    Some("auto-wgpu") => {
                        log::warn!(
                            "[renderer] auto wgpu (default adapter) never reached a usable \
                             state -> retrying wgpu on the alternate adapter"
                        );
                        WGPU_ALT_ADAPTER.store(true, std::sync::atomic::Ordering::Relaxed);
                        arm_renderer_sentinel("auto-wgpu-alt");
                        return (
                            RendererTier::Wgpu,
                            "auto-detect (alternate wgpu adapter after a failed start)"
                                .to_string(),
                        );
                    }
                    // Rung 3: both wgpu adapters failed -> persist GL
                    // (compatibility), version-stamped for the re-probe.
                    Some("auto-wgpu-alt") => {
                        log::warn!(
                            "[renderer] both wgpu adapters failed to start -> persisting \
                             the GL (compatibility) renderer"
                        );
                        prefs.renderer = "gl".to_string();
                        prefs.renderer_auto_degraded = env!("CARGO_PKG_VERSION").to_string();
                        crate::ui_prefs::save(&prefs);
                        RENDERER_DEGRADED.store(true, std::sync::atomic::Ordering::Relaxed);
                        arm_renderer_sentinel("auto-gl");
                        return (
                            RendererTier::FemtovgGl,
                            "Settings (gl — both wgpu adapters failed to start)".to_string(),
                        );
                    }
                    // Rung 4: even GL died (broken EGL stacks exist) ->
                    // software, the floor that cannot fail.
                    Some("auto-gl") => {
                        log::warn!(
                            "[renderer] the GL renderer also failed to start -> persisting \
                             the software renderer"
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
                    _ => {}
                }
            }
        }
        let tier = detect_hardware_gpu();
        // Every fallible auto rung is sentinel-armed (GL stacks can be as
        // broken as wgpu ones); a known-bad default adapter goes straight
        // to the alt rung.
        arm_auto_tier(tier, &prefs);
        return (tier, "auto-detect".to_string());
}
