//! Accessibility (REDESIGNED) themes — light polarity: `wcag_light`,
//! `high_contrast_light`. Final verified palettes (Part B).
//!
//! Unlike the standard rows, the a11y themes specify SOLID (opaque) status
//! surfaces and borders, not rgba() alpha tints — accessible contrast can't be
//! guaranteed through translucency over an arbitrary backdrop. So these rows are
//! materialized directly rather than via `StdSpec`.

use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

use super::LEGACY_CARD_SHADOW;

/// `wcag-light` — WCAG AAA Light (Part B §B.1). Body text AAA (7:1) + APCA ≥75;
/// non-text ≥3:1. text-primary `#1a1a1a` (not pure black) to avoid reverse-halation.
pub(super) fn wcag_light() -> ThemeColors {
    let danger = Rgba::rgb(0xa3, 0x00, 0x00);
    let warning = Rgba::rgb(0x6b, 0x45, 0x00);
    // derived success: deep green clearing AAA on white (7.36:1).
    let success = Rgba::rgb(0x13, 0x63, 0x2f);
    let accent = Rgba::rgb(0x0a, 0x4e, 0xa3);
    ThemeColors {
        surface_main: Rgba::rgb(0xff, 0xff, 0xff),     // bg-primary
        surface_card: Rgba::rgb(0xf4, 0xf5, 0xf7),     // bg-secondary
        surface_elevated: Rgba::rgb(0xe7, 0xe9, 0xee), // bg-tertiary
        surface_hover: Rgba::rgba(0, 0, 0, 0x10),      // ~6% black (light polarity)
        bg_hover: Rgba::rgb(0xdd, 0xe0, 0xe6),         // bg-hover

        text_primary: Rgba::rgb(0x1a, 0x1a, 0x1a),
        text_secondary: Rgba::rgb(0x3a, 0x3a, 0x3a),
        text_muted: Rgba::rgb(0x59, 0x59, 0x59),
        text_disabled: Rgba::rgb(0x76, 0x76, 0x76),

        accent,
        accent_hover: Rgba::rgb(0x08, 0x3d, 0x80),
        accent_pressed: Rgba::rgb(0x06, 0x2e, 0x60),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff), // btn-primary-text

        danger,
        danger_bg: Rgba::rgb(0xfb, 0xe9, 0xe9),     // solid
        danger_border: Rgba::rgb(0xaa, 0x60, 0x60), // solid
        danger_hover: Rgba::rgb(0x85, 0x00, 0x00),

        warning,
        warning_bg: Rgba::rgb(0xff, 0xf7, 0xe6),     // solid
        warning_border: Rgba::rgb(0x9c, 0x73, 0x20), // solid
        warning_hover: Rgba::rgb(0x55, 0x37, 0x00),

        success,
        success_bg: Rgba::rgb(0xe6, 0xf4, 0xea),     // solid
        success_border: Rgba::rgb(0x5a, 0x9c, 0x72), // solid
        success_hover: Rgba::rgb(0x0f, 0x4f, 0x25),

        border_subtle: Rgba::rgb(0xc9, 0xcc, 0xd2), // decorative divider
        border_muted: Rgba::rgba(0, 0, 0, 0x38),    // ~22% black
        border_strong: Rgba::rgb(0x6e, 0x6e, 0x6e), // control border

        focus_ring: accent, // reuses accent

        favorite: danger,
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(true), // light theme -> black-based overlays
    }
}

/// `high-contrast-light` (LIGHT, new) — Part B §B.3b. Reciprocal deep-blue
/// accent. Warning corrected `#735c00` → `#5e4b00` (AA-only → AAA on white).
pub(super) fn high_contrast_light() -> ThemeColors {
    let danger = Rgba::rgb(0xa3, 0x00, 0x00);
    let warning = Rgba::rgb(0x5e, 0x4b, 0x00); // CORRECTED from #735c00
    // derived success: deep green ≥ HC bar (8.47:1 on white).
    let success = Rgba::rgb(0x00, 0x5a, 0x1c);
    let accent = Rgba::rgb(0x00, 0x00, 0xcc);
    ThemeColors {
        surface_main: Rgba::rgb(0xff, 0xff, 0xff),
        surface_card: Rgba::rgb(0xf2, 0xf2, 0xf2),
        surface_elevated: Rgba::rgb(0xe6, 0xe6, 0xe6),
        surface_hover: Rgba::rgba(0, 0, 0, 0x10),
        bg_hover: Rgba::rgb(0xda, 0xda, 0xda),

        text_primary: Rgba::rgb(0x00, 0x00, 0x00),
        text_secondary: Rgba::rgb(0x1a, 0x1a, 0x1a), // near-primary
        text_muted: Rgba::rgb(0x33, 0x33, 0x33),     // near-primary, NOT gray
        text_disabled: Rgba::rgb(0x59, 0x59, 0x59),  // reserved gray (7.00:1 AAA)

        accent,                                      // reciprocal deep blue
        accent_hover: Rgba::rgb(0x00, 0x00, 0xa3),
        accent_pressed: Rgba::rgb(0x00, 0x00, 0x80),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),    // reads on blue fill

        danger,
        danger_bg: Rgba::rgb(0xff, 0xe5, 0xe5),      // opaque; always bordered
        danger_border: danger,                       // = danger hue
        danger_hover: Rgba::rgb(0x7a, 0x00, 0x00),

        warning,
        warning_bg: Rgba::rgb(0xff, 0xf4, 0xd6),     // opaque; always bordered
        warning_border: warning,                     // = corrected warning hue
        warning_hover: Rgba::rgb(0x5c, 0x49, 0x00),

        success,
        success_bg: Rgba::rgb(0xdc, 0xf2, 0xe2),     // opaque; always bordered
        success_border: success,                     // = success hue
        success_hover: Rgba::rgb(0x00, 0x44, 0x17),

        border_subtle: Rgba::rgb(0x66, 0x66, 0x66), // clearly visible (5.74:1)
        border_muted: Rgba::rgba(0, 0, 0, 0x38),
        border_strong: Rgba::rgb(0x00, 0x00, 0x00), // = text color

        focus_ring: accent, // accent doubles as focus ring

        favorite: danger,
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(true), // light theme -> black-based overlays
    }
}
