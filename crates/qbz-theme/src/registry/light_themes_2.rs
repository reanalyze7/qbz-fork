//! Light (branded / community) themes, part 2: `adwaita_light`,
//! `duotone_snow`, `snow_storm`, `kurosaki`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

pub(super) fn adwaita_light() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xff, 0xff, 0xff),
        bg_secondary: Rgba::rgb(0xf6, 0xf5, 0xf4),
        bg_tertiary: Rgba::rgb(0xea, 0xe9, 0xe7),
        bg_hover: Rgba::rgb(0xdc, 0xd9, 0xd7),
        text_primary: Rgba::rgb(0x24, 0x1f, 0x31),
        text_secondary: Rgba::rgb(0x5f, 0x5b, 0x6b),
        text_muted: Rgba::rgb(0x7f, 0x7b, 0x8c),
        text_disabled: Rgba::rgb(0xb1, 0xae, 0xbc),
        accent: Rgba::rgb(0x1e, 0x78, 0xe4),
        accent_hover: Rgba::rgb(0x3f, 0x8e, 0xf0),
        accent_pressed: Rgba::rgb(0x15, 0x5a, 0x9c),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),
        danger: Rgba::rgb(0xc0, 0x1c, 0x28),
        warning: Rgba::rgb(0xf5, 0xc2, 0x11),
        border_subtle: Rgba::rgb(0xdc, 0xd9, 0xd7),
        border_strong: Rgba::rgb(0xc6, 0xc2, 0xcf),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn duotone_snow() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xff, 0xff, 0xff),
        bg_secondary: Rgba::rgb(0xf8, 0xf9, 0xfa),
        bg_tertiary: Rgba::rgb(0xef, 0xf1, 0xf5),
        bg_hover: Rgba::rgb(0xe6, 0xe8, 0xec),
        text_primary: Rgba::rgb(0x4a, 0x59, 0x6e),
        text_secondary: Rgba::rgb(0x6b, 0x73, 0x8a),
        text_muted: Rgba::rgb(0x8c, 0x95, 0xa8),
        text_disabled: Rgba::rgb(0xb0, 0xb4, 0xc1),
        accent: Rgba::rgb(0x4a, 0x82, 0xd8),
        accent_hover: Rgba::rgb(0x6b, 0x9b, 0xe0),
        accent_pressed: Rgba::rgb(0x3a, 0x6f, 0xc2),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),
        danger: Rgba::rgb(0xd3, 0x7e, 0x7e),
        warning: Rgba::rgb(0xc0, 0x9c, 0x4a),
        border_subtle: Rgba::rgb(0xdf, 0xe3, 0xe8),
        border_strong: Rgba::rgb(0xc9, 0xce, 0xd4),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn snow_storm() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xec, 0xef, 0xf4),
        bg_secondary: Rgba::rgb(0xe5, 0xe9, 0xf0),
        bg_tertiary: Rgba::rgb(0xd8, 0xde, 0xe9),
        bg_hover: Rgba::rgb(0xcb, 0xd5, 0xe0),
        text_primary: Rgba::rgb(0x2e, 0x34, 0x40),
        text_secondary: Rgba::rgb(0x3b, 0x42, 0x52),
        text_muted: Rgba::rgb(0x43, 0x4c, 0x5e),
        text_disabled: Rgba::rgb(0x4c, 0x56, 0x6a),
        accent: Rgba::rgb(0x5e, 0x81, 0xac),
        accent_hover: Rgba::rgb(0x81, 0xa1, 0xc1),
        accent_pressed: Rgba::rgb(0x88, 0xc0, 0xd0),
        accent_text: Rgba::rgb(0x2e, 0x34, 0x40),
        danger: Rgba::rgb(0xbf, 0x61, 0x6a),
        warning: Rgba::rgb(0xd0, 0x87, 0x70),
        border_subtle: Rgba::rgb(0xd8, 0xde, 0xe9),
        border_strong: Rgba::rgb(0xe5, 0xe9, 0xf0),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn kurosaki() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xfb, 0xf9, 0xf2),
        bg_secondary: Rgba::rgb(0xf3, 0xf0, 0xe8),
        bg_tertiary: Rgba::rgb(0xe8, 0xe2, 0xd4),
        bg_hover: Rgba::rgb(0xe1, 0xda, 0xc8),
        text_primary: Rgba::rgb(0x26, 0x24, 0x24),
        text_secondary: Rgba::rgb(0x54, 0x4d, 0x48),
        text_muted: Rgba::rgb(0x82, 0x7d, 0x78),
        text_disabled: Rgba::rgb(0xb3, 0xad, 0xa7),
        accent: Rgba::rgb(0xd5, 0xbe, 0x58),
        accent_hover: Rgba::rgb(0xe8, 0xce, 0x66),
        accent_pressed: Rgba::rgb(0xb4, 0x9f, 0x45),
        accent_text: Rgba::rgb(0x26, 0x24, 0x24),
        danger: Rgba::rgb(0xc0, 0x39, 0x2b),
        warning: Rgba::rgb(0xd8, 0x9b, 0x1c),
        border_subtle: Rgba::rgb(0xe4, 0xdc, 0xb4),
        border_strong: Rgba::rgb(0xd1, 0xc7, 0xaa),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
