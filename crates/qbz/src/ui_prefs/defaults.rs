//! Small `default_*()` free functions backing serde `#[serde(default = "...")]`
//! attributes on [`super::model::UiPrefs`], plus their related constants.

use super::index_maps::{DEFAULT_AUTO_THEME_SOURCE, DEFAULT_LANGUAGE};
use super::index_maps_display::DEFAULT_APP_BACKGROUND;

/// Default album/artist header backdrop setting.
pub const DEFAULT_ALBUM_HEADER_GRADIENT: bool = true;

/// Default intelligent-search setting (smart cache, ranking, preview dropdown).
pub const DEFAULT_INTELLIGENT_SEARCH: bool = true;

pub(super) fn default_system_notifications() -> bool {
    true
}

pub(super) fn default_musicbrainz_enabled() -> bool {
    true
}

pub(super) fn default_nav_in_sidebar() -> bool {
    true
}

pub(super) fn default_volume() -> f32 {
    1.0
}

pub(super) fn default_startup_page() -> String {
    "home".to_string()
}

pub(super) fn default_last_view() -> String {
    "home".to_string()
}

/// Sentinel for "no saved window position" (let the WM place the window).
pub(super) fn default_window_pos() -> i32 {
    i32::MIN
}

/// Per-OS chrome default: Linux keeps the system decorations; macOS defaults
/// to the overlay (custom) mode — see the field doc.
pub(super) fn default_use_system_title_bar() -> bool {
    !cfg!(target_os = "macos")
}

pub(super) fn default_show_window_controls() -> bool {
    true
}

pub(super) fn default_wc_position() -> String {
    "right".to_string()
}

pub(super) fn default_gpu_power() -> String {
    "auto".to_string()
}

pub(super) fn default_renderer() -> String {
    "auto".to_string()
}

pub(super) fn default_ui_scale() -> String {
    "default".to_string()
}

pub(super) fn default_last_dpr() -> f32 {
    1.0
}

pub(super) fn default_language() -> String {
    DEFAULT_LANGUAGE.to_string()
}

pub(super) fn default_large_visualizer() -> bool {
    true
}

pub(super) fn default_large_spectrum_mode() -> String {
    "bars".to_string()
}

pub(super) fn default_album_header_gradient() -> bool {
    DEFAULT_ALBUM_HEADER_GRADIENT
}

pub(super) fn default_intelligent_search() -> bool {
    DEFAULT_INTELLIGENT_SEARCH
}

pub(super) fn default_window_title_show() -> bool {
    false
}
pub(super) fn default_show_volume_steppers() -> bool {
    false
}
pub(super) fn default_sidebar_playlist_collage() -> bool {
    true
}
pub(super) fn default_local_library_track_artwork() -> bool {
    false
}
pub(super) fn default_in_app_toasts() -> bool {
    true
}
pub(super) fn default_theme_filter() -> i32 {
    0
}

pub(super) fn default_app_background() -> String {
    DEFAULT_APP_BACKGROUND.to_string()
}

/// Default theme slug. Owner decision 2026-06-20: OLED Dark is the default for
/// fresh installs and any profile without a persisted theme. Sourced from the
/// `qbz-theme` registry so the default stays single-sourced.
pub(super) fn default_theme() -> String {
    qbz_theme::default_slug().to_string()
}

pub(super) fn default_auto_theme_source() -> String {
    DEFAULT_AUTO_THEME_SOURCE.to_string()
}
