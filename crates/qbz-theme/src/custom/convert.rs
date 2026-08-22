//! Hex <-> color plumbing shared by [`super::derive`].

use crate::auto::PaletteColor;
use crate::color::Rgba;

/// Legacy card shadow (`rgba(0,0,0,0.4)`), identical to the registry/generator
/// constant so a custom theme drops the same shadow as every other theme.
pub(super) const CARD_SHADOW: Rgba = Rgba::rgba(0, 0, 0, 0x66);

/// Parse an opaque `#rrggbb` token, dropping any alpha, falling back to `fallback`
/// on malformed input (a hand-edited JSON can never crash the theme pipeline).
pub(super) fn parse(hex: &str, fallback: Rgba) -> Rgba {
    Rgba::from_hex(hex)
        .map(|c| Rgba::rgb(c.r, c.g, c.b))
        .unwrap_or(fallback)
}

pub(super) fn to_pal(c: Rgba) -> PaletteColor {
    PaletteColor::new(c.r, c.g, c.b)
}

pub(super) fn from_pal(c: PaletteColor) -> Rgba {
    Rgba::rgb(c.r, c.g, c.b)
}
