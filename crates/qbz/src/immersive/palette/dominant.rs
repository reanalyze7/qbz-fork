use image::{imageops, RgbaImage};
use slint::Color;

/// Dominant swatch of a cover, for the single-cover playlist card letterbox.
///
/// Mirrors Tauri's `extractPalette().dominant`: the card uses `image-fit:
/// contain`, so the gaps around a non-square cover are filled with the cover's
/// most-abundant colour instead of theme grey. We downscale to 16x16, bin the
/// pixels by quantised colour (one vote per pixel = coverage, NOT saturation),
/// and average the winning bin's actual pixels back to an opaque swatch — so a
/// dark/metallic cover resolves to the muted tone you actually see, not an
/// amplified speck of an unseen highlight. Transparent pixels are skipped; an
/// effectively-empty cover falls back to a neutral dark surface.
pub fn dominant_cover_color(pixels: &[u8], width: u32, height: u32) -> Color {
    // Neutral dark fallback (close to Tauri's `--bg-tertiary`), used when the
    // buffer is unusable or carries no opaque pixels.
    let fallback = Color::from_rgb_u8(30, 30, 34);
    let Some(src) = RgbaImage::from_raw(width, height, pixels.to_vec()) else {
        return fallback;
    };
    let tiny = imageops::resize(&src, 16, 16, imageops::FilterType::Triangle);

    // Quantise each opaque pixel into a coarse 4x4x4 RGB bucket (shift 6 bits ->
    // 64 buckets) and tally coverage + accumulate the exact channel sums so the
    // winner can be averaged back to its true tone.
    const BUCKETS: usize = 64; // 4 * 4 * 4
    let mut count = [0u32; BUCKETS];
    let mut sum_r = [0u32; BUCKETS];
    let mut sum_g = [0u32; BUCKETS];
    let mut sum_b = [0u32; BUCKETS];
    let mut opaque = 0u32;

    for px in tiny.pixels() {
        if px[3] < 16 {
            continue; // skip (near-)transparent pixels
        }
        let bucket =
            ((px[0] >> 6) as usize) * 16 + ((px[1] >> 6) as usize) * 4 + ((px[2] >> 6) as usize);
        count[bucket] += 1;
        sum_r[bucket] += px[0] as u32;
        sum_g[bucket] += px[1] as u32;
        sum_b[bucket] += px[2] as u32;
        opaque += 1;
    }

    if opaque == 0 {
        return fallback;
    }

    // Most-abundant bucket = the perceived dominant colour.
    let mut best = 0usize;
    for i in 1..BUCKETS {
        if count[i] > count[best] {
            best = i;
        }
    }
    let n = count[best].max(1);
    Color::from_rgb_u8(
        (sum_r[best] / n) as u8,
        (sum_g[best] / n) as u8,
        (sum_b[best] / n) as u8,
    )
}
