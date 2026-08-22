//! Core themes: the P1 originals `dark` and `oled`. `tokyo_night`/`light`/
//! `sepia` live in `core_themes_2.rs` to stay under the 130-line budget.

use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

use super::{with_alpha, LEGACY_BORDER_MUTED, LEGACY_BORDER_SUBTLE, LEGACY_CARD_SHADOW, LEGACY_SURFACE_HOVER};

/// `:root` Dark — the base every other theme inherits omissions from.
/// All hex values cite `src/app.css :root` via the inventory doc.
pub(super) fn dark() -> ThemeColors {
    let danger = Rgba::rgb(0xef, 0x44, 0x44); // --danger
    let warning = Rgba::rgb(0xfb, 0xbf, 0x24); // --warning
    let success = Rgba::rgb(0x3f, 0xae, 0x6a); // NEW (project green)
    ThemeColors {
        surface_main: Rgba::rgb(0x0f, 0x0f, 0x0f),     // --bg-primary
        surface_card: Rgba::rgb(0x1a, 0x1a, 0x1a),     // --bg-secondary
        surface_elevated: Rgba::rgb(0x2a, 0x2a, 0x2a), // --bg-tertiary
        surface_hover: LEGACY_SURFACE_HOVER,
        bg_hover: Rgba::rgb(0x1f, 0x1f, 0x1f), // --bg-hover

        text_primary: Rgba::rgb(0xff, 0xff, 0xff),   // --text-primary
        text_secondary: Rgba::rgb(0xcc, 0xcc, 0xcc), // --text-secondary
        text_muted: Rgba::rgb(0x88, 0x88, 0x88),     // --text-muted
        text_disabled: Rgba::rgb(0x55, 0x55, 0x55),  // --text-disabled

        accent: Rgba::rgb(0x42, 0x85, 0xf4),         // --accent-primary
        accent_hover: Rgba::rgb(0x5a, 0x9b, 0xf4),   // --accent-hover
        accent_pressed: Rgba::rgb(0x32, 0x75, 0xe4), // --accent-active
        accent_text: Rgba::rgb(0xff, 0xff, 0xff),    // --btn-primary-text

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
        border_strong: Rgba::rgb(0x3a, 0x3a, 0x3a), // --border-strong

        focus_ring: Rgba::rgb(0x42, 0x85, 0xf4), // = accent (no Tauri token; new)

        favorite: danger, // the loved-heart uses danger red
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(false), // dark theme -> white-based overlays
    }
}

/// OLED Black — inherits everything from Dark except backgrounds + borders.
/// The legacy Slint OLED only overrode the three surfaces; keep that exactly,
/// inherit the rest from `dark()`.
pub(super) fn oled() -> ThemeColors {
    ThemeColors {
        surface_main: Rgba::rgb(0x00, 0x00, 0x00),     // --bg-primary
        surface_card: Rgba::rgb(0x0a, 0x0a, 0x0a),     // --bg-secondary
        surface_elevated: Rgba::rgb(0x1a, 0x1a, 0x1a), // --bg-tertiary
        bg_hover: Rgba::rgb(0x11, 0x11, 0x11),         // --bg-hover (oled)
        border_strong: Rgba::rgb(0x2a, 0x2a, 0x2a),    // --border-strong (oled)
        ..dark()
    }
}

