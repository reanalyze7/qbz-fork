use image::RgbaImage;
use slint::Color;

use crate::immersive::color_math::{hsl_to_rgb, rgb_to_hsl};

/// Two vivid bar colors for the Spectrum visualizer, derived from the artwork's
/// PERCEIVED dominant tone. We bin chromatic pixels by hue and pick the most
/// ABUNDANT hue (one vote per pixel = coverage), NOT the most saturated — so a
/// metallic/dark cover resolves to the steel-blue you actually see, instead of
/// an amplified speck of an unseen magenta highlight. The picked hue is forced
/// vivid + mid-bright so it reads on the black bg; the secondary stop rotates
/// +55° for a clear gradient. A cover with essentially no chromatic pixels (a
/// true B&W cover) falls back to a default duotone.
///
/// NOTE: this hue-histogram binning duplicates the one in
/// `lyrics_accent::lyrics_accent_color` — intentionally left as-is (a
/// behavior-preserving split), see that function's doc comment.
///
/// Takes the pre-downscaled 16x16 sample from [`super::super::cover_tiny_samples`].
/// Returns (primary at the base, secondary at the tip).
pub fn spectrum_colors(tiny: &RgbaImage) -> (Color, Color) {
    let default = (
        Color::from_rgb_u8(0, 220, 200),
        Color::from_rgb_u8(150, 50, 255),
    );

    // Hue histogram over CHROMATIC pixels, weighted by COVERAGE (one vote per
    // pixel). 24 bins of 15°. Near-grey / near-black / near-white pixels carry
    // no usable hue and are skipped, so the grey mass never votes.
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

    // Too few tinted pixels (effectively a B&W cover): no perceived tone.
    if chromatic < 4 {
        return default;
    }

    // Smoothed cluster score for a bin (peak + circular neighbours).
    let score_at = |i: usize| hist[i] + 0.5 * (hist[(i + BINS - 1) % BINS] + hist[(i + 1) % BINS]);

    // PRIMARY = most abundant hue cluster.
    let mut best_i = 0usize;
    let mut best = -1.0f32;
    for i in 0..BINS {
        let sc = score_at(i);
        if sc > best {
            best = sc;
            best_i = i;
        }
    }
    let primary_hue = (best_i as f32 + 0.5) * (360.0 / BINS as f32);

    // SECONDARY = a SECOND genuine hue cluster, at least ~45° away from the
    // primary and carrying real mass (>= 35% of the peak). If the cover is
    // essentially one colour (e.g. the mono pink/magenta Caifanes cover) there
    // is NO honest second hue — derive the secondary from the SAME hue, just
    // deeper + more saturated, instead of fabricating a hue-rotated colour the
    // album doesn't contain (the old `+55°` turned a pink cover into pink→orange).
    let mut sec_i: Option<usize> = None;
    let mut sec_best = 0.0f32;
    for i in 0..BINS {
        let circ = (i as i32 - best_i as i32).rem_euclid(BINS as i32);
        let dist = circ.min(BINS as i32 - circ); // circular distance in bins
        if dist < 3 {
            continue; // keep >= ~45° away from the primary
        }
        let sc = score_at(i);
        if sc > sec_best {
            sec_best = sc;
            sec_i = Some(i);
        }
    }

    let primary = hsl_to_rgb(primary_hue, 0.85, 0.58);
    let secondary = match sec_i.filter(|_| sec_best >= best * 0.35) {
        // Two genuinely different album colours → gradient between them.
        Some(si) => {
            let sec_hue = (si as f32 + 0.5) * (360.0 / BINS as f32);
            hsl_to_rgb(sec_hue, 0.88, 0.62)
        }
        // Single-colour cover → same hue, deeper (stays on-album).
        None => hsl_to_rgb(primary_hue, 0.95, 0.40),
    };
    (
        Color::from_rgb_u8(primary.0, primary.1, primary.2),
        Color::from_rgb_u8(secondary.0, secondary.1, secondary.2),
    )
}
