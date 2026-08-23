#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    // Audio: saved settings
    pub audio_output_device: Option<String>,
    pub audio_backend_type: Option<String>,
    pub audio_exclusive_mode: bool,
    pub audio_dac_passthrough: bool,
    pub audio_preferred_sample_rate: Option<u32>,
    pub audio_alsa_plugin: Option<String>,
    pub audio_alsa_hardware_volume: bool,
    pub audio_normalization_enabled: bool,
    pub audio_normalization_target_lufs: f32,
    pub audio_gapless_enabled: bool,
    pub audio_pw_force_bitperfect: bool,
    pub audio_stream_buffer_seconds: u8,
    pub audio_streaming_only: bool,

    // Graphics: saved settings
    pub gfx_hardware_acceleration: bool,
    pub gfx_force_x11: bool,
    pub gfx_gdk_scale: Option<String>,
    pub gfx_gdk_dpi_scale: Option<String>,
    pub gfx_gsk_renderer: Option<String>,

    // Graphics: runtime (what actually applied at startup)
    pub runtime_using_fallback: bool,
    pub runtime_is_wayland: bool,
    pub runtime_has_nvidia: bool,
    pub runtime_has_amd: bool,
    pub runtime_has_intel: bool,
    pub runtime_is_vm: bool,
    pub runtime_hw_accel_enabled: bool,
    pub runtime_force_x11_active: bool,
    /// Human-readable GPU model name (driver-reported on Linux).
    /// For hybrid laptops joins both vendors: "NVIDIA (...) + Intel (...)".
    pub runtime_gpu_name: String,
    /// Desktop environment string ($XDG_CURRENT_DESKTOP or fallbacks).
    pub runtime_desktop_environment: String,

    // Developer settings
    pub dev_force_dmabuf: bool,

    // Environment variables (what WebKit actually sees)
    pub env_webkit_disable_dmabuf: Option<String>,
    pub env_webkit_disable_compositing: Option<String>,
    pub env_gdk_backend: Option<String>,
    pub env_gsk_renderer: Option<String>,
    pub env_libgl_always_software: Option<String>,
    pub env_wayland_display: Option<String>,
    pub env_xdg_session_type: Option<String>,

    // App info
    pub app_version: String,
}

/// Graphics runtime state the caller feeds in.
///
/// Mirrors the Tauri startup atomics (`get_graphics_startup_status()`), so the
/// builder stays framework-agnostic. The Tauri command maps its atomics into
/// this; the Slint bin computes it fresh via [`super::detect_graphics_runtime`].
pub struct GraphicsRuntime {
    pub using_fallback: bool,
    pub is_wayland: bool,
    pub has_nvidia: bool,
    pub has_amd: bool,
    pub has_intel: bool,
    pub is_vm: bool,
    pub hardware_accel_enabled: bool,
    pub force_x11_active: bool,
}

/// Inputs for [`super::runtime_diagnostics`]. The caller reads the three settings
/// structs from their stores, builds a [`GraphicsRuntime`], and passes a real
/// `app_version` string.
pub struct DiagnosticsInputs<'a> {
    pub audio: &'a qbz_audio::settings::AudioSettings,
    pub graphics: &'a crate::settings::graphics::GraphicsSettings,
    pub developer: &'a crate::settings::developer::DeveloperSettings,
    pub gfx: GraphicsRuntime,
    pub app_version: &'a str,
}
