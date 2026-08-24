use crate::*;

/// Renderer tiers, best first. `FemtovgGl` is the middle tier for weak GPUs
/// (Raspberry Pi-class): they expose a real Vulkan adapter (v3dv/panfrost/…) that
/// wgpu happily binds, but that driver path crawls there — Mesa's GLES driver is
/// the fast path on such hardware, so femtovg-over-GL beats femtovg-over-wgpu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RendererTier {
    Wgpu,
    FemtovgGl,
    Software,
}

/// What the renderer selection actually decided, surfaced in the Developer
/// tools diagnostics ("which renderer am I really running, and why?"). Set
/// once during `select_slint_backend`; read by `diagnostics.rs`.
pub struct RendererDecision {
    /// Active tier label, e.g. "wgpu (femtovg)" | "GL (femtovg)" | "software"
    /// | "skia (Metal)" on macOS.
    pub tier: &'static str,
    /// Why it was chosen: env var, Settings override, auto-detect, or the
    /// auto-revert after a failed start.
    pub source: String,
}

pub static RENDERER_DECISION: std::sync::OnceLock<RendererDecision> = std::sync::OnceLock::new();
/// Adapter enumeration summary from `detect_hardware_gpu` ("-" if the probe
/// never ran, i.e. the tier was forced by env/Settings).
pub(crate) static RENDERER_ADAPTERS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
/// Set when a persisted (non-auto) renderer override was reverted because the
/// previous start died before its first paint; read once the UI is up to show
/// the explanatory toast.
pub(crate) static RENDERER_REVERTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Set when the AUTO-DETECTED wgpu tier crashed pre-paint on the previous
/// start and the setting was degraded to "gl" (#542); read once the UI is up
/// to show the explanatory toast.
pub(crate) static RENDERER_DEGRADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// One-line diagnostics summary of the active renderer + adapters.
pub fn renderer_decision_summary() -> (String, String) {
    match RENDERER_DECISION.get() {
        Some(d) => (
            format!("{} — {}", d.tier, d.source),
            RENDERER_ADAPTERS.get().cloned().unwrap_or_else(|| "—".to_string()),
        ),
        None => ("—".to_string(), "—".to_string()),
    }
}

/// Startup auto-revert sentinel for a persisted (non-auto) renderer override.
/// Armed BEFORE the risky backend init, disarmed a few seconds after the main
/// window is up. If it survives to the next start, that start reverts the
/// override to "auto" so a bad renderer choice can't lock the user out of
/// Settings. Lives next to ui_prefs.json.
pub(crate) fn renderer_sentinel_path() -> Option<std::path::PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("renderer_attempt"))
}

pub(crate) fn arm_renderer_sentinel(key: &str) {
    let Some(path) = renderer_sentinel_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&path, key) {
        log::warn!("[renderer] could not arm the startup sentinel: {e}");
    }
}

pub(crate) fn clear_renderer_sentinel() {
    if let Some(path) = renderer_sentinel_path() {
        let _ = std::fs::remove_file(path);
    }
}

pub(crate) fn renderer_sentinel_armed() -> bool {
    renderer_sentinel_path().map(|p| p.exists()).unwrap_or(false)
}

/// Startup crash-chain watchdog — the renderer-sentinel pattern generalized
/// to the whole startup path (incident 2026-07-08: a "Recursion detected"
/// render panic inside the RESTORED view fired before first input, so every
/// start re-restored the crashing view and died — a crash-loop the renderer
/// ladder could not see). A counter file is incremented at every launch
/// BEFORE risky init and cleared by the same liveness proof that disarms
/// the renderer sentinel; the value it had already reached at launch is the
/// number of consecutive starts that died before liveness.
///
/// Recovery ladder (surgical: state is reset or bypassed, never deleted):
/// - level 2 (one prior start died): reset the persisted view restore —
///   `last_nav` -> "{}", `last_view` -> "home" — so this and future boots
///   start on Home.
/// - level >=3: additionally BYPASS the session-persist queue restore for
///   this boot only (the persisted queue file is kept untouched).
pub(crate) fn startup_probe_path() -> Option<std::path::PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("startup_probe"))
}

/// This boot's crash-chain level (1 = clean previous shutdown/liveness).
pub(crate) static CRASH_CHAIN_LEVEL: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub(crate) fn crash_chain_level() -> u8 {
    CRASH_CHAIN_LEVEL.load(std::sync::atomic::Ordering::Relaxed)
}

