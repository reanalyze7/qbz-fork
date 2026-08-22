//! Shared thresholds + color-math helpers for the a11y contrast test suite.

use crate::color::Rgba;

pub(crate) const AAA_BODY: f64 = 7.0; // WCAG 2.x AAA normal text
pub(crate) const AA_NORMAL: f64 = 4.5; // WCAG 2.x AA normal text
pub(crate) const NON_TEXT: f64 = 3.0; // WCAG 2.x SC 1.4.11 / 1.4.3-large

/// Solid composite of `fg` over `bg` (a11y status surfaces are opaque, but
/// translucent overlays compose straight-alpha for contrast measurement).
#[allow(dead_code)]
pub(crate) fn over(fg: Rgba, bg: Rgba) -> Rgba {
    if fg.a == 255 {
        return fg;
    }
    let a = fg.a as f64 / 255.0;
    let mix = |f: u8, b: u8| ((f as f64 * a) + (b as f64 * (1.0 - a))).round() as u8;
    Rgba::rgb(mix(fg.r, bg.r), mix(fg.g, bg.g), mix(fg.b, bg.b))
}

/// Crude protanopia/deuteranopia simulation (Brettel-style fixed matrices,
/// sufficient to confirm the documented hue separation survives red-green
/// CVD). Returns the simulated sRGB. Used only to assert ΔE separation, not
/// for rendering.
pub(crate) fn simulate_deutan(c: Rgba) -> Rgba {
    // Linearize, apply the standard deuteranopia LMS-collapse matrix
    // (Machado 2009, severity 1.0), re-encode. Approximate but stable.
    let lin = |v: u8| {
        let cs = v as f64 / 255.0;
        if cs <= 0.04045 {
            cs / 12.92
        } else {
            ((cs + 0.055) / 1.055).powf(2.4)
        }
    };
    let enc = |v: f64| {
        let v = v.clamp(0.0, 1.0);
        let s = if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round().clamp(0.0, 255.0) as u8
    };
    let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
    // Machado deuteranomaly severity=1.0 matrix:
    let nr = 0.367_322 * r + 0.860_646 * g + -0.227_968 * b;
    let ng = 0.280_085 * r + 0.672_501 * g + 0.047_413 * b;
    let nb = -0.011_820 * r + 0.042_940 * g + 0.968_881 * b;
    Rgba::rgb(enc(nr), enc(ng), enc(nb))
}

/// CIE76 ΔE in Lab (sufficient threshold check for hue separation).
pub(crate) fn delta_e(a: Rgba, b: Rgba) -> f64 {
    fn to_lab(c: Rgba) -> (f64, f64, f64) {
        let lin = |v: u8| {
            let cs = v as f64 / 255.0;
            if cs <= 0.04045 {
                cs / 12.92
            } else {
                ((cs + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
        // linear sRGB -> XYZ (D65)
        let x = 0.4124 * r + 0.3576 * g + 0.1805 * b;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let z = 0.0193 * r + 0.1192 * g + 0.9505 * b;
        let f = |t: f64| {
            if t > 0.008_856 {
                t.cbrt()
            } else {
                7.787 * t + 16.0 / 116.0
            }
        };
        let (xn, yn, zn) = (0.95047, 1.0, 1.08883);
        let (fx, fy, fz) = (f(x / xn), f(y / yn), f(z / zn));
        (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
    }
    let (l1, a1, b1) = to_lab(a);
    let (l2, a2, b2) = to_lab(b);
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}
