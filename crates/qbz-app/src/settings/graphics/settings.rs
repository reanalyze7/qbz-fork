use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// GPU rendering toggle. Read at startup as default; env var QBZ_HARDWARE_ACCEL overrides.
    pub hardware_acceleration: bool,
    /// Force X11/XWayland backend on Wayland sessions (requires restart).
    pub force_x11: bool,
    /// GDK_SCALE override for XWayland (None = auto). Integer values: "1", "2".
    pub gdk_scale: Option<String>,
    /// GDK_DPI_SCALE override for XWayland (None = auto). Float values: "0.5", "1", "1.5".
    pub gdk_dpi_scale: Option<String>,
    /// GSK_RENDERER override (None = auto). Values: "gl", "ngl", "vulkan", "cairo".
    pub gsk_renderer: Option<String>,
    /// Rendering GPU selection.
    ///
    /// Valid values are "auto", "integrated", "discrete", "software", or a
    /// host-specific GPU id such as a PCI slot.
    pub preferred_gpu: String,
    /// Opt-in NVIDIA Wayland compatibility mode. The host applies the runtime
    /// environment changes before graphics initialization.
    pub nvidia_compat_mode: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            hardware_acceleration: true,
            force_x11: false,
            gdk_scale: None,
            gdk_dpi_scale: None,
            gsk_renderer: None,
            preferred_gpu: "auto".to_string(),
            nvidia_compat_mode: false,
        }
    }
}
