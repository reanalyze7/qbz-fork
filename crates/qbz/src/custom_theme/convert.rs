//! Color-space conversions between the registry `Rgba` and Slint `Color`.

use qbz_theme::Rgba;
use slint::Color;

/// Convert a registry `Rgba` to a Slint `Color` (straight alpha). Local mirror of
/// `crate::theme::to_color` (which is private to that module).
pub(super) fn to_color(c: Rgba) -> Color {
    Color::from_argb_u8(c.a, c.r, c.g, c.b)
}

/// Convert a Slint `Color` back to a registry `Rgba`.
pub(super) fn rgba_of(c: Color) -> Rgba {
    Rgba::rgba(c.red(), c.green(), c.blue(), c.alpha())
}

/// Parse an opaque `#rrggbb` base token into a Slint `Color`, falling back to
/// transparent-safe black on malformed input (the derivation applies the real
/// fallbacks; this only feeds the editor swatch preview).
pub(super) fn hex_to_color(hex: &str) -> Color {
    to_color(Rgba::from_hex(hex).unwrap_or(Rgba::rgb(0, 0, 0)))
}
