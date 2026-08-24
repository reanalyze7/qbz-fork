use crate::*;

/// Minimal adapter probe for the paths that skip `detect_hardware_gpu`
/// (QBZ_RENDERER / Settings forcing the wgpu tier).
pub(crate) fn probe_gpu_topology() -> GpuTopology {
    use slint::wgpu_28::wgpu;
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapters =
        poll_ready(instance.enumerate_adapters(wgpu::Backends::all())).unwrap_or_default();
    let mut topo = GpuTopology::default();
    for adapter in &adapters {
        match adapter.get_info().device_type {
            wgpu::DeviceType::DiscreteGpu => topo.discrete = true,
            wgpu::DeviceType::IntegratedGpu => topo.integrated = true,
            _ => {}
        }
    }
    topo
}

/// A real (non-CPU) GPU adapter, for the "Preferred GPU" Settings dropdown.
#[derive(Clone)]
pub(crate) struct GpuAdapterInfo {
    name: String,
    discrete: bool,
}
pub(crate) static GPU_ADAPTERS: std::sync::OnceLock<Vec<GpuAdapterInfo>> = std::sync::OnceLock::new();

/// Enumerate the machine's real GPUs (integrated + discrete), deduped by name
/// (enumerate_adapters returns one per backend, so a single physical GPU shows
/// up under Vulkan AND GL). Cached — the wgpu instance is created once.
pub(crate) fn gpu_adapters() -> &'static Vec<GpuAdapterInfo> {
    GPU_ADAPTERS.get_or_init(|| {
        use slint::wgpu_28::wgpu;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapters =
            poll_ready(instance.enumerate_adapters(wgpu::Backends::all())).unwrap_or_default();
        let mut out: Vec<GpuAdapterInfo> = Vec::new();
        for adapter in &adapters {
            let info = adapter.get_info();
            let discrete = match info.device_type {
                wgpu::DeviceType::DiscreteGpu => true,
                wgpu::DeviceType::IntegratedGpu => false,
                _ => continue, // skip CPU / virtual / other
            };
            if !out.iter().any(|a| a.name == info.name) {
                out.push(GpuAdapterInfo {
                    name: info.name.clone(),
                    discrete,
                });
            }
        }
        out
    })
}

/// Dropdown labels for the "Preferred GPU" setting: "Auto" + one per detected
/// GPU, tagged (discrete)/(integrated). Built in Rust so it reflects the real
/// hardware; index 0 = Auto, index i>0 = `gpu_adapters()[i-1]`.
pub(crate) fn gpu_power_options() -> Vec<slint::SharedString> {
    let mut opts: Vec<slint::SharedString> = vec![qbz_i18n::t("Auto (recommended)").into()];
    for a in gpu_adapters() {
        let tag = if a.discrete {
            qbz_i18n::t("discrete")
        } else {
            qbz_i18n::t("integrated")
        };
        opts.push(format!("{} ({})", a.name, tag).into());
    }
    opts
}

/// Persisted `gpu_power` -> the dropdown index in the CURRENT enumeration.
/// Accepts a device name, the legacy type keys ("integrated"/"discrete"), or
/// "auto". Falls back to 0 (Auto) when the stored device is no longer present.
pub(crate) fn gpu_power_seed_index(persisted: &str) -> i32 {
    let adapters = gpu_adapters();
    match persisted {
        "" | "auto" => 0,
        "integrated" => adapters
            .iter()
            .position(|a| !a.discrete)
            .map_or(0, |i| i as i32 + 1),
        "discrete" => adapters
            .iter()
            .position(|a| a.discrete)
            .map_or(0, |i| i as i32 + 1),
        name => adapters
            .iter()
            .position(|a| a.name == name)
            .map_or(0, |i| i as i32 + 1),
    }
}

/// Dropdown index -> the value to persist in `gpu_power` (0 = "auto", else the
/// selected adapter's name).
pub(crate) fn gpu_power_value_for_index(index: i32) -> String {
    if index <= 0 {
        return "auto".to_string();
    }
    gpu_adapters()
        .get((index - 1) as usize)
        .map_or_else(|| "auto".to_string(), |a| a.name.clone())
}

