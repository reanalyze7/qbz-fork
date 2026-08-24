use crate::*;

/// Probe wgpu's adapters and classify the best renderer tier available.
/// Conservative: any probe failure, an empty list, or an all-CPU list => Software,
/// because software Vulkan/GL adapters (llvmpipe / lavapipe) report `DeviceType::Cpu`.
/// A non-CPU adapter is WEAK (GL tier) when its name/driver matches a known
/// GLES-class embedded driver, when it's an integrated GPU on arm Linux, or when
/// its texture limits are tiny; anything else is a real GPU (wgpu tier).
/// Adapters reachable only through wgpu's GL backend never count for the wgpu
/// tier: the femtovg-wgpu renderer masks `Backends::GL` out of its instance
/// (vendored wgpu.rs passes it as `backends_to_avoid`), so the wgpu tier could
/// never bind them — they only prove the femtovg-GL tier works.
pub(crate) fn detect_hardware_gpu() -> RendererTier {
    use slint::wgpu_28::wgpu;
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    // wgpu's native `enumerate_adapters` resolves immediately; poll it once.
    let adapters = match poll_ready(instance.enumerate_adapters(wgpu::Backends::all())) {
        Some(a) => a,
        None => {
            log::warn!("[renderer] wgpu adapter enumeration was not immediately ready -> software");
            return RendererTier::Software;
        }
    };
    if adapters.is_empty() {
        log::warn!("[renderer] wgpu found no adapters -> software renderer");
        return RendererTier::Software;
    }
    // Embedded/mobile GPU drivers whose Vulkan path is the slow one (Mesa's GL
    // driver is the mature path on that hardware). `hasvk` is Mesa's legacy
    // Intel gen7/7.5 (Ivy Bridge / Haswell) Vulkan driver — same story: GL is
    // the fast path there. llvmpipe normally reports DeviceType::Cpu and is
    // skipped above, listed only as a belt-and-braces. Intel UHD 600 (Gemini
    // Lake) flickers on the wgpu tier — GL is the stable path there (#578);
    // both name spellings covered since Mesa reports "UHD Graphics 600".
    const WEAK_GPU_MARKERS: [&str; 10] = [
        "v3dv",
        "hasvk",
        "panfrost",
        "panvk",
        "lima",
        "mali",
        "videocore",
        "llvmpipe",
        "uhd graphics 600",
        "uhd 600",
    ];
    let mut best = RendererTier::Software;
    let mut adapter_summary: Vec<String> = Vec::new();
    let mut topo = GpuTopology::default();
    for adapter in &adapters {
        let info = adapter.get_info();
        match info.device_type {
            wgpu::DeviceType::DiscreteGpu => topo.discrete = true,
            wgpu::DeviceType::IntegratedGpu => topo.integrated = true,
            _ => {}
        }
        if info.device_type == wgpu::DeviceType::Cpu {
            log::info!(
                "[renderer] wgpu adapter: '{}' backend={:?} type={:?} class=cpu",
                info.name,
                info.backend,
                info.device_type
            );
            adapter_summary.push(format!("{} [{:?}, cpu]", info.name, info.backend));
            continue;
        }
        // The wgpu tier can never bind this adapter if it only shows up on
        // wgpu's GL backend: the actual renderer init excludes GL (see the fn
        // doc). A GPU with no usable Vulkan ICD would otherwise be classified
        // strong here and then wgpu-init would fall back to llvmpipe (CPU
        // rasterization) or panic "Failed to find an appropriate adapter".
        let gl_backend = info.backend == wgpu::Backend::Gl;
        let name = info.name.to_ascii_lowercase();
        let driver = format!("{} {}", info.driver, info.driver_info).to_ascii_lowercase();
        let weak = WEAK_GPU_MARKERS.iter().any(|m| name.contains(m) || driver.contains(m))
            // Integrated GPU on arm Linux (Pi & friends). Deliberately NOT plain
            // aarch64: Apple Silicon is integrated+aarch64 and is a strong GPU.
            || (cfg!(all(
                target_os = "linux",
                any(target_arch = "aarch64", target_arch = "arm")
            )) && info.device_type == wgpu::DeviceType::IntegratedGpu)
            || adapter.limits().max_texture_dimension_2d <= 4096;
        let class = if weak {
            "weak"
        } else if gl_backend {
            "gl-only"
        } else {
            "strong"
        };
        log::info!(
            "[renderer] wgpu adapter: '{}' backend={:?} type={:?} driver='{}' class={}",
            info.name,
            info.backend,
            info.device_type,
            info.driver,
            class
        );
        adapter_summary.push(format!(
            "{} [{:?}, {}, {}]",
            info.name, info.backend, info.driver, class
        ));
        if !weak && !gl_backend {
            best = RendererTier::Wgpu;
        } else if best == RendererTier::Software {
            best = RendererTier::FemtovgGl;
        }
    }
    let _ = RENDERER_ADAPTERS.set(adapter_summary.join("; "));
    let _ = GPU_TOPOLOGY.set(topo);
    match best {
        RendererTier::Wgpu => log::info!("[renderer] real GPU adapter found -> wgpu renderer"),
        RendererTier::FemtovgGl => log::info!(
            "[renderer] only weak (GLES-class) GPU adapters available -> femtovg GL renderer"
        ),
        RendererTier::Software => log::warn!(
            "[renderer] only software (CPU) GPU adapters available (llvmpipe/lavapipe) \
             -> software renderer"
        ),
    }
    best
}

