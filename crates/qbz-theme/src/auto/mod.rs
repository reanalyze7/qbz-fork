//! Auto-theme generation: derive a full [`ThemeColors`] set from the desktop
//! environment (accent / full color scheme), the wallpaper, or a user-picked
//! image.
//!
//! This is a 1:1 logic port of the legacy Tauri `src-tauri/src/auto_theme/*`
//! modules, retargeted so the OUTPUT is the frontend-agnostic
//! [`crate::ThemeColors`] contract (ADR-006) instead of a map of CSS custom
//! properties. The palette math (k-means, WCAG contrast, HSL shifts) is
//! unchanged; only the final assembly differs — it now mirrors the registry's
//! `StdSpec::build` so a generated theme composites identically to a static one
//! (same success/focus/favorite/border-muted derivations, same polarity-driven
//! alpha ramp).

mod color;
mod color_hsl;
pub mod generator;
pub mod palette;
mod scheme;
pub mod system;

pub use color::PaletteColor;
pub use generator::{theme_from_palette, theme_from_scheme};
pub use scheme::{SystemColorScheme, ThemePalette};
pub use system::{
    detect_desktop_environment, get_system_accent_color, get_system_color_scheme,
    get_system_wallpaper, DesktopEnvironment,
};

use crate::colors::ThemeColors;

/// Where an auto theme sources its colors from. Mirrors the Tauri store's
/// `AutoThemeSource` (`'system' | 'wallpaper' | 'image'`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoSource {
    /// Full DE color scheme, with a wallpaper-extraction fallback (the store's
    /// `system` cascade: scheme → wallpaper → error).
    System,
    /// Extract a palette from the current desktop wallpaper.
    Wallpaper,
    /// Extract a palette from a user-picked image at this path.
    Image(String),
}

/// Generate a [`ThemeColors`] set for the given source.
///
/// Ports the three Tauri commands (`v2_generate_theme_from_system_colors` /
/// `_wallpaper` / `_image`) plus the store's `system` cascade (a `system`
/// request falls back to wallpaper extraction when the DE exposes no readable
/// color scheme, and only errors when both fail).
pub fn generate(source: &AutoSource) -> Result<ThemeColors, String> {
    match source {
        AutoSource::System => match system::get_system_color_scheme() {
            Ok(scheme) => Ok(generator::theme_from_scheme(&scheme)),
            Err(scheme_err) => {
                // Cascade: full color scheme → wallpaper → error (matches the
                // Tauri store's `enableAutoTheme('system')` fallback).
                let wallpaper = system::get_system_wallpaper().map_err(|wp_err| {
                    format!(
                        "Could not read system color scheme ({scheme_err}) or wallpaper ({wp_err})"
                    )
                })?;
                let palette = palette::extract_palette(&wallpaper)?;
                Ok(generator::theme_from_palette(&palette))
            }
        },
        AutoSource::Wallpaper => {
            let wallpaper = system::get_system_wallpaper()?;
            let palette = palette::extract_palette(&wallpaper)?;
            Ok(generator::theme_from_palette(&palette))
        }
        AutoSource::Image(path) => {
            let palette = palette::extract_palette(path)?;
            Ok(generator::theme_from_palette(&palette))
        }
    }
}
