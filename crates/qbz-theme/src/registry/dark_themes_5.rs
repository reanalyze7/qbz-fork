//! Dark (branded / community) themes, part 5: `mira`, `frost`, `langley`.

use crate::color::Rgba;
use crate::colors::ThemeColors;

use super::std_spec::{bg_is_light, StdSpec};

pub(super) fn mira() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x10, 0x18, 0x20),
        bg_secondary: Rgba::rgb(0x14, 0x1a, 0x28),
        bg_tertiary: Rgba::rgb(0x1d, 0x26, 0x35),
        bg_hover: Rgba::rgb(0x2a, 0x34, 0x48),
        text_primary: Rgba::rgb(0xe5, 0xe5, 0xe5),
        text_secondary: Rgba::rgb(0xb0, 0xb3, 0xc6),
        text_muted: Rgba::rgb(0x8a, 0x8d, 0xa0),
        text_disabled: Rgba::rgb(0x5c, 0x5e, 0x72),
        accent: Rgba::rgb(0xd9, 0x46, 0x85),
        accent_hover: Rgba::rgb(0xff, 0x00, 0x7f),
        accent_pressed: Rgba::rgb(0xff, 0xd7, 0x00), // intentional yellow
        accent_text: Rgba::rgb(0x10, 0x18, 0x20),
        danger: Rgba::rgb(0xc5, 0x30, 0x32),
        warning: Rgba::rgb(0xff, 0xd7, 0x00),
        border_subtle: Rgba::rgb(0x20, 0x2c, 0x3d),
        border_strong: Rgba::rgb(0x34, 0x40, 0x5a),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

/// frost — registered `type:light` in Tauri but a DARK Nord-polar canvas
/// (`#2e3440`). Polarity is luminance-derived, so it correctly resolves to a
/// white alpha base. (doc 01 §frost; corrected light/dark flag.)
pub(super) fn frost() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x2e, 0x34, 0x40),
        bg_secondary: Rgba::rgb(0x3b, 0x42, 0x52),
        bg_tertiary: Rgba::rgb(0x43, 0x4c, 0x5e),
        bg_hover: Rgba::rgb(0x4c, 0x56, 0x6a),
        text_primary: Rgba::rgb(0xd8, 0xde, 0xe9),
        text_secondary: Rgba::rgb(0xe5, 0xe9, 0xf0),
        text_muted: Rgba::rgb(0x8f, 0xbc, 0xbb),
        text_disabled: Rgba::rgb(0x4c, 0x56, 0x6a),
        accent: Rgba::rgb(0x88, 0xc0, 0xd0),
        accent_hover: Rgba::rgb(0x81, 0xa1, 0xc1),
        accent_pressed: Rgba::rgb(0x5e, 0x81, 0xac),
        accent_text: Rgba::rgb(0x2e, 0x34, 0x40),
        danger: Rgba::rgb(0xbf, 0x61, 0x6a),
        warning: Rgba::rgb(0xd0, 0x87, 0x70),
        border_subtle: Rgba::rgb(0x4c, 0x56, 0x6a),
        border_strong: Rgba::rgb(0x43, 0x4c, 0x5e),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}

/// langley — registered `type:light` in Tauri but a DEEP-MAROON dark canvas
/// (`#2c0a0a`). Luminance-derived polarity -> white alpha base. (doc 01 §langley)
pub(super) fn langley() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0x2c, 0x0a, 0x0a),
        bg_secondary: Rgba::rgb(0x3a, 0x0e, 0x0e),
        bg_tertiary: Rgba::rgb(0x4c, 0x14, 0x13),
        bg_hover: Rgba::rgb(0x71, 0x1c, 0x1c),
        text_primary: Rgba::rgb(0xf2, 0xda, 0xda),
        text_secondary: Rgba::rgb(0xd9, 0xa3, 0xa3),
        text_muted: Rgba::rgb(0xa9, 0x7b, 0x7b),
        text_disabled: Rgba::rgb(0x7a, 0x3d, 0x3d),
        accent: Rgba::rgb(0xe6, 0x7e, 0x22),
        accent_hover: Rgba::rgb(0xf3, 0x9c, 0x4d),
        accent_pressed: Rgba::rgb(0xd8, 0x6b, 0x1f),
        accent_text: Rgba::rgb(0x2c, 0x0a, 0x0a),
        danger: Rgba::rgb(0xc0, 0x39, 0x2b),
        warning: Rgba::rgb(0xe5, 0xa6, 0x3d),
        border_subtle: Rgba::rgb(0x3a, 0x0e, 0x0e),
        border_strong: Rgba::rgb(0x4c, 0x14, 0x13),
        ..Default::default()
    };
    s.build(bg_is_light(s.bg_primary))
}
