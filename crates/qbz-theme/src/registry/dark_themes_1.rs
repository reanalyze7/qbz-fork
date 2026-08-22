//! Dark (branded / community) themes, part 1: `warm`, `nord`, `dracula`,
//! `catppuccin_mocha`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

pub(super) fn warm() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x2b, 0x1a, 0x14),
        bg_secondary: Rgba::rgb(0x3a, 0x24, 0x1a),
        bg_tertiary: Rgba::rgb(0x4a, 0x2f, 0x23),
        bg_hover: Rgba::rgb(0x5b, 0x3a, 0x2e),
        text_primary: Rgba::rgb(0xf5, 0xe9, 0xe2),
        text_secondary: Rgba::rgb(0xd8, 0xc3, 0xb7),
        text_muted: Rgba::rgb(0xbf, 0xa3, 0x96),
        text_disabled: Rgba::rgb(0x8d, 0x73, 0x67),
        accent: Rgba::rgb(0xe5, 0x98, 0x66),
        accent_hover: Rgba::rgb(0xf0, 0xa7, 0x7b),
        accent_pressed: Rgba::rgb(0xd8, 0x86, 0x52),
        accent_text: Rgba::rgb(0x00, 0x00, 0x00),
        danger: Rgba::rgb(0xbf, 0x4f, 0x4f),
        warning: Rgba::rgb(0xd6, 0xa9, 0x4f),
        border_subtle: Rgba::rgb(0x4a, 0x2f, 0x23),
        border_strong: Rgba::rgb(0x5b, 0x3a, 0x2e),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn nord() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x1d, 0x22, 0x30),
        bg_secondary: Rgba::rgb(0x2a, 0x2f, 0x3c),
        bg_tertiary: Rgba::rgb(0x32, 0x38, 0x4c),
        bg_hover: Rgba::rgb(0x3c, 0x42, 0x56),
        text_primary: Rgba::rgb(0xec, 0xec, 0xec),
        text_secondary: Rgba::rgb(0xc6, 0xc6, 0xc6),
        text_muted: Rgba::rgb(0x99, 0x99, 0xa3),
        text_disabled: Rgba::rgb(0x6f, 0x6f, 0x7b),
        accent: Rgba::rgb(0x35, 0x84, 0xe4),
        accent_hover: Rgba::rgb(0x5f, 0x9e, 0xe6),
        accent_pressed: Rgba::rgb(0x1a, 0x5f, 0xb4),
        accent_text: Rgba::rgb(0x24, 0x1f, 0x31),
        danger: Rgba::rgb(0xc0, 0x1c, 0x28),
        warning: Rgba::rgb(0xf5, 0xc2, 0x11),
        border_subtle: Rgba::rgb(0x2a, 0x2f, 0x3c),
        border_strong: Rgba::rgb(0x32, 0x38, 0x4c),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn dracula() -> ThemeColors {
    // NON-STANDARD tint fractions: bg .15 / border .4 / hover .25.
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x28, 0x2a, 0x36),
        bg_secondary: Rgba::rgb(0x21, 0x22, 0x2c),
        bg_tertiary: Rgba::rgb(0x34, 0x37, 0x46),
        bg_hover: Rgba::rgb(0x44, 0x47, 0x5a),
        text_primary: Rgba::rgb(0xf8, 0xf8, 0xf2),
        text_secondary: Rgba::rgb(0xe2, 0xe2, 0xdc),
        text_muted: Rgba::rgb(0x62, 0x72, 0xa4),
        text_disabled: Rgba::rgb(0x44, 0x47, 0x5a),
        accent: Rgba::rgb(0xbd, 0x93, 0xf9),
        accent_hover: Rgba::rgb(0xff, 0x79, 0xc6),
        accent_pressed: Rgba::rgb(0x8b, 0xe9, 0xfd),
        accent_text: Rgba::rgb(0x28, 0x2a, 0x36),
        danger: Rgba::rgb(0xff, 0x55, 0x55),
        warning: Rgba::rgb(0xff, 0xb8, 0x6c),
        tint_bg: 0.15,
        tint_border: 0.4,
        tint_hover: 0.25,
        border_subtle: Rgba::rgb(0x34, 0x37, 0x46),
        border_strong: Rgba::rgb(0x44, 0x47, 0x5a),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn catppuccin_mocha() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x1e, 0x1e, 0x2e),
        bg_secondary: Rgba::rgb(0x18, 0x18, 0x25),
        bg_tertiary: Rgba::rgb(0x11, 0x11, 0x1b),
        bg_hover: Rgba::rgb(0x31, 0x32, 0x44),
        text_primary: Rgba::rgb(0xcd, 0xd6, 0xf4),
        text_secondary: Rgba::rgb(0xba, 0xc2, 0xde),
        text_muted: Rgba::rgb(0xa6, 0xad, 0xc8),
        text_disabled: Rgba::rgb(0x73, 0x79, 0x94),
        accent: Rgba::rgb(0xcb, 0xa6, 0xf7),
        accent_hover: Rgba::rgb(0x89, 0xb4, 0xfa),
        accent_pressed: Rgba::rgb(0xf3, 0x8b, 0xa8),
        accent_text: Rgba::rgb(0x1e, 0x1e, 0x2e),
        danger: Rgba::rgb(0xf3, 0x8b, 0xa8),
        warning: Rgba::rgb(0xf9, 0xe2, 0xaf),
        border_subtle: Rgba::rgb(0x31, 0x32, 0x44),
        border_strong: Rgba::rgb(0x45, 0x47, 0x5a),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
