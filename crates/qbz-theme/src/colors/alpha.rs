//! The 24-tier alpha ramp: percentages, byte conversion, and index lookup.

use crate::color::Rgba;

/// The 24 alpha tiers, in ascending percentage order. This is the SUPERSET of
/// the two Tauri alpha scales (cosmetic 20-tier + a11y 22-tier), per the
/// migration plan (A.3). Index into [`super::ThemeColors::alpha`] with these.
pub const ALPHA_PERCENTS: [u8; 24] = [
    4, 5, 6, 8, 10, 12, 15, 18, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95,
];

/// Number of alpha tiers (= `ALPHA_PERCENTS.len()`).
pub const ALPHA_COUNT: usize = 24;

/// Map an alpha percentage to its `0xAA` byte (rounded). `pct * 255 / 100`.
pub const fn alpha_byte(pct: u8) -> u8 {
    ((pct as u16 * 255 + 50) / 100) as u8
}

/// Position of an alpha percentage within [`ALPHA_PERCENTS`], or `None`.
pub fn alpha_index(pct: u8) -> Option<usize> {
    let mut i = 0;
    while i < ALPHA_COUNT {
        if ALPHA_PERCENTS[i] == pct {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Build the 24-tier alpha ramp for a theme of the given polarity.
/// `is_light` themes get a BLACK base (dark hairlines/hovers read on light
/// surfaces); dark themes get a WHITE base — matching Tauri's per-theme flip.
pub fn alpha_ramp(is_light: bool) -> [Rgba; ALPHA_COUNT] {
    let (r, g, b) = if is_light { (0, 0, 0) } else { (255, 255, 255) };
    let mut out = [Rgba::rgba(r, g, b, 0); ALPHA_COUNT];
    let mut i = 0;
    while i < ALPHA_COUNT {
        out[i] = Rgba::rgba(r, g, b, alpha_byte(ALPHA_PERCENTS[i]));
        i += 1;
    }
    out
}
