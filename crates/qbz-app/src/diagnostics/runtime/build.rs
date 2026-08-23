use super::types::{DiagnosticsInputs, GraphicsRuntime, RuntimeDiagnostics};

/// Build the runtime diagnostics snapshot. Infallible.
///
/// Faithful port of `v2_get_runtime_diagnostics` (Tauri) reading from the
/// passed-in structs instead of `tauri::State`. The two `format!("{:?}", ..)`
/// Debug conversions for `audio_backend_type` and `audio_alsa_plugin` are kept
/// so the exported strings match Tauri exactly (stable enum variant names).
pub fn runtime_diagnostics(i: &DiagnosticsInputs<'_>) -> RuntimeDiagnostics {
    let audio = i.audio;
    let gfx = i.graphics;
    let dev = i.developer;

    let env_var = |name: &str| std::env::var(name).ok();

    RuntimeDiagnostics {
        audio_output_device: audio.output_device.clone(),
        audio_backend_type: audio.backend_type.map(|b| format!("{:?}", b)),
        audio_exclusive_mode: audio.exclusive_mode,
        audio_dac_passthrough: audio.dac_passthrough,
        audio_preferred_sample_rate: audio.preferred_sample_rate,
        audio_alsa_plugin: audio.alsa_plugin.map(|p| format!("{:?}", p)),
        audio_alsa_hardware_volume: audio.alsa_hardware_volume,
        audio_normalization_enabled: audio.normalization_enabled,
        audio_normalization_target_lufs: audio.normalization_target_lufs,
        audio_gapless_enabled: audio.gapless_enabled,
        audio_pw_force_bitperfect: audio.pw_force_bitperfect,
        audio_stream_buffer_seconds: audio.stream_buffer_seconds,
        audio_streaming_only: audio.streaming_only,

        gfx_hardware_acceleration: gfx.hardware_acceleration,
        gfx_force_x11: gfx.force_x11,
        gfx_gdk_scale: gfx.gdk_scale.clone(),
        gfx_gdk_dpi_scale: gfx.gdk_dpi_scale.clone(),
        gfx_gsk_renderer: gfx.gsk_renderer.clone(),

        runtime_using_fallback: i.gfx.using_fallback,
        runtime_is_wayland: i.gfx.is_wayland,
        runtime_has_nvidia: i.gfx.has_nvidia,
        runtime_has_amd: i.gfx.has_amd,
        runtime_has_intel: i.gfx.has_intel,
        runtime_is_vm: i.gfx.is_vm,
        runtime_hw_accel_enabled: i.gfx.hardware_accel_enabled,
        runtime_force_x11_active: i.gfx.force_x11_active,
        runtime_gpu_name: crate::graphics_autoconfig::detect_gpu_name(
            i.gfx.has_nvidia,
            i.gfx.has_amd,
            i.gfx.has_intel,
        ),
        runtime_desktop_environment: std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "Unknown".to_string()),

        dev_force_dmabuf: dev.force_dmabuf,

        env_webkit_disable_dmabuf: env_var("WEBKIT_DISABLE_DMABUF_RENDERER"),
        env_webkit_disable_compositing: env_var("WEBKIT_DISABLE_COMPOSITING_MODE"),
        env_gdk_backend: env_var("GDK_BACKEND"),
        env_gsk_renderer: env_var("GSK_RENDERER"),
        env_libgl_always_software: env_var("LIBGL_ALWAYS_SOFTWARE"),
        env_wayland_display: env_var("WAYLAND_DISPLAY"),
        env_xdg_session_type: env_var("XDG_SESSION_TYPE"),

        app_version: i.app_version.to_string(),
    }
}

/// Compute a [`GraphicsRuntime`] for the headless/Slint path.
///
/// Runs [`crate::graphics_autoconfig::detect_environment`] (pure /proc + /sys +
/// env detection) and maps it. `hardware_accel_enabled` reflects the saved
/// graphics setting; `force_x11_active` is `false` (the Slint bin has no
/// force-x11 path — it renders via winit/wgpu, not GDK); `using_fallback` is
/// passed in by the caller (e.g. set when the graphics store failed to open).
pub fn detect_graphics_runtime(
    saved: &crate::settings::graphics::GraphicsSettings,
    using_fallback: bool,
) -> GraphicsRuntime {
    let env = crate::graphics_autoconfig::detect_environment();
    GraphicsRuntime {
        using_fallback,
        is_wayland: env.display_server == "Wayland",
        has_nvidia: env.gpu_nvidia,
        has_amd: env.gpu_amd,
        has_intel: env.gpu_intel,
        is_vm: env.is_vm,
        hardware_accel_enabled: saved.hardware_acceleration,
        force_x11_active: false,
    }
}
