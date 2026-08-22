//! Dark (branded / community) themes, part 4: `iscariot`, `stratego`, `rumi`,
//! `zoey`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

pub(super) fn iscariot() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x2a, 0x10, 0x2a),
        bg_secondary: Rgba::rgb(0x38, 0x13, 0x3b),
        bg_tertiary: Rgba::rgb(0x45, 0x18, 0x46),
        bg_hover: Rgba::rgb(0x5d, 0x20, 0x60),
        text_primary: Rgba::rgb(0xf4, 0xea, 0xf5),
        text_secondary: Rgba::rgb(0xcf, 0xaa, 0xcb),
        text_muted: Rgba::rgb(0xa2, 0x78, 0xa6),
        text_disabled: Rgba::rgb(0x5d, 0x20, 0x60),
        accent: Rgba::rgb(0xe9, 0x4f, 0x94),
        accent_hover: Rgba::rgb(0xff, 0x7a, 0xbf),
        accent_pressed: Rgba::rgb(0xc9, 0x45, 0xa3),
        accent_text: Rgba::rgb(0x2a, 0x10, 0x2a),
        danger: Rgba::rgb(0xc0, 0x39, 0x2b),
        warning: Rgba::rgb(0xe5, 0xb6, 0x4b),
        border_subtle: Rgba::rgb(0x38, 0x13, 0x3b),
        border_strong: Rgba::rgb(0x5d, 0x20, 0x60),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn stratego() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x0a, 0x0a, 0x0b),
        bg_secondary: Rgba::rgb(0x14, 0x14, 0x18),
        bg_tertiary: Rgba::rgb(0x1d, 0x1e, 0x22),
        bg_hover: Rgba::rgb(0x28, 0x2a, 0x30),
        text_primary: Rgba::rgb(0xec, 0xe6, 0xd6),
        text_secondary: Rgba::rgb(0xb5, 0xaf, 0xa0),
        text_muted: Rgba::rgb(0x8a, 0x85, 0x7a),
        text_disabled: Rgba::rgb(0x4a, 0x48, 0x42),
        accent: Rgba::rgb(0xed, 0x2f, 0x3d),
        accent_hover: Rgba::rgb(0xf7, 0x4a, 0x58),
        accent_pressed: Rgba::rgb(0xc4, 0x1e, 0x2a),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),
        danger: Rgba::rgb(0xe6, 0x39, 0x46),
        warning: Rgba::rgb(0xc4, 0xa5, 0x6a),
        border_subtle: Rgba::rgb(0x2a, 0x2a, 0x30),
        border_strong: Rgba::rgb(0x3a, 0x3a, 0x42),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn rumi() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x00, 0x00, 0x00),
        bg_secondary: Rgba::rgb(0x0d, 0x0d, 0x0d),
        bg_tertiary: Rgba::rgb(0x1a, 0x1a, 0x1a),
        bg_hover: Rgba::rgb(0x33, 0x33, 0x33),
        text_primary: Rgba::rgb(0xf0, 0xf0, 0xf0),
        text_secondary: Rgba::rgb(0xb2, 0xb2, 0xb2),
        text_muted: Rgba::rgb(0x80, 0x80, 0x80),
        text_disabled: Rgba::rgb(0x5a, 0x5a, 0x5a),
        accent: Rgba::rgb(0xe5, 0x8f, 0x24),
        accent_hover: Rgba::rgb(0xf0, 0xa5, 0x3c),
        accent_pressed: Rgba::rgb(0xcc, 0x7a, 0x12),
        accent_text: Rgba::rgb(0x00, 0x00, 0x00),
        danger: Rgba::rgb(0xe7, 0x4c, 0x3c),
        warning: Rgba::rgb(0xf3, 0x9c, 0x12),
        border_subtle: Rgba::rgb(0x1a, 0x1a, 0x1a),
        border_strong: Rgba::rgb(0x33, 0x33, 0x33),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn zoey() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x15, 0x1e, 0x2d),
        bg_secondary: Rgba::rgb(0x0e, 0x14, 0x1e),
        bg_tertiary: Rgba::rgb(0x10, 0x1a, 0x2a),
        bg_hover: Rgba::rgb(0x1b, 0x29, 0x3e),
        text_primary: Rgba::rgb(0xe0, 0xe2, 0xd5),
        text_secondary: Rgba::rgb(0xb5, 0xb7, 0xaa),
        text_muted: Rgba::rgb(0x8e, 0x90, 0x80),
        text_disabled: Rgba::rgb(0x60, 0x63, 0x54),
        accent: Rgba::rgb(0x46, 0xb4, 0xd3),
        accent_hover: Rgba::rgb(0x5c, 0xc0, 0xd9),
        accent_pressed: Rgba::rgb(0x3a, 0x97, 0xb6),
        accent_text: Rgba::rgb(0x15, 0x1e, 0x2d),
        danger: Rgba::rgb(0xbf, 0x61, 0x6a),
        warning: Rgba::rgb(0xd0, 0x87, 0x70),
        border_subtle: Rgba::rgb(0x0e, 0x14, 0x1e),
        border_strong: Rgba::rgb(0x1b, 0x29, 0x3e),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
