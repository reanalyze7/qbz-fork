use crate::*;

/// True when /sys/class/power_supply exposes a SYSTEM battery. Peripheral
/// batteries (wireless mice etc.) report `scope=Device` — exclude them, or
/// every desktop with a wireless mouse would classify as a laptop.
pub(crate) fn linux_has_system_battery() -> bool {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_battery = std::fs::read_to_string(path.join("type"))
            .map(|t| t.trim() == "Battery")
            .unwrap_or(false);
        if !is_battery {
            continue;
        }
        let device_scope = std::fs::read_to_string(path.join("scope"))
            .map(|s| s.trim().eq_ignore_ascii_case("device"))
            .unwrap_or(false);
        if !device_scope {
            return true;
        }
    }
    false
}

/// Default wgpu power preference (WGPU_POWER_PREF always wins, in the caller).
/// LowPower keeps hybrid LAPTOPS on their integrated GPU: the panel is wired
/// to it and this mostly-idle UI doesn't need the discrete card. On a hybrid
/// DESKTOP the monitor usually hangs off the discrete card and the display-less
/// iGPU cannot present the window surface — wgpu still picks it under LowPower
/// and `Surface::configure` panics with "Invalid surface" (#542). So a hybrid
/// machine WITHOUT a system battery (= desktop) defaults to HighPerformance.
pub(crate) fn default_wgpu_power_preference() -> slint::wgpu_28::wgpu::PowerPreference {
    use slint::wgpu_28::wgpu::PowerPreference;
    if cfg!(target_os = "macos") {
        return PowerPreference::LowPower;
    }
    let topo = GPU_TOPOLOGY.get_or_init(probe_gpu_topology);
    if topo.discrete && topo.integrated && !linux_has_system_battery() {
        log::info!(
            "[renderer] hybrid discrete+integrated GPUs with no system battery (desktop) \
             -> HighPerformance adapter default"
        );
        PowerPreference::HighPerformance
    } else {
        PowerPreference::LowPower
    }
}

/// Persisted "Preferred GPU" setting -> wgpu adapter PowerPreference override.
/// `None` = "auto" (fall through to `default_wgpu_power_preference`). The
/// WGPU_POWER_PREF env still wins over this (it is checked first in the caller).
/// On a hybrid laptop "discrete" (HighPerformance) moves the render off the
/// integrated GPU — the fix for the iGPU running hot under the shader background.
pub(crate) fn gpu_power_from_prefs() -> Option<slint::wgpu_28::wgpu::PowerPreference> {
    use slint::wgpu_28::wgpu::PowerPreference;
    let pref = crate::ui_prefs::load().gpu_power;
    match pref.as_str() {
        "" | "auto" => None,
        // Legacy type keys.
        "integrated" => Some(PowerPreference::LowPower),
        "discrete" => Some(PowerPreference::HighPerformance),
        // A specific device name: pick the power preference that steers wgpu to
        // that adapter's class (exact on the usual 1 iGPU + 1 dGPU laptop; a
        // same-class tie resolves to whichever wgpu prefers). Unknown name (GPU
        // removed) → None = auto.
        name => gpu_adapters().iter().find(|a| a.name == name).map(|a| {
            if a.discrete {
                PowerPreference::HighPerformance
            } else {
                PowerPreference::LowPower
            }
        }),
    }
}

/// Minimal `block_on` for the wgpu init futures (ready on first poll on native
/// platforms; the parking loop keeps it correct even if a driver ever leaves
/// them pending). The tokio runtime does not exist yet this early in startup,
/// and pulling one in just for the adapter/device requests is overkill.
pub(crate) fn block_on_wgpu<F: std::future::Future>(future: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    struct Parker(std::thread::Thread);
    impl Wake for Parker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker: Waker = Arc::new(Parker(std::thread::current())).into();
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(out) => return out,
            Poll::Pending => std::thread::park(),
        }
    }
}

