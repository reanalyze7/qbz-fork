//! Light (branded / community) themes, part 1: `alucard`, `rose_pine_dawn`,
//! `breeze_light`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

/// alucard — light/cream theme (`#fffbeb` canvas). Luminance-derived -> black
/// alpha base. (doc 01 §alucard.)
pub(super) fn alucard() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xff, 0xfb, 0xeb),
        bg_secondary: Rgba::rgb(0xef, 0xed, 0xdc),
        bg_tertiary: Rgba::rgb(0xec, 0xe9, 0xdf),
        bg_hover: Rgba::rgb(0xcf, 0xcf, 0xde),
        text_primary: Rgba::rgb(0x1f, 0x1f, 0x1f),
        text_secondary: Rgba::rgb(0x6c, 0x66, 0x4b),
        text_muted: Rgba::rgb(0x9b, 0x92, 0x75),
        text_disabled: Rgba::rgb(0xbc, 0xba, 0xb3),
        accent: Rgba::rgb(0x64, 0x4a, 0xc9),
        accent_hover: Rgba::rgb(0xa3, 0x14, 0x4d),
        accent_pressed: Rgba::rgb(0x03, 0x6a, 0x96),
        accent_text: Rgba::rgb(0xff, 0xfb, 0xeb),
        danger: Rgba::rgb(0xcb, 0x3a, 0x2a),
        warning: Rgba::rgb(0xa3, 0x4d, 0x14),
        border_subtle: Rgba::rgb(0xec, 0xe9, 0xdf),
        border_strong: Rgba::rgb(0xde, 0xdc, 0xcf),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn rose_pine_dawn() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xfa, 0xf4, 0xed),
        bg_secondary: Rgba::rgb(0xf4, 0xed, 0xe8),
        bg_tertiary: Rgba::rgb(0xdf, 0xda, 0xd9),
        bg_hover: Rgba::rgb(0xce, 0xca, 0xcd),
        text_primary: Rgba::rgb(0x57, 0x52, 0x79),
        text_secondary: Rgba::rgb(0x79, 0x75, 0x93),
        text_muted: Rgba::rgb(0x98, 0x93, 0xa5),
        text_disabled: Rgba::rgb(0xb5, 0xae, 0xbc),
        accent: Rgba::rgb(0xd7, 0x82, 0x7e),
        accent_hover: Rgba::rgb(0xe5, 0xa4, 0x78),
        accent_pressed: Rgba::rgb(0x28, 0x69, 0x83),
        accent_text: Rgba::rgb(0x57, 0x52, 0x79),
        danger: Rgba::rgb(0xb4, 0x63, 0x7a),
        warning: Rgba::rgb(0xea, 0x9d, 0x34),
        border_subtle: Rgba::rgb(0xce, 0xca, 0xcd),
        border_strong: Rgba::rgb(0xdf, 0xda, 0xd9),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

pub(super) fn breeze_light() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xff, 0xff, 0xff),
        bg_secondary: Rgba::rgb(0xf2, 0xf2, 0xf2),
        bg_tertiary: Rgba::rgb(0xe5, 0xe5, 0xe5),
        bg_hover: Rgba::rgb(0xdc, 0xdc, 0xdc),
        text_primary: Rgba::rgb(0x31, 0x36, 0x3b),
        text_secondary: Rgba::rgb(0x5c, 0x61, 0x66),
        text_muted: Rgba::rgb(0x7d, 0x81, 0x86),
        text_disabled: Rgba::rgb(0xa1, 0xa5, 0xa9),
        accent: Rgba::rgb(0x1d, 0x99, 0xf3),
        accent_hover: Rgba::rgb(0x3d, 0xae, 0xe9),
        accent_pressed: Rgba::rgb(0x00, 0x78, 0xd4),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),
        danger: Rgba::rgb(0xc3, 0x27, 0x2b),
        warning: Rgba::rgb(0xf5, 0x97, 0x00),
        border_subtle: Rgba::rgb(0xd0, 0xd4, 0xd8),
        border_strong: Rgba::rgb(0xb7, 0xbd, 0xc2),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
