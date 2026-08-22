//! HSL conversion + lightness-shift helpers for [`super::PaletteColor`], split
//! out of `color.rs` for the 130-line budget.

use super::PaletteColor;

impl PaletteColor {
    /// Shift lightness by `amount` (-1.0 to 1.0) in HSL space. Returns a new color.
    pub fn shift_lightness(&self, amount: f64) -> PaletteColor {
        let (h, s, l) = self.to_hsl();
        let new_l = (l + amount).clamp(0.0, 1.0);
        PaletteColor::from_hsl(h, s, new_l)
    }

    /// Convert to HSL (h in [0, 360), s and l in [0, 1]).
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let (r, g, b) = (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        let delta = max - min;

        if delta < 1e-6 {
            return (0.0, 0.0, l);
        }

        let s = if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if (max - r).abs() < 1e-6 {
            ((g - b) / delta) % 6.0
        } else if (max - g).abs() < 1e-6 {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };
        let h = (h * 60.0 + 360.0) % 360.0;

        (h, s, l)
    }

    /// Construct from HSL values.
    pub fn from_hsl(h: f64, s: f64, l: f64) -> PaletteColor {
        if s < 1e-6 {
            let v = (l * 255.0).round() as u8;
            return PaletteColor::new(v, v, v);
        }

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match h_prime as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        PaletteColor::new(
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        )
    }
}
