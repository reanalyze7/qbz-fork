// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for.
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

