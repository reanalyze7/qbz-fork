//! Core themes (continued): standard `light`/`sepia` plus `tokyo_night`.

use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

use super::std_spec::StdSpec;
use super::{with_alpha, LEGACY_BORDER_MUTED, LEGACY_BORDER_SUBTLE, LEGACY_CARD_SHADOW, LEGACY_SURFACE_HOVER};

/// `light` — core Light theme. OMITS the accent trio (inherits the Dark blue
/// `#4285F4` family from `:root`); alpha base flips to black. (doc 01 §light)
pub(super) fn light() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xff, 0xff, 0xff),
        bg_secondary: Rgba::rgb(0xf5, 0xf5, 0xf5),
        bg_tertiary: Rgba::rgb(0xe8, 0xe8, 0xe8),
        bg_hover: Rgba::rgb(0xf0, 0xf0, 0xf0),
        text_primary: Rgba::rgb(0x0f, 0x0f, 0x0f),
        text_secondary: Rgba::rgb(0x44, 0x44, 0x44),
        text_muted: Rgba::rgb(0x66, 0x66, 0x66),
        text_disabled: Rgba::rgb(0x99, 0x99, 0x99),
        // accent trio inherited from :root Dark:
        accent: Rgba::rgb(0x42, 0x85, 0xf4),
        accent_hover: Rgba::rgb(0x5a, 0x9b, 0xf4),
        accent_pressed: Rgba::rgb(0x32, 0x75, 0xe4),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff), // --btn-primary-text
        // light defines its own danger/warning hues (darker):
        danger: Rgba::rgb(0xdc, 0x26, 0x26),
        warning: Rgba::rgb(0xd9, 0x77, 0x06),
        // light uses 0.1/0.3/0.15 in app.css (hover is 0.15 not 0.2). Keep faithful.
        tint_hover: 0.15,
        border_subtle: Rgba::rgb(0xe0, 0xe0, 0xe0),
        border_strong: Rgba::rgb(0xcc, 0xcc, 0xcc),
        ..Default::default()
    };
    s.build(true)
}

/// Warm sepia/yellow paper tone — the "eye comfort" / night-light-style theme
/// (owner-requested), akin to e-reader sepia mode: a cream-yellow background
/// and warm dark-brown ink instead of pure black-on-white, to cut down on
/// blue light and harsh contrast during long sessions.
pub(super) fn sepia() -> ThemeColors {
    let s = StdSpec {
        bg_primary: Rgba::rgb(0xf4, 0xec, 0xd8),
        bg_secondary: Rgba::rgb(0xed, 0xe0, 0xc8),
        bg_tertiary: Rgba::rgb(0xe6, 0xd4, 0xb8),
        bg_hover: Rgba::rgb(0xdd, 0xc7, 0xa3),
        text_primary: Rgba::rgb(0x3a, 0x2f, 0x1f),
        text_secondary: Rgba::rgb(0x5c, 0x4d, 0x38),
        text_muted: Rgba::rgb(0x8a, 0x78, 0x60),
        text_disabled: Rgba::rgb(0xb5, 0xa5, 0x8c),
        accent: Rgba::rgb(0xb5, 0x65, 0x1d),
        accent_hover: Rgba::rgb(0xc9, 0x75, 0x2b),
        accent_pressed: Rgba::rgb(0x9a, 0x54, 0x14),
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),
        danger: Rgba::rgb(0xb3, 0x41, 0x3a),
        warning: Rgba::rgb(0xb8, 0x86, 0x1f),
        border_subtle: Rgba::rgb(0xe6, 0xd4, 0xb8),
        border_strong: Rgba::rgb(0xdd, 0xc7, 0xa3),
        ..Default::default()
    };
    s.build(true)
}

/// Tokyo Night — full recolor. Surfaces/text/accent transcribed from the legacy
/// Slint ternary (which matches `src/app.css [data-theme="tokyo-night"]`).
pub(super) fn tokyo_night() -> ThemeColors {
    let danger = Rgba::rgb(0xdb, 0x4b, 0x4b); // --danger
    let warning = Rgba::rgb(0xe0, 0xaf, 0x68); // --warning
    let success = Rgba::rgb(0x3f, 0xae, 0x6a);
    ThemeColors {
        surface_main: Rgba::rgb(0x1a, 0x1b, 0x26),     // --bg-primary
        surface_card: Rgba::rgb(0x16, 0x16, 0x1e),     // --bg-secondary
        surface_elevated: Rgba::rgb(0x1c, 0x1d, 0x29), // --bg-tertiary
        surface_hover: LEGACY_SURFACE_HOVER,
        bg_hover: Rgba::rgb(0x20, 0x23, 0x30), // --bg-hover

        text_primary: Rgba::rgb(0xa9, 0xb1, 0xd6),   // --text-primary
        text_secondary: Rgba::rgb(0x78, 0x7c, 0x99), // --text-secondary
        text_muted: Rgba::rgb(0x54, 0x5c, 0x7e),     // --text-muted
        text_disabled: Rgba::rgb(0x3d, 0x42, 0x5e),  // --text-disabled

        accent: Rgba::rgb(0x7a, 0xa2, 0xf7),         // --accent-primary
        accent_hover: Rgba::rgb(0x7d, 0xcf, 0xff),   // --accent-hover
        accent_pressed: Rgba::rgb(0xbb, 0x9a, 0xf7), // --accent-active
        accent_text: Rgba::rgb(0x1a, 0x1b, 0x26),    // --btn-primary-text

        danger,
        danger_bg: with_alpha(danger, 0.1),
        danger_border: with_alpha(danger, 0.3),
        danger_hover: with_alpha(danger, 0.2),

        warning,
        warning_bg: with_alpha(warning, 0.1),
        warning_border: with_alpha(warning, 0.3),
        warning_hover: with_alpha(warning, 0.2),

        success,
        success_bg: with_alpha(success, 0.1),
        success_border: with_alpha(success, 0.3),
        success_hover: with_alpha(success, 0.2),

        border_subtle: LEGACY_BORDER_SUBTLE,
        border_muted: LEGACY_BORDER_MUTED,
        border_strong: Rgba::rgb(0x20, 0x23, 0x30), // --border-strong

        focus_ring: Rgba::rgb(0x7a, 0xa2, 0xf7), // = accent

        favorite: danger,
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(false), // dark theme -> white-based overlays
    }
}
