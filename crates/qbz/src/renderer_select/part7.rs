use crate::*;

/// Resolve the renderer preference, with its human-readable source for the
/// diagnostics: QBZ_RENDERER env override first, then the persisted Settings
/// choice (guarded by the auto-revert sentinel), else auto-detect.
pub(crate) fn requested_renderer_tier() -> (RendererTier, String) {
    match std::env::var("QBZ_RENDERER")
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
    {
        Some(v) if matches!(v.as_str(), "software" | "cpu" | "soft") => {
            log::info!("[renderer] QBZ_RENDERER={v} -> forcing software renderer");
            (RendererTier::Software, format!("QBZ_RENDERER={v}"))
        }
        Some(v) if matches!(v.as_str(), "gpu" | "wgpu" | "hardware" | "hw") => {
            log::info!("[renderer] QBZ_RENDERER={v} -> forcing wgpu (GPU) renderer");
            (RendererTier::Wgpu, format!("QBZ_RENDERER={v}"))
        }
        Some(v) if matches!(v.as_str(), "gl" | "gles" | "femtovg") => {
            log::info!("[renderer] QBZ_RENDERER={v} -> forcing femtovg GL renderer");
            (RendererTier::FemtovgGl, format!("QBZ_RENDERER={v}"))
        }
        Some(v) if !v.is_empty() && v != "auto" => {
            log::warn!("[renderer] QBZ_RENDERER='{v}' unrecognized -> auto-detecting");
            (detect_hardware_gpu(), "auto-detect".to_string())
        }
        _ => renderer_tier_from_prefs(),
    }
}

