//! WCAG 2.x relative luminance + contrast ratio.

use super::Rgba;

/// sRGB 0..=255 channel -> linear-light 0.0..=1.0 (WCAG 2.x transfer function).
fn srgb_to_linear(c: u8) -> f64 {
    let cs = c as f64 / 255.0;
    if cs <= 0.040_45 {
        cs / 12.92
    } else {
        ((cs + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.x relative luminance of an OPAQUE color (alpha ignored).
/// `Y = 0.2126 R + 0.7152 G + 0.0722 B` on linear channels.
pub fn relative_luminance(c: Rgba) -> f64 {
    0.2126 * srgb_to_linear(c.r) + 0.7152 * srgb_to_linear(c.g) + 0.0722 * srgb_to_linear(c.b)
}

/// WCAG 2.x contrast ratio between two opaque colors, in `[1.0, 21.0]`.
/// `(L_lighter + 0.05) / (L_darker + 0.05)`. Order-independent.
pub fn contrast_ratio(a: Rgba, b: Rgba) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}
