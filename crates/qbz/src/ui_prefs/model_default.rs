//! `impl Default for UiPrefs` — split out of `model.rs` to stay under the
//! file-length limit (the struct definition itself cannot be split across
//! files: Rust has no partial-struct mechanism, and splitting the fields into
//! nested sub-structs would change the flat JSON shape + the `prefs.field`
//! access pattern used pervasively across the crate, so the single flat
//! struct in `model.rs` is a deliberate, documented exception to the
//! 130-line rule).

use std::collections::BTreeMap;

use super::defaults::*;
use super::model::UiPrefs;
use super::quality::default_streaming_quality;

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            streaming_quality: default_streaming_quality(),
            language: default_language(),
            large_visualizer: default_large_visualizer(),
            large_spectrum_mode: default_large_spectrum_mode(),
            album_header_gradient: default_album_header_gradient(),
            intelligent_search: default_intelligent_search(),
            window_title_show: default_window_title_show(),
            show_volume_steppers: default_show_volume_steppers(),
            sidebar_playlist_collage: default_sidebar_playlist_collage(),
            local_library_track_artwork: default_local_library_track_artwork(),
            in_app_toasts: default_in_app_toasts(),
            theme_filter: default_theme_filter(),
            system_notifications: default_system_notifications(),
            musicbrainz_enabled: default_musicbrainz_enabled(),
            use_system_title_bar: default_use_system_title_bar(),
            hide_title_bar: false,
            show_window_controls: default_show_window_controls(),
            wc_position: default_wc_position(),
            sidebar_state: 0,
            nav_in_sidebar: default_nav_in_sidebar(),
            nav_header_compact: false,
            volume: default_volume(),
            startup_page: default_startup_page(),
            last_view: default_last_view(),
            last_nav: None,
            app_background: default_app_background(),
            theme: default_theme(),
            auto_theme_source: default_auto_theme_source(),
            auto_theme_image_path: String::new(),
            keybindings: BTreeMap::new(),
            window_width: 0.0,
            window_height: 0.0,
            window_x: default_window_pos(),
            window_y: default_window_pos(),
            window_maximized: false,
            renderer: default_renderer(),
            gpu_power: default_gpu_power(),
            renderer_auto_degraded: String::new(),
            renderer_wgpu_alt: String::new(),
            ui_scale: default_ui_scale(),
            last_dpr: default_last_dpr(),
        }
    }
}
