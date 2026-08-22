//! Accessibility (REDESIGNED) themes — dark polarity: `wcag_dark`,
//! `high_contrast`. Final verified palettes (Part B). See `a11y_light.rs` for
//! why these are materialized directly instead of via `StdSpec`.

use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

use super::LEGACY_CARD_SHADOW;

/// `wcag-dark` — WCAG AAA Dark (Part B §B.2). AAA (7:1) + APCA content/body;
/// non-text ≥3:1. bg `#0d1117` + text `#e6edf3` to kill halation.
pub(super) fn wcag_dark() -> ThemeColors {
    let danger = Rgba::rgb(0xff, 0x9d, 0x9d);
    let warning = Rgba::rgb(0xff, 0xca, 0x6a);
    // derived success: lightened green clearing the AAA bar on dark (11.77:1).
    let success = Rgba::rgb(0x7e, 0xe0, 0xa0);
    let accent = Rgba::rgb(0x9e, 0xc1, 0xff);
    ThemeColors {
        surface_main: Rgba::rgb(0x0d, 0x11, 0x17),
        surface_card: Rgba::rgb(0x16, 0x1b, 0x22),
        surface_elevated: Rgba::rgb(0x21, 0x26, 0x2d),
        surface_hover: Rgba::rgba(255, 255, 255, 0x10), // ~6% white
        bg_hover: Rgba::rgb(0x2a, 0x31, 0x3a),

        text_primary: Rgba::rgb(0xe6, 0xed, 0xf3),
        text_secondary: Rgba::rgb(0xc9, 0xd1, 0xd9),
        text_muted: Rgba::rgb(0xb8, 0xc0, 0xcc),
        text_disabled: Rgba::rgb(0x7d, 0x87, 0x94),

        accent,
        accent_hover: Rgba::rgb(0xb9, 0xd2, 0xff),
        accent_pressed: Rgba::rgb(0xcf, 0xe0, 0xff),
        accent_text: Rgba::rgb(0x06, 0x09, 0x0f), // dark text on light-blue

        danger,
        danger_bg: Rgba::rgb(0x2d, 0x14, 0x16),     // opaque dark-red tint
        danger_border: Rgba::rgb(0xa8, 0x56, 0x56), // see adjacency constraint
        danger_hover: Rgba::rgb(0xff, 0xb3, 0xb3),

        warning,
        warning_bg: Rgba::rgb(0x2d, 0x24, 0x10),     // opaque dark-amber tint
        warning_border: Rgba::rgb(0x9c, 0x74, 0x30),
        warning_hover: Rgba::rgb(0xff, 0xd9, 0x8a),

        success,
        success_bg: Rgba::rgb(0x11, 0x24, 0x1a),     // opaque dark-green tint
        success_border: Rgba::rgb(0x3f, 0x7d, 0x56),
        success_hover: Rgba::rgb(0x9a, 0xed, 0xb6),

        border_subtle: Rgba::rgb(0x2d, 0x33, 0x3b), // decorative separator
        border_muted: Rgba::rgba(255, 255, 255, 0x38),
        border_strong: Rgba::rgb(0x6b, 0x76, 0x86), // control border (≥3:1 all tiers)

        focus_ring: accent,

        favorite: danger,
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(false), // dark theme -> white-based overlays
    }
}

/// `high-contrast` (DARK) — Part B §B.3a. Lifted off pure black (`#0a0a0a`),
/// reciprocal cyan accent, bright yellow demoted to the focus-ring slot.
pub(super) fn high_contrast() -> ThemeColors {
    let danger = Rgba::rgb(0xff, 0x8a, 0x8a);
    let warning = Rgba::rgb(0xff, 0xb0, 0x00);
    // derived success: bright green ≥ wcag-dark bar (15.51:1 on bg-primary).
    let success = Rgba::rgb(0x62, 0xff, 0xb0);
    let accent = Rgba::rgb(0x62, 0xd4, 0xff);
    ThemeColors {
        surface_main: Rgba::rgb(0x0a, 0x0a, 0x0a),     // lifted off pure black
        surface_card: Rgba::rgb(0x14, 0x14, 0x14),
        surface_elevated: Rgba::rgb(0x1f, 0x1f, 0x1f),
        surface_hover: Rgba::rgba(255, 255, 255, 0x10),
        bg_hover: Rgba::rgb(0x2b, 0x2b, 0x2b),

        text_primary: Rgba::rgb(0xff, 0xff, 0xff),
        text_secondary: Rgba::rgb(0xf0, 0xf0, 0xf0), // near-primary, NOT gray
        text_muted: Rgba::rgb(0xe0, 0xe0, 0xe0),     // near-primary, NOT gray
        text_disabled: Rgba::rgb(0x8c, 0x8c, 0x8c),  // the only reserved gray

        accent,                                      // reciprocal cyan
        accent_hover: Rgba::rgb(0x8c, 0xe3, 0xff),
        accent_pressed: Rgba::rgb(0xae, 0xed, 0xff),
        accent_text: Rgba::rgb(0x00, 0x00, 0x00),    // reads on cyan fill

        danger,
        danger_bg: Rgba::rgb(0x2a, 0x00, 0x00),      // opaque; always bordered
        danger_border: danger,                       // = danger hue
        danger_hover: Rgba::rgb(0xff, 0xb3, 0xb3),

        warning,
        warning_bg: Rgba::rgb(0x2a, 0x1d, 0x00),     // opaque; always bordered
        warning_border: warning,                     // = warning hue
        warning_hover: Rgba::rgb(0xff, 0xc9, 0x4d),

        success,
        success_bg: Rgba::rgb(0x00, 0x26, 0x1a),     // opaque; always bordered
        success_border: success,                     // = success hue
        success_hover: Rgba::rgb(0x8a, 0xff, 0xc8),

        border_subtle: Rgba::rgb(0x7a, 0x7a, 0x7a), // still clearly visible (4.61:1)
        border_muted: Rgba::rgba(255, 255, 255, 0x38),
        border_strong: Rgba::rgb(0xff, 0xff, 0xff), // = text color

        focus_ring: Rgba::rgb(0xff, 0xee, 0x32),    // bright yellow's correct home

        favorite: danger,
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(false), // dark theme -> white-based overlays
    }
}
