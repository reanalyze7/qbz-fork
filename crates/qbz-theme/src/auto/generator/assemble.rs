//! Shared assembly: turn already-chosen solid surface/text/accent/status
//! colors into a full [`ThemeColors`], deriving the alpha-tinted status
//! families, translucent edges, and polarity alpha ramp.

use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

/// Legacy card shadow (`rgba(0,0,0,0.4)`), identical to the registry constant so
/// generated themes drop the same shadow as static ones.
const CARD_SHADOW: Rgba = Rgba::rgba(0, 0, 0, 0x66);

/// Convert an opaque palette color to `Rgba`.
pub(super) fn opaque(c: super::super::PaletteColor) -> Rgba {
    Rgba::rgb(c.r, c.g, c.b)
}

/// Straight-alpha overlay of an opaque hue at `frac` opacity (0.0..=1.0).
/// Mirrors `registry::with_alpha` / Tauri's `rgba(hue, frac)` tint. Shared with
/// the custom-theme builder (`crate::custom`) so the status-family tint shape is
/// defined once.
pub(crate) fn tint(c: Rgba, frac: f32) -> Rgba {
    let a = (frac * 255.0 + 0.5) as u8;
    Rgba::rgba(c.r, c.g, c.b, a)
}

/// Assemble the derived families shared by both entry points, given the solid
/// surface/text/accent/status colors already chosen. `is_dark` drives polarity
/// (alpha ramp base, translucent edges, success hue). `status_hover` is the
/// per-polarity hover alpha the Tauri generator uses (0.2 dark / 0.15 light) —
/// applied to danger/warning AND the derived success family so the whole status
/// group shares one hover strength.
#[allow(clippy::too_many_arguments)]
pub(super) fn assemble(
    is_dark: bool,
    surface_main: Rgba,
    surface_card: Rgba,
    surface_elevated: Rgba,
    bg_hover: Rgba,
    text_primary: Rgba,
    text_secondary: Rgba,
    text_muted: Rgba,
    text_disabled: Rgba,
    accent: Rgba,
    accent_hover: Rgba,
    accent_pressed: Rgba,
    accent_text: Rgba,
    danger: Rgba,
    warning: Rgba,
    status_hover: f32,
    border_subtle: Rgba,
    border_strong: Rgba,
) -> ThemeColors {
    let is_light = !is_dark;

    // Polarity-aware translucent edges + hover overlay (legacy Slint-only
    // tokens, no CSS-map parity). White base on dark, black base on light —
    // identical to `StdSpec::build`.
    let (eh, eg, eb) = if is_light { (0, 0, 0) } else { (255, 255, 255) };
    let surface_hover = Rgba::rgba(eh, eg, eb, 0x10); // ~6%
    let border_muted = Rgba::rgba(eh, eg, eb, 0x38); // ~22%

    // success: NEW token (no Tauri parity). Same theme-green the registry uses,
    // darker on light canvases so it clears >=3:1. Tint shape follows the status
    // family (bg 0.1 / border 0.3 / hover = status_hover).
    let success = if is_light {
        Rgba::rgb(0x1f, 0x8a, 0x4c)
    } else {
        Rgba::rgb(0x3f, 0xae, 0x6a)
    };

    ThemeColors {
        surface_main,
        surface_card,
        surface_elevated,
        surface_hover,
        bg_hover,

        text_primary,
        text_secondary,
        text_muted,
        text_disabled,

        accent,
        accent_hover,
        accent_pressed,
        accent_text,

        danger,
        danger_bg: tint(danger, 0.1),
        danger_border: tint(danger, 0.3),
        danger_hover: tint(danger, status_hover),

        warning,
        warning_bg: tint(warning, 0.1),
        warning_border: tint(warning, 0.3),
        warning_hover: tint(warning, status_hover),

        success,
        success_bg: tint(success, 0.1),
        success_border: tint(success, 0.3),
        success_hover: tint(success, status_hover),

        border_subtle,
        border_muted,
        border_strong,

        focus_ring: accent, // = accent (matches registry / P1)
        favorite: danger,   // loved-heart uses danger red (matches registry)
        card_shadow: CARD_SHADOW,

        alpha: alpha_ramp(is_light), // black base on light, white base on dark
    }
}
