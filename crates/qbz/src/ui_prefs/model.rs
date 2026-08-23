//! The `UiPrefs` struct definition. Its `Default` impl lives in
//! `model_default.rs` (split out to reduce this file's line count — the
//! struct itself cannot be split further: Rust has no partial-struct
//! mechanism, and nesting the fields into sub-structs would change the flat
//! JSON shape and the `prefs.field` access pattern used pervasively across
//! the crate).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::defaults::*;
use super::quality::default_streaming_quality;

/// Persisted UI preferences. New fields must default sanely so an older file
/// (missing the field) still deserializes. See `model_default.rs` for the
/// `Default` impl, kept in a sibling file to stay under the line limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrefs {
    /// Streaming-quality key — one of `STREAMING_QUALITIES[*].key`.
    #[serde(default = "default_streaming_quality")]
    pub streaming_quality: String,
    /// UI language key: "auto" (follow the OS locale) or one of en/es/de/fr/pt. Persists the raw user choice; "auto" resolves at startup via `qbz_i18n::resolve_auto()`.
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_large_visualizer")]
    pub large_visualizer: bool,
    /// Large-NPB dock spectrum visualization: "bars"/"waveform"/"energy" — maps to `ShellState.large-spectrum-mode` (0/1/2).
    #[serde(default = "default_large_spectrum_mode")]
    pub large_spectrum_mode: String,
    /// Whether album/artist detail headers use artwork-derived backdrops.
    #[serde(default = "default_album_header_gradient")]
    pub album_header_gradient: bool,
    /// Whether intelligent search (cache, ranking, preview dropdown) is enabled.
    #[serde(default = "default_intelligent_search")]
    pub intelligent_search: bool,
    /// Appearance toggles, seeding the live Slint globals at startup.
    #[serde(default = "default_window_title_show")]
    pub window_title_show: bool,
    #[serde(default = "default_show_volume_steppers")]
    pub show_volume_steppers: bool,
    #[serde(default = "default_sidebar_playlist_collage")]
    pub sidebar_playlist_collage: bool,
    #[serde(default = "default_local_library_track_artwork")]
    pub local_library_track_artwork: bool,
    #[serde(default = "default_in_app_toasts")]
    pub in_app_toasts: bool,
    #[serde(default = "default_theme_filter")]
    pub theme_filter: i32,
    /// Desktop "now playing" system notifications on track change. Default ON.
    #[serde(default = "default_system_notifications")]
    pub system_notifications: bool,
    /// MusicBrainz metadata enrichment (opt-out, default ON) — gates the artist Network/Scene sidebar + playlist Suggested-Songs.
    #[serde(default = "default_musicbrainz_enabled")]
    pub musicbrainz_enabled: bool,
    /// System window decorations. Per-OS default: TRUE on Linux, FALSE on macOS (overlay chrome). Startup-time only; changes need a restart.
    #[serde(default = "default_use_system_title_bar")]
    pub use_system_title_bar: bool,
    /// Frameless window WITHOUT drawn controls/drag (tiling-WM). Only meaningful when `use_system_title_bar` is false. Default OFF.
    #[serde(default)]
    pub hide_title_bar: bool,
    #[serde(default = "default_show_window_controls")]
    pub show_window_controls: bool,
    /// Window-controls side: "left"/"right". Default right.
    #[serde(default = "default_wc_position")]
    pub wc_position: String,
    /// Three-state sidebar: 0 open / 1 mini / 2 closed. Restored at startup.
    #[serde(default)]
    pub sidebar_state: i32,
    #[serde(default = "default_nav_in_sidebar")]
    pub nav_in_sidebar: bool,
    #[serde(default)]
    pub nav_header_compact: bool,
    /// Player volume, 0.0..=1.0. Restored at startup. Default full.
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_startup_page")]
    pub startup_page: String,
    /// Last visited SAFE top-level view (no required id), for "remember". Detail views are never stored here.
    #[serde(default = "default_last_view")]
    pub last_view: String,
    /// Full last nav destination as JSON-encoded `nav::NavEntry`, for "remember" (restores the EXACT view, re-fetched by id). `None` until a view is visited.
    #[serde(default)]
    pub last_nav: Option<String>,
    /// App-wide dynamic background key: "off"/"ambient"/"blurred". See [`super::index_maps_display::DEFAULT_APP_BACKGROUND`].
    #[serde(default = "default_app_background")]
    pub app_background: String,
    /// Active theme — a stable `qbz_theme::ThemeId` slug. Owner default: OLED Dark.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Auto-theme source (only meaningful when `theme == "auto"`): "system"/"wallpaper"/"image". See [`super::index_maps::DEFAULT_AUTO_THEME_SOURCE`].
    #[serde(default = "default_auto_theme_source")]
    pub auto_theme_source: String,
    #[serde(default)]
    pub auto_theme_image_path: String,

    /// User keybinding overrides: action id -> shortcut string. A missing entry means the action uses its compiled default (see `crate::keybindings::ACTIONS`).
    #[serde(default)]
    pub keybindings: BTreeMap<String, String>,

    /// Last main-window LOGICAL size. 0 = never saved -> use the `.slint` preferred size, clamped to the monitor.
    #[serde(default)]
    pub window_width: f32,
    #[serde(default)]
    pub window_height: f32,
    /// Last main-window PHYSICAL outer position. `i32::MIN` = never saved -> let the window manager place it.
    #[serde(default = "default_window_pos")]
    pub window_x: i32,
    #[serde(default = "default_window_pos")]
    pub window_y: i32,
    #[serde(default)]
    pub window_maximized: bool,
    /// Renderer tier override: "auto"/"wgpu"/"gl"/"software". Linux-only surface, read before the window exists.
    #[serde(default = "default_renderer")]
    pub renderer: String,
    #[serde(default = "default_gpu_power")]
    pub gpu_power: String,
    /// App version that AUTO-degraded `renderer` (ladder persisted "gl"/"software" after failed starts). Empty = user's own choice.
    #[serde(default)]
    pub renderer_auto_degraded: String,
    /// App version whose ALT-adapter wgpu rung SURVIVED a session. Version-keyed like `renderer_auto_degraded`; cleared on a manual pick.
    #[serde(default)]
    pub renderer_wgpu_alt: String,
    /// Interface-size preset: "default"/"small"/"large"/"xl". Read at the very top of main() to set SLINT_SCALE_FACTOR.
    #[serde(default = "default_ui_scale")]
    pub ui_scale: String,
    /// Last observed compositor device-pixel-ratio (`env = last_dpr × preset`, since SLINT_SCALE_FACTOR overrides rather than multiplies).
    #[serde(default = "default_last_dpr")]
    pub last_dpr: f32,
}
