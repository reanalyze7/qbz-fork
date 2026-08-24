use crate::*;

pub(crate) fn arm_startup_probe() {
    let Some(path) = startup_probe_path() else { return };
    let prev: u8 = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let level = prev.saturating_add(1);
    CRASH_CHAIN_LEVEL.store(level, std::sync::atomic::Ordering::Relaxed);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&path, level.to_string()) {
        log::warn!("[crash-chain] could not arm the startup probe: {e}");
    }
    if level < 2 {
        return;
    }
    log::warn!(
        "[crash-chain] {} consecutive start(s) died before liveness — recovery level {level}",
        level - 1
    );
    // Level 2: the persisted view restore is the prime suspect — reset it so
    // the app starts on Home. This edits exactly last_nav/last_view; nothing
    // else in ui_prefs is touched.
    let mut prefs = crate::ui_prefs::load();
    if prefs.last_nav.as_deref() != Some("{}") || prefs.last_view != "home" {
        prefs.last_nav = Some("{}".to_string());
        prefs.last_view = "home".to_string();
        crate::ui_prefs::save(&prefs);
        log::warn!(
            "[crash-chain] reset persisted view restore: last_nav -> {{}}, last_view -> home"
        );
    }
    if level >= 3 {
        log::warn!(
            "[crash-chain] level {level} — the session-persist queue restore will be \
             bypassed this boot (queue data kept on disk)"
        );
    }
}

pub(crate) fn clear_startup_probe() {
    if let Some(path) = startup_probe_path() {
        let _ = std::fs::remove_file(path);
    }
}

/// Disarm the sentinel on proof of LIVENESS: the first real user input (or
/// a close request — a crash never emits one), with a 30s timer as the
/// no-touch fallback. Once-guarded so the hot input path costs one relaxed
/// swap after the first call.
pub(crate) fn disarm_renderer_sentinel_on_liveness(signal: &str) {
    static DISARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !DISARMED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        log::info!("[renderer] startup sentinel disarmed ({signal})");
        clear_renderer_sentinel();
        // Same liveness proof clears the startup crash-chain probe.
        clear_startup_probe();
        log::info!("[crash-chain] startup probe cleared ({signal})");
        // The surviving rung must OUTLIVE the process: when this session ran
        // on the ALTERNATE wgpu adapter, version-stamp that success so the
        // next start arms the alt rung directly — otherwise the rung-2 win
        // dies with the process and the machine crash-cycles every other
        // launch (rung 1 default adapter, crash, rung 2, work, repeat).
        if WGPU_ALT_ADAPTER.load(std::sync::atomic::Ordering::Relaxed) {
            let mut prefs = crate::ui_prefs::load();
            if prefs.renderer_wgpu_alt != env!("CARGO_PKG_VERSION") {
                prefs.renderer_wgpu_alt = env!("CARGO_PKG_VERSION").to_string();
                crate::ui_prefs::save(&prefs);
                log::info!(
                    "[renderer] alternate wgpu adapter survived — persisted for this build"
                );
            }
        }
    }
}

/// Arm the sentinel for a freshly auto-detected tier. Wgpu goes straight to
/// the ALT-adapter rung when this build already proved the default adapter
/// dead (`renderer_wgpu_alt` stamped at disarm time on an alt-rung run).
/// Software is the unarmored floor. macOS stays out of the ladder.
pub(crate) fn arm_auto_tier(tier: RendererTier, prefs: &crate::ui_prefs::UiPrefs) {
    if cfg!(target_os = "macos") {
        return;
    }
    match tier {
        RendererTier::Wgpu => {
            if prefs.renderer_wgpu_alt == env!("CARGO_PKG_VERSION") {
                log::info!(
                    "[renderer] default wgpu adapter is known-bad on this build -> \
                     alternate adapter directly"
                );
                WGPU_ALT_ADAPTER.store(true, std::sync::atomic::Ordering::Relaxed);
                arm_renderer_sentinel("auto-wgpu-alt");
            } else {
                arm_renderer_sentinel("auto-wgpu");
            }
        }
        RendererTier::FemtovgGl => arm_renderer_sentinel("auto-gl"),
        RendererTier::Software => {}
    }
}

/// What the armed sentinel was protecting ("wgpu"/"gl"/"software" for a
/// Settings override, "auto-wgpu"/"auto-wgpu-alt"/"auto-gl" for the auto
/// ladder rungs).
pub(crate) fn renderer_sentinel_value() -> Option<String> {
    std::fs::read_to_string(renderer_sentinel_path()?).ok()
}

/// GPU topology seen during adapter enumeration: whether a discrete and an
/// integrated GPU are BOTH present (hybrid machine). Set by
/// `detect_hardware_gpu`; probed on demand for the forced-wgpu paths.
#[derive(Clone, Copy, Default)]
pub(crate) struct GpuTopology {
    pub(crate) discrete: bool,
    pub(crate) integrated: bool,
}
pub(crate) static GPU_TOPOLOGY: std::sync::OnceLock<GpuTopology> = std::sync::OnceLock::new();

/// Ladder rung 2 (see `renderer_tier_from_prefs`): retry wgpu with the
/// OPPOSITE PowerPreference after the default adapter failed a start —
/// the #542 family is an adapter mixup, not a wgpu failure.
pub(crate) static WGPU_ALT_ADAPTER: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

