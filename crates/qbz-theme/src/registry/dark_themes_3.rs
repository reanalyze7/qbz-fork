//! Dark (branded / community) themes, part 3: `adwaita_dark`, `aurora`,
//! `ikari`, `ayanami`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

/// adwaita-dark — OMITS the danger/warning families + alpha scale → inherits
/// from `:root` Dark.
pub(super) fn adwaita_dark() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x1d, 0x1d, 0x20),
        bg_secondary: Rgba::rgb(0x22, 0x22, 0x26),
        bg_tertiary: Rgba::rgb(0x28, 0x28, 0x2c),
        bg_hover: Rgba::rgb(0x2e, 0x2e, 0x32),
        text_primary: Rgba::rgb(0xff, 0xff, 0xff),
        text_secondary: Rgba::rgb(0xff, 0xff, 0xff),
        text_muted: Rgba::rgb(0xb3, 0xb3, 0xb8),
        text_disabled: Rgba::rgb(0x2e, 0x2e, 0x32),
        accent: Rgba::rgb(0x35, 0x84, 0xe4),
        accent_hover: Rgba::rgb(0x1c, 0x71, 0xd8),
        accent_pressed: Rgba::rgb(0x1a, 0x5f, 0xb4),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),
        // inherited from :root Dark:
        danger: Rgba::rgb(0xef, 0x44, 0x44),
        warning: Rgba::rgb(0xfb, 0xbf, 0x24),
        border_subtle: Rgba::rgb(0x28, 0x28, 0x2c),
        border_strong: Rgba::rgb(0x2e, 0x2e, 0x32),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn aurora() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x2e, 0x34, 0x40),
        bg_secondary: Rgba::rgb(0x3b, 0x42, 0x52),
        bg_tertiary: Rgba::rgb(0x43, 0x4c, 0x5e),
        bg_hover: Rgba::rgb(0x4c, 0x56, 0x6a),
        text_primary: Rgba::rgb(0xd8, 0xde, 0xe9),
        text_secondary: Rgba::rgb(0xe5, 0xe9, 0xf0),
        text_muted: Rgba::rgb(0xb4, 0x8e, 0xad),
        text_disabled: Rgba::rgb(0x4c, 0x56, 0x6a),
        accent: Rgba::rgb(0xa3, 0xbe, 0x8c),
        accent_hover: Rgba::rgb(0xeb, 0xcb, 0x8b),
        accent_pressed: Rgba::rgb(0xd0, 0x87, 0x70),
        accent_text: Rgba::rgb(0x2e, 0x34, 0x40),
        danger: Rgba::rgb(0xbf, 0x61, 0x6a),
        warning: Rgba::rgb(0xeb, 0xcb, 0x8b),
        border_subtle: Rgba::rgb(0x4c, 0x56, 0x6a),
        border_strong: Rgba::rgb(0x43, 0x4c, 0x5e),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn ikari() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x1c, 0x12, 0x39),
        bg_secondary: Rgba::rgb(0x24, 0x1a, 0x48),
        bg_tertiary: Rgba::rgb(0x30, 0x24, 0x58),
        bg_hover: Rgba::rgb(0x3c, 0x2f, 0x71),
        text_primary: Rgba::rgb(0xe8, 0xe6, 0xf2),
        text_secondary: Rgba::rgb(0xc6, 0xc2, 0xd8),
        text_muted: Rgba::rgb(0x95, 0x8f, 0xb5),
        text_disabled: Rgba::rgb(0x57, 0x4b, 0x79),
        accent: Rgba::rgb(0x7e, 0xda, 0x53),
        accent_hover: Rgba::rgb(0xa5, 0xf0, 0x66),
        accent_pressed: Rgba::rgb(0xd5, 0x8e, 0x27),
        accent_text: Rgba::rgb(0x1c, 0x12, 0x39),
        danger: Rgba::rgb(0xd8, 0x4a, 0x4a),
        warning: Rgba::rgb(0xe5, 0x9b, 0x2f),
        border_subtle: Rgba::rgb(0x30, 0x24, 0x58),
        border_strong: Rgba::rgb(0x3c, 0x2f, 0x71),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn ayanami() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x0f, 0x25, 0x3f),
        bg_secondary: Rgba::rgb(0x16, 0x3e, 0x60),
        bg_tertiary: Rgba::rgb(0x21, 0x4f, 0x7d),
        bg_hover: Rgba::rgb(0x2d, 0x63, 0x9f),
        text_primary: Rgba::rgb(0xf2, 0xf0, 0xe5),
        text_secondary: Rgba::rgb(0xd6, 0xd2, 0xc2),
        text_muted: Rgba::rgb(0x95, 0xa4, 0xb7),
        text_disabled: Rgba::rgb(0x2d, 0x63, 0x9f),
        accent: Rgba::rgb(0xe5, 0xb8, 0x2e),
        accent_hover: Rgba::rgb(0xf0, 0xcd, 0x63),
        accent_pressed: Rgba::rgb(0xcf, 0xa2, 0x2e),
        accent_text: Rgba::rgb(0x0f, 0x25, 0x3f),
        danger: Rgba::rgb(0xc0, 0x39, 0x2b),
        warning: Rgba::rgb(0xd8, 0x9b, 0x1c),
        border_subtle: Rgba::rgb(0x21, 0x4f, 0x7d),
        border_strong: Rgba::rgb(0x2d, 0x63, 0x9f),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
