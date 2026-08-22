//! Dark (branded / community) themes, part 2: `catppuccin_latte` (light
//! flavor, black alpha base), `catppuccin_frappe`, `catppuccin_macchiato`,
//! `breeze_dark`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

/// catppuccin-latte — light theme (Part B §latte). Black alpha base.
pub(super) fn catppuccin_latte() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xef, 0xf1, 0xf5),
        bg_secondary: Rgba::rgb(0xe6, 0xe9, 0xef),
        bg_tertiary: Rgba::rgb(0xdc, 0xe0, 0xe8),
        bg_hover: Rgba::rgb(0xcc, 0xd0, 0xda),
        text_primary: Rgba::rgb(0x4c, 0x4f, 0x69),
        text_secondary: Rgba::rgb(0x5c, 0x5f, 0x77),
        text_muted: Rgba::rgb(0x6c, 0x6f, 0x85),
        text_disabled: Rgba::rgb(0x9c, 0xa0, 0xb0),
        accent: Rgba::rgb(0x88, 0x39, 0xef),
        accent_hover: Rgba::rgb(0x1e, 0x66, 0xf5),
        accent_pressed: Rgba::rgb(0xd2, 0x0f, 0x39),
        accent_text: Rgba::rgb(0xef, 0xf1, 0xf5),
        danger: Rgba::rgb(0xd2, 0x0f, 0x39),
        warning: Rgba::rgb(0xdf, 0x8e, 0x1d),
        border_subtle: Rgba::rgb(0xbc, 0xc0, 0xcc),
        border_strong: Rgba::rgb(0xac, 0xb0, 0xbe),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

/// catppuccin-frappe — dark theme. White alpha base.
pub(super) fn catppuccin_frappe() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x30, 0x34, 0x46),
        bg_secondary: Rgba::rgb(0x29, 0x2c, 0x3c),
        bg_tertiary: Rgba::rgb(0x23, 0x26, 0x34),
        bg_hover: Rgba::rgb(0x41, 0x45, 0x59),
        text_primary: Rgba::rgb(0xc6, 0xd0, 0xf5),
        text_secondary: Rgba::rgb(0xb5, 0xbf, 0xe2),
        text_muted: Rgba::rgb(0xa5, 0xad, 0xce),
        text_disabled: Rgba::rgb(0x73, 0x79, 0x94),
        accent: Rgba::rgb(0xca, 0x9e, 0xe6),
        accent_hover: Rgba::rgb(0x8c, 0xaa, 0xee),
        accent_pressed: Rgba::rgb(0xe7, 0x82, 0x84),
        accent_text: Rgba::rgb(0x30, 0x34, 0x46),
        danger: Rgba::rgb(0xe7, 0x82, 0x84),
        warning: Rgba::rgb(0xe5, 0xc8, 0x90),
        border_subtle: Rgba::rgb(0x51, 0x57, 0x6d),
        border_strong: Rgba::rgb(0x62, 0x68, 0x80),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

/// catppuccin-macchiato — dark theme. White alpha base.
pub(super) fn catppuccin_macchiato() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x24, 0x27, 0x3a),
        bg_secondary: Rgba::rgb(0x1e, 0x20, 0x30),
        bg_tertiary: Rgba::rgb(0x18, 0x19, 0x26),
        bg_hover: Rgba::rgb(0x36, 0x3a, 0x4f),
        text_primary: Rgba::rgb(0xca, 0xd3, 0xf5),
        text_secondary: Rgba::rgb(0xb8, 0xc0, 0xe0),
        text_muted: Rgba::rgb(0xa5, 0xad, 0xcb),
        text_disabled: Rgba::rgb(0x6e, 0x73, 0x8d),
        accent: Rgba::rgb(0xc6, 0xa0, 0xf6),
        accent_hover: Rgba::rgb(0x8a, 0xad, 0xf4),
        accent_pressed: Rgba::rgb(0xed, 0x87, 0x96),
        accent_text: Rgba::rgb(0x24, 0x27, 0x3a),
        danger: Rgba::rgb(0xed, 0x87, 0x96),
        warning: Rgba::rgb(0xee, 0xd4, 0x9f),
        border_subtle: Rgba::rgb(0x49, 0x4d, 0x64),
        border_strong: Rgba::rgb(0x5b, 0x60, 0x78),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

/// breeze-dark — OMITS the danger/warning families AND the alpha scale →
/// inherits them from `:root` Dark. We materialize the inherited danger/warning
/// hues (red `#ef4444`, amber `#fbbf24`) explicitly.
pub(super) fn breeze_dark() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x14, 0x16, 0x18),
        bg_secondary: Rgba::rgb(0x20, 0x23, 0x26),
        bg_tertiary: Rgba::rgb(0x29, 0x2c, 0x30),
        bg_hover: Rgba::rgb(0x31, 0x36, 0x3b),
        text_primary: Rgba::rgb(0xff, 0xff, 0xff),
        text_secondary: Rgba::rgb(0xfc, 0xfc, 0xfc),
        text_muted: Rgba::rgb(0xa1, 0xa9, 0xb1),
        text_disabled: Rgba::rgb(0x31, 0x36, 0x3b),
        accent: Rgba::rgb(0x3d, 0xae, 0xe9),
        accent_hover: Rgba::rgb(0x9b, 0x59, 0xb6),
        accent_pressed: Rgba::rgb(0x1d, 0x99, 0xf3),
        accent_text: Rgba::rgb(0x14, 0x16, 0x18),
        // inherited from :root Dark:
        danger: Rgba::rgb(0xef, 0x44, 0x44),
        warning: Rgba::rgb(0xfb, 0xbf, 0x24),
        border_subtle: Rgba::rgb(0x29, 0x2c, 0x30),
        border_strong: Rgba::rgb(0x31, 0x36, 0x3b),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
