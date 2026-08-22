use super::*;
use crate::auto::{PaletteColor, SystemColorScheme, ThemePalette};
use crate::color::Rgba;
use crate::colors::ALPHA_COUNT;

fn dark_palette() -> ThemePalette {
    ThemePalette {
        bg_primary: PaletteColor::new(15, 15, 20),
        bg_secondary: PaletteColor::new(26, 26, 30),
        bg_tertiary: PaletteColor::new(42, 42, 48),
        bg_hover: PaletteColor::new(31, 31, 35),
        accent: PaletteColor::new(66, 133, 244),
        is_dark: true,
        all_colors: vec![],
    }
}

fn light_palette() -> ThemePalette {
    ThemePalette {
        bg_primary: PaletteColor::new(245, 245, 245),
        bg_secondary: PaletteColor::new(235, 235, 235),
        bg_tertiary: PaletteColor::new(220, 220, 220),
        bg_hover: PaletteColor::new(240, 240, 240),
        accent: PaletteColor::new(26, 115, 232),
        is_dark: false,
        all_colors: vec![],
    }
}

#[test]
fn dark_polarity_white_alpha_base() {
    let c = theme_from_palette(&dark_palette());
    assert_eq!(c.alpha.len(), ALPHA_COUNT);
    // Dark themes get a WHITE-based alpha ramp + translucent edges.
    assert_eq!(c.alpha[c.alpha.len() - 1].r, 255);
    assert_eq!(c.surface_hover, Rgba::rgba(255, 255, 255, 0x10));
    assert_eq!(c.border_muted, Rgba::rgba(255, 255, 255, 0x38));
    // Surfaces map straight through.
    assert_eq!(c.surface_main, Rgba::rgb(15, 15, 20));
    assert_eq!(c.surface_card, Rgba::rgb(26, 26, 30));
    assert_eq!(c.surface_elevated, Rgba::rgb(42, 42, 48));
    assert_eq!(c.bg_hover, Rgba::rgb(31, 31, 35));
    // Accent maps straight; focus_ring == accent; favorite == danger.
    assert_eq!(c.accent, Rgba::rgb(66, 133, 244));
    assert_eq!(c.focus_ring, c.accent);
    assert_eq!(c.favorite, c.danger);
    // Dark success hue.
    assert_eq!(c.success, Rgba::rgb(0x3f, 0xae, 0x6a));
}

#[test]
fn light_polarity_black_alpha_base() {
    let c = theme_from_palette(&light_palette());
    assert_eq!(c.alpha[c.alpha.len() - 1].r, 0);
    assert_eq!(c.surface_hover, Rgba::rgba(0, 0, 0, 0x10));
    assert_eq!(c.border_muted, Rgba::rgba(0, 0, 0, 0x38));
    // Light success hue (darker so it clears >=3:1 on a light surface).
    assert_eq!(c.success, Rgba::rgb(0x1f, 0x8a, 0x4c));
    // Light danger hue.
    assert_eq!(c.danger, Rgba::rgb(220, 38, 38));
}

#[test]
fn derived_status_families_have_expected_tints() {
    let c = theme_from_palette(&dark_palette());
    // bg = 0.1, border = 0.3, hover = 0.2 (dark) — straight-alpha of the hue.
    assert_eq!(c.danger_bg.a, (0.1f32 * 255.0 + 0.5) as u8);
    assert_eq!(c.danger_border.a, (0.3f32 * 255.0 + 0.5) as u8);
    assert_eq!(c.danger_hover.a, (0.2f32 * 255.0 + 0.5) as u8);
    assert_eq!(c.danger_bg.r, c.danger.r);
    assert_eq!(c.success_hover.a, c.danger_hover.a);
}

#[test]
fn deterministic_for_fixed_seed() {
    let a = theme_from_palette(&dark_palette());
    let b = theme_from_palette(&dark_palette());
    assert_eq!(a, b);
}

#[test]
fn scheme_polarity_from_window_bg() {
    let mut scheme = SystemColorScheme {
        window_bg: Some(PaletteColor::new(30, 30, 30)),
        window_bg_alt: None,
        view_bg: None,
        button_bg: None,
        header_bg: None,
        header_bg_inactive: None,
        tooltip_bg: None,
        window_fg: None,
        window_fg_inactive: None,
        view_fg: None,
        button_fg: None,
        selection_bg: None,
        selection_fg: None,
        selection_hover: None,
        accent: Some(PaletteColor::new(66, 133, 244)),
        fg_link: None,
        fg_negative: None,
        fg_neutral: None,
        fg_positive: None,
        wm_active_bg: None,
        wm_active_fg: None,
        wm_inactive_bg: None,
    };
    let dark = theme_from_scheme(&scheme);
    assert_eq!(dark.surface_main, Rgba::rgb(30, 30, 30));
    assert_eq!(dark.alpha[dark.alpha.len() - 1].r, 255); // dark -> white base
    assert_eq!(dark.accent, Rgba::rgb(66, 133, 244));

    scheme.window_bg = Some(PaletteColor::new(240, 240, 240));
    let light = theme_from_scheme(&scheme);
    assert_eq!(light.alpha[light.alpha.len() - 1].r, 0); // light -> black base
}
