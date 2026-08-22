//! Pure data carriers used by [`super::palette`], [`super::system`], and
//! [`super::generator`]: the extracted image palette and the full DE color
//! scheme.

use serde::{Deserialize, Serialize};

use super::color::PaletteColor;

/// Extracted palette from an image (dominant surfaces + accent + polarity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    pub bg_primary: PaletteColor,
    pub bg_secondary: PaletteColor,
    pub bg_tertiary: PaletteColor,
    pub bg_hover: PaletteColor,
    pub accent: PaletteColor,
    pub is_dark: bool,
    pub all_colors: Vec<PaletteColor>,
}

/// Full color scheme read from the desktop environment (KDE kdeglobals, GNOME
/// dconf, …). Each field is optional because not all DEs expose all roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemColorScheme {
    // Backgrounds
    pub window_bg: Option<PaletteColor>,
    pub window_bg_alt: Option<PaletteColor>,
    pub view_bg: Option<PaletteColor>,
    pub button_bg: Option<PaletteColor>,
    pub header_bg: Option<PaletteColor>,
    pub header_bg_inactive: Option<PaletteColor>,
    pub tooltip_bg: Option<PaletteColor>,

    // Foregrounds (text)
    pub window_fg: Option<PaletteColor>,
    pub window_fg_inactive: Option<PaletteColor>,
    pub view_fg: Option<PaletteColor>,
    pub button_fg: Option<PaletteColor>,

    // Selection / accent
    pub selection_bg: Option<PaletteColor>,
    pub selection_fg: Option<PaletteColor>,
    pub selection_hover: Option<PaletteColor>,
    pub accent: Option<PaletteColor>,

    // Semantic
    pub fg_link: Option<PaletteColor>,
    pub fg_negative: Option<PaletteColor>,
    pub fg_neutral: Option<PaletteColor>,
    pub fg_positive: Option<PaletteColor>,

    // Window manager
    pub wm_active_bg: Option<PaletteColor>,
    pub wm_active_fg: Option<PaletteColor>,
    pub wm_inactive_bg: Option<PaletteColor>,
}
