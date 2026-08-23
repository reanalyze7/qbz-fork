//! Index<->key mapping pure functions for renderer, interface-size,
//! app-background and large-dock-spectrum dropdown-backed settings.

/// Renderer select index (0 = Auto, 1 = GPU, 2 = GPU compatibility/GL,
/// 3 = Software) -> persisted key. Unknown indices fall back to "auto".
pub fn renderer_for_index(index: i32) -> &'static str {
    match index {
        1 => "wgpu",
        2 => "gl",
        3 => "software",
        _ => "auto",
    }
}

/// Inverse: select index for a persisted renderer key.
pub fn renderer_index(key: &str) -> i32 {
    match key {
        "wgpu" => 1,
        "gl" => 2,
        "software" => 3,
        _ => 0,
    }
}

/// Interface-size select index (0 = Extra small, 1 = Small, 2 = Default,
/// 3 = Large, 4 = Extra large) -> persisted key. Unknown indices fall back
/// to "default".
pub fn ui_scale_for_index(index: i32) -> &'static str {
    match index {
        0 => "xs",
        1 => "small",
        3 => "large",
        4 => "xl",
        _ => "default",
    }
}

/// Inverse: select index for a persisted interface-size key.
pub fn ui_scale_index(key: &str) -> i32 {
    match key {
        "xs" => 0,
        "small" => 1,
        "large" => 3,
        "xl" => 4,
        _ => 2,
    }
}

/// Numeric window-scale multiplier for a persisted interface-size key.
pub fn ui_scale_factor(key: &str) -> f32 {
    match key {
        "xs" => 0.8,
        "small" => 0.9,
        "large" => 1.2,
        "xl" => 1.5,
        _ => 1.0,
    }
}

/// Default app-wide dynamic background: "off". Other keys: "ambient" (GPU
/// shader scene, wgpu tier only) | "blurred" (blurred-artwork atmosphere).
pub const DEFAULT_APP_BACKGROUND: &str = "off";

/// Map an app-background select index to its persisted key. On-screen order is
/// Off / Ambient / Blurred (0-2); any unknown index falls back to the default
/// (`"off"`).
pub fn app_background_for_index(index: i32) -> &'static str {
    match index {
        0 => "off",
        1 => "ambient",
        2 => "blurred",
        _ => DEFAULT_APP_BACKGROUND,
    }
}

/// Inverse of [`app_background_for_index`]: the select index for a persisted
/// key, falling back to the default's index (0 = "off").
pub fn app_background_index(key: &str) -> i32 {
    match key {
        "off" => 0,
        "ambient" => 1,
        "blurred" => 2,
        _ => 0,
    }
}

/// Map a persisted Large-dock spectrum key to `ShellState.large-spectrum-mode`
/// (Bars = 0, Waveform = 1, Energy = 2). Unknown keys fall back to Bars.
pub fn large_spectrum_mode_index(key: &str) -> i32 {
    match key {
        "waveform" => 1,
        "energy" => 2,
        _ => 0,
    }
}

/// Inverse of [`large_spectrum_mode_index`] — the persisted key for an int mode.
pub fn large_spectrum_mode_key(index: i32) -> &'static str {
    match index {
        1 => "waveform",
        2 => "energy",
        _ => "bars",
    }
}
