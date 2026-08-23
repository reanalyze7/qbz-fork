//! Index<->key mapping pure functions for dropdown-backed settings: language,
//! auto-theme source, startup page. Renderer/UI-scale/background/spectrum
//! mappings live in `index_maps_display.rs` (all except streaming quality,
//! which lives in `quality.rs`).

/// Default UI language key. `"auto"` follows the OS locale (resolved at startup
/// via `qbz_i18n::resolve_auto()`); otherwise one of `en` | `es` | `de` | `fr` |
/// `pt`. Persists the raw user choice ("auto" stays "auto").
pub const DEFAULT_LANGUAGE: &str = "auto";

/// Map a language select index to its persisted key. The on-screen order in
/// `AppearanceState.languages` is Auto / English / Español / Français / Deutsch
/// / Português / Русский / 日本語 / Nederlands (0-8); any unknown index falls
/// back to the default (`"auto"`).
pub fn language_for_index(index: i32) -> &'static str {
    match index {
        1 => "en",
        2 => "es",
        3 => "fr",
        4 => "de",
        5 => "pt",
        6 => "ru",
        7 => "ja",
        8 => "nl",
        _ => DEFAULT_LANGUAGE,
    }
}

/// Inverse of [`language_for_index`]: the select index for a persisted key,
/// falling back to the default's index (0 = "auto").
pub fn language_index(key: &str) -> i32 {
    match key {
        "en" => 1,
        "es" => 2,
        "fr" => 3,
        "de" => 4,
        "pt" => 5,
        "ru" => 6,
        "ja" => 7,
        "nl" => 8,
        _ => 0,
    }
}

/// Default auto-theme source key (`"system"`: DE color scheme with a wallpaper
/// fallback). The other keys are `"wallpaper"` and `"image"`.
pub const DEFAULT_AUTO_THEME_SOURCE: &str = "system";

/// Map an auto-theme-source select index to its persisted key. On-screen order
/// is System Colors / Wallpaper Sync / Custom Image (0-2); unknown indices fall
/// back to the default (`"system"`).
pub fn auto_theme_source_for_index(index: i32) -> &'static str {
    match index {
        1 => "wallpaper",
        2 => "image",
        _ => "system",
    }
}

/// Inverse of [`auto_theme_source_for_index`]: the select index for a persisted
/// key, falling back to the default's index (0 = "system").
pub fn auto_theme_source_index(key: &str) -> i32 {
    match key {
        "wallpaper" => 1,
        "image" => 2,
        _ => 0,
    }
}

/// Startup-page select index (0 = Home, 1 = Where you left off) -> key.
pub fn startup_page_for_index(index: i32) -> &'static str {
    if index == 1 {
        "remember"
    } else {
        "home"
    }
}

/// Inverse: select index for a persisted startup-page key.
pub fn startup_page_index(key: &str) -> i32 {
    if key == "remember" {
        1
    } else {
        0
    }
}
