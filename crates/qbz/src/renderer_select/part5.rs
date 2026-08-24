use crate::*;

/// Create the wgpu instance/adapter/device/queue ONCE at startup so they can be
/// handed to Slint as `WGPUConfiguration::Manual` and REUSED across every
/// window re-creation. Mirrors Slint's own Automatic init (vendored
/// i-slint-core `graphics/wgpu_28.rs`) minus the surface — none exists yet at
/// this point, and Vulkan-Wayland adapters are not surface-specific.
///
/// Why: on Wayland `hide()` destroys the winit window and every `show()`
/// re-runs the femtovg-wgpu surface setup; with `Automatic` each re-creation
/// spins up a BRAND-NEW VkInstance + VkDevice (and drops the old ones). On the
/// NVIDIA proprietary driver that churn segfaults — a null call inside
/// libnvidia-glcore during surface re-creation, racing in-flight GPU work
/// (close-to-tray → restore; reproduced under gdb 2026-07-18: crash inside
/// `set_surface`, before the first render, with playback starting mid-restore).
/// With a Manual stack the restore only creates a new VkSurfaceKHR + swapchain
/// on the SAME device — the path every resize already exercises.
///
/// Returns `None` on any failure so the caller can fall back to `Automatic`.
pub(crate) fn create_shared_wgpu_stack(
    settings: &slint::wgpu_28::WGPUSettings,
) -> Option<(
    slint::wgpu_28::wgpu::Instance,
    slint::wgpu_28::wgpu::Adapter,
    slint::wgpu_28::wgpu::Device,
    slint::wgpu_28::wgpu::Queue,
)> {
    use slint::wgpu_28::wgpu;
    // Same backend mask as the femtovg-wgpu renderer (GL excluded for its
    // rendering artifacts); the rest follows WGPUSettings/env as before.
    let instance = block_on_wgpu(wgpu::util::new_instance_with_webgpu_detection(
        &wgpu::InstanceDescriptor {
            backends: settings.backends & !wgpu::Backends::GL,
            flags: settings.instance_flags,
            backend_options: settings.backend_options.clone(),
            memory_budget_thresholds: settings.instance_memory_budget_thresholds,
        },
    ));
    let adapter = match block_on_wgpu(wgpu::util::initialize_adapter_from_env(&instance, None)) {
        Ok(adapter) => adapter,
        Err(_) => block_on_wgpu(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: settings.power_preference,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .map_err(|e| log::warn!("[renderer] shared wgpu adapter request failed: {e}"))
        .ok()?,
    };
    let (device, queue) = block_on_wgpu(adapter.request_device(&wgpu::DeviceDescriptor {
        label: settings.device_label.as_deref(),
        required_features: settings.device_required_features,
        required_limits: settings
            .device_required_limits
            .clone()
            .using_resolution(adapter.limits()),
        experimental_features: settings.device_experimental_features,
        memory_hints: settings.device_memory_hints.clone(),
        trace: wgpu::Trace::default(),
    }))
    .map_err(|e| log::warn!("[renderer] shared wgpu device request failed: {e}"))
    .ok()?;
    log::info!(
        "[renderer] shared wgpu stack created ({}), reused across surface re-creations",
        adapter.get_info().name
    );
    Some((instance, adapter, device, queue))
}

