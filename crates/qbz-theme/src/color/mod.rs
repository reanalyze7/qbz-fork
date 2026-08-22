//! Plain RGBA color + hand-rolled contrast math (WCAG 2.x + APCA approximation).
//!
//! No external color crate: the registry must compile and unit-test fast on its
//! own (ADR-006), so the few formulas we need live here.

mod apca;
mod wcag;
#[cfg(test)]
mod tests;

pub use apca::apca_lc;
pub use wcag::{contrast_ratio, relative_luminance};

/// 8-bit-per-channel color with straight (non-premultiplied) alpha.
///
/// `a == 255` is fully opaque, `a == 0` fully transparent. The Slint side maps
/// this 1:1 to `slint::Color::from_argb_u8(a, r, g, b)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    /// Opaque color from 8-bit channels.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Color with explicit alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse `#rrggbb` / `rrggbb` / `#rrggbbaa` (case-insensitive). Returns
    /// `None` on any malformed input. `const`-incompatible (loops), used only
    /// in tests + as a convenience.
    pub fn from_hex(s: &str) -> Option<Self> {
        let h = s.strip_prefix('#').unwrap_or(s);
        let bytes = h.as_bytes();
        let hx = |hi: u8, lo: u8| -> Option<u8> {
            let v = |c: u8| -> Option<u8> {
                match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                }
            };
            Some(v(hi)? * 16 + v(lo)?)
        };
        match bytes.len() {
            6 => Some(Self::rgb(
                hx(bytes[0], bytes[1])?,
                hx(bytes[2], bytes[3])?,
                hx(bytes[4], bytes[5])?,
            )),
            8 => Some(Self::rgba(
                hx(bytes[0], bytes[1])?,
                hx(bytes[2], bytes[3])?,
                hx(bytes[4], bytes[5])?,
                hx(bytes[6], bytes[7])?,
            )),
            _ => None,
        }
    }

    /// Format as `#rrggbb` (lowercase, alpha dropped). The custom-theme base
    /// tokens are all opaque, so the alpha channel is intentionally not
    /// serialized — the picker HEX field and the on-disk `custom_theme.json`
    /// both round-trip through this 6-digit form.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
