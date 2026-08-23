use image::RgbaImage;
use slint::Color;

use crate::immersive::color_math::{hsl_to_rgb, rgb_to_hsl};

/// Immersive lyrics-focus accent: a color DERIVED from the album art but
/// chosen to CONTRAST with the atmosphere background (which is itself built
/// from the same cover). Using the cover's dominant hue (e.g. `spectrum-primary`)
/// fails on monochromatic covers — the text lands the same tone as its own
/// background and disappears. So we take the cover's coverage-dominant hue and
/// rotate it to its COMPLEMENT (+180°), forced vivid + mid-bright so the single
/// focus line reads against the warm/dark atmosphere (owner-reported: a warm
/// cover gave salmon-on-orange). A near-grey (B&W) cover has no usable hue, so
/// it falls back to a fixed high-contrast teal.
///
/// NOTE: this hue-histogram binning duplicates the one in
/// `spectrum::spectrum_colors` — intentionally left as-is (a
/// behavior-preserving split, not a dedup pass).
///
/// Takes the pre-downscaled 16x16 sample from [`super::super::cover_tiny_samples`].
pub fn lyrics_accent_color(tiny: &RgbaImage) -> Color {
    let default = Color::from_rgb_u8(0x3f, 0xd9, 0xc8); // bright teal

    // Coverage-weighted dominant hue (same binning as spectrum_colors: one vote
    // per chromatic pixel, so the perceived dominant tone wins over a speck).
    const BINS: usize = 24;
    let mut hist = [0.0f32; BINS];
    let mut chromatic = 0u32;
    for px in tiny.pixels() {
        let (h, s, l) = rgb_to_hsl(px[0], px[1], px[2]);
        if !(0.10..=0.93).contains(&l) || s < 0.08 {
            continue;
        }
        let bin = ((h / 360.0 * BINS as f32) as usize).min(BINS - 1);
        hist[bin] += 1.0;
        chromatic += 1;
    }
    // Effectively B&W cover -> no honest album hue to complement.
    if chromatic < 4 {
        return default;
    }
    let score_at = |i: usize| hist[i] + 0.5 * (hist[(i + BINS - 1) % BINS] + hist[(i + 1) % BINS]);
    let mut best_i = 0usize;
    let mut best = -1.0f32;
    for i in 0..BINS {
        let sc = score_at(i);
        if sc > best {
            best = sc;
            best_i = i;
        }
    }
    let dominant_hue = (best_i as f32 + 0.5) * (360.0 / BINS as f32);
    // Complement = guaranteed contrast against the dominant tone the atmosphere
    // is built from. Vivid + mid-bright so it reads on the dark/warm field.
    let accent_hue = (dominant_hue + 180.0).rem_euclid(360.0);
    let (r, g, b) = hsl_to_rgb(accent_hue, 0.85, 0.62);
    Color::from_rgb_u8(r, g, b)
}
