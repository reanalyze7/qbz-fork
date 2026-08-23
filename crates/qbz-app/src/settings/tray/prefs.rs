use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraySettings {
    /// Show system tray icon (requires restart to take effect).
    pub enable_tray: bool,
    /// Hide window to tray when clicking minimize.
    pub minimize_to_tray: bool,
    /// Hide window to tray instead of quitting when clicking close.
    /// Opt-out: enabled by default (closing sends the window to the tray and
    /// keeps the app running, like Spotify/Discord).
    pub close_to_tray: bool,
    /// Tray icon variant override:
    /// - "auto" follows system color scheme,
    /// - "mono-light" uses a light glyph for dark panels,
    /// - "mono-dark" uses a dark glyph for light panels,
    /// - "color" uses the full-color vinyl logo.
    #[serde(default = "default_tray_icon_theme")]
    pub tray_icon_theme: String,
    /// macOS only: when closed to the menu bar, switch the activation policy
    /// to `.accessory` (no Dock icon, menu-bar-only). Off keeps the Dock icon
    /// (Spotify-style). Ignored on Linux/Windows.
    #[serde(default)]
    pub mac_hide_dock: bool,
}

fn default_tray_icon_theme() -> String {
    "auto".to_string()
}

/// Coerce free-form values to the supported set. Anything outside the
/// supported list falls back to "auto".
///
/// Legacy 1.2.9-pre values "light"/"dark" had inverted semantics
/// relative to the labels users saw. They are remapped here to the value
/// that matches the user's original intent.
pub fn normalize_tray_icon_theme(input: &str) -> String {
    match input {
        "mono-light" | "mono-dark" | "color" | "auto" => input.to_string(),
        "light" => "mono-light".to_string(),
        "dark" => "mono-dark".to_string(),
        _ => "auto".to_string(),
    }
}

impl Default for TraySettings {
    fn default() -> Self {
        Self {
            enable_tray: true,
            minimize_to_tray: false,
            close_to_tray: true,
            tray_icon_theme: default_tray_icon_theme(),
            mac_hide_dock: false,
        }
    }
}
