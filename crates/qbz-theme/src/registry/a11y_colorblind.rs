//! `colorblind` — universal Okabe-Ito accessibility theme (Part B §B.4).

use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

use super::LEGACY_CARD_SHADOW;

/// `colorblind` — universal Okabe-Ito (Part B §B.4). danger (reddish-purple)
/// split from warning (amber) across CVD confusion axes; foregrounds lightened
/// to clear AA on all tiers. `success` kept `#009e73`; body text routes to
/// `success-hover #33c397` on the lightest tier (C.6).
pub(super) fn colorblind() -> ThemeColors {
    let danger = Rgba::rgb(0xd4, 0x88, 0xb1);
    let warning = Rgba::rgb(0xe6, 0x9f, 0x00);
    let success = Rgba::rgb(0x00, 0x9e, 0x73); // Okabe-Ito bluish green
    let accent = Rgba::rgb(0x62, 0xa5, 0xe4);
    ThemeColors {
        surface_main: Rgba::rgb(0x1a, 0x1a, 0x2e),     // retained dark navy
        surface_card: Rgba::rgb(0x22, 0x22, 0x3a),
        surface_elevated: Rgba::rgb(0x2c, 0x2c, 0x46),
        surface_hover: Rgba::rgba(255, 255, 255, 0x10),
        bg_hover: Rgba::rgb(0x36, 0x36, 0x52),

        text_primary: Rgba::rgb(0xff, 0xff, 0xff),
        text_secondary: Rgba::rgb(0xdc, 0xdc, 0xe0),
        text_muted: Rgba::rgb(0xaa, 0xaa, 0xb8),    // lightened -> AA on bg-tertiary
        text_disabled: Rgba::rgb(0x6f, 0x6f, 0x86), // exempt, perceptible

        accent,                                     // lightened Okabe-Ito blue
        accent_hover: Rgba::rgb(0x7e, 0xb4, 0xe8),
        accent_pressed: Rgba::rgb(0x8b, 0xbc, 0xec),
        accent_text: Rgba::rgb(0x0a, 0x0a, 0x14),   // near-black on light-blue

        danger,                                     // lightened reddish-purple
        danger_bg: Rgba::rgb(0x3a, 0x1a, 0x2e),     // solid tint
        danger_border: Rgba::rgb(0x8a, 0x4a, 0x70), // solid tint
        danger_hover: Rgba::rgb(0xe0, 0xa3, 0xc5),

        warning,                                    // Okabe-Ito amber
        warning_bg: Rgba::rgb(0x3a, 0x2e, 0x00),    // solid tint
        warning_border: Rgba::rgb(0x8a, 0x6e, 0x00),
        warning_hover: Rgba::rgb(0xf0, 0xb6, 0x30),

        success,                                    // AA on primary/secondary
        success_bg: Rgba::rgb(0x0a, 0x33, 0x29),    // solid tint
        success_border: Rgba::rgb(0x1f, 0x6b, 0x54),
        success_hover: Rgba::rgb(0x33, 0xc3, 0x97), // body-size success on bg-tertiary

        border_subtle: Rgba::rgb(0x3e, 0x3e, 0x56), // decorative (1.65:1)
        border_muted: Rgba::rgba(255, 255, 255, 0x38),
        border_strong: Rgba::rgb(0x6e, 0x6e, 0x88), // control boundary (3.45:1)

        focus_ring: Rgba::rgb(0x8b, 0xbc, 0xec),    // high-tone blue (8.53:1)

        favorite: danger,
        card_shadow: LEGACY_CARD_SHADOW,

        alpha: alpha_ramp(false), // dark theme -> white-based overlays
    }
}
