//! `ThemeColors` field-by-field mapping from the registry's `Rgba` to the
//! generated Slint struct.

use crate::ThemeColors as SlintThemeColors;
use qbz_theme::Rgba;
use slint::Color;

/// Convert a registry `Rgba` to a Slint `Color` (straight alpha).
fn to_color(c: Rgba) -> Color {
    Color::from_argb_u8(c.a, c.r, c.g, c.b)
}

/// Build the generated Slint `ThemeColors` from a registry `ThemeColors`.
pub(super) fn to_slint(c: &qbz_theme::ThemeColors) -> SlintThemeColors {
    SlintThemeColors {
        surface_main: to_color(c.surface_main),
        surface_card: to_color(c.surface_card),
        surface_elevated: to_color(c.surface_elevated),
        surface_hover: to_color(c.surface_hover),
        bg_hover: to_color(c.bg_hover),

        text_primary: to_color(c.text_primary),
        text_secondary: to_color(c.text_secondary),
        text_muted: to_color(c.text_muted),
        text_disabled: to_color(c.text_disabled),

        accent: to_color(c.accent),
        accent_hover: to_color(c.accent_hover),
        accent_pressed: to_color(c.accent_pressed),
        accent_text: to_color(c.accent_text),

        danger: to_color(c.danger),
        danger_bg: to_color(c.danger_bg),
        danger_border: to_color(c.danger_border),
        danger_hover: to_color(c.danger_hover),

        warning: to_color(c.warning),
        warning_bg: to_color(c.warning_bg),
        warning_border: to_color(c.warning_border),
        warning_hover: to_color(c.warning_hover),

        success: to_color(c.success),
        success_bg: to_color(c.success_bg),
        success_border: to_color(c.success_border),
        success_hover: to_color(c.success_hover),

        border_subtle: to_color(c.border_subtle),
        border_muted: to_color(c.border_muted),
        border_strong: to_color(c.border_strong),

        focus_ring: to_color(c.focus_ring),

        favorite: to_color(c.favorite),
        card_shadow: to_color(c.card_shadow),

        alpha_4: to_color(c.alpha_pct(4)),
        alpha_5: to_color(c.alpha_pct(5)),
        alpha_6: to_color(c.alpha_pct(6)),
        alpha_8: to_color(c.alpha_pct(8)),
        alpha_10: to_color(c.alpha_pct(10)),
        alpha_12: to_color(c.alpha_pct(12)),
        alpha_15: to_color(c.alpha_pct(15)),
        alpha_18: to_color(c.alpha_pct(18)),
        alpha_20: to_color(c.alpha_pct(20)),
        alpha_25: to_color(c.alpha_pct(25)),
        alpha_30: to_color(c.alpha_pct(30)),
        alpha_35: to_color(c.alpha_pct(35)),
        alpha_40: to_color(c.alpha_pct(40)),
        alpha_45: to_color(c.alpha_pct(45)),
        alpha_50: to_color(c.alpha_pct(50)),
        alpha_55: to_color(c.alpha_pct(55)),
        alpha_60: to_color(c.alpha_pct(60)),
        alpha_65: to_color(c.alpha_pct(65)),
        alpha_70: to_color(c.alpha_pct(70)),
        alpha_75: to_color(c.alpha_pct(75)),
        alpha_80: to_color(c.alpha_pct(80)),
        alpha_85: to_color(c.alpha_pct(85)),
        alpha_90: to_color(c.alpha_pct(90)),
        alpha_95: to_color(c.alpha_pct(95)),
    }
}
