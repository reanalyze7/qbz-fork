//! `PaletteColor`: a plain RGB color plus the color-math used throughout
//! palette extraction and generation (luminance, saturation, contrast, HSL).

use serde::{Deserialize, Serialize};

/// A single RGB color used throughout palette extraction and generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PaletteColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PaletteColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Relative luminance (ITU-R BT.709) in [0.0, 1.0].
    pub fn luminance(&self) -> f64 {
        fn linearize(c: u8) -> f64 {
            let s = c as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linearize(self.r) + 0.7152 * linearize(self.g) + 0.0722 * linearize(self.b)
    }

    /// HSL saturation in [0.0, 1.0].
    pub fn saturation(&self) -> f64 {
        let (r, g, b) = (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        if delta < 1e-6 {
            return 0.0;
        }
        let l = (max + min) / 2.0;
        if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        }
    }

    /// WCAG contrast ratio against another color (range [1, 21]).
    pub fn contrast_ratio(&self, other: &PaletteColor) -> f64 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Euclidean distance in RGB space.
    pub fn distance(&self, other: &PaletteColor) -> f64 {
        let dr = self.r as f64 - other.r as f64;
        let dg = self.g as f64 - other.g as f64;
        let db = self.b as f64 - other.b as f64;
        (dr * dr + dg * dg + db * db).sqrt()
    }
}

#[cfg(test)]
#[path = "color_tests.rs"]
mod tests;
