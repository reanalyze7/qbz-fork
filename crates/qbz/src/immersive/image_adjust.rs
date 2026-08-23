//! Pixel-buffer adjustment passes used only by `atmosphere.rs`'s pipeline.

use image::{Rgba, RgbaImage};

pub(super) fn color_adjust(mut img: RgbaImage) -> RgbaImage {
    let mut min_r = 255.0f32;
    let mut min_g = 255.0f32;
    let mut min_b = 255.0f32;
    let mut max_r = 0.0f32;
    let mut max_g = 0.0f32;
    let mut max_b = 0.0f32;
    let mut total_brightness = 0.0f32;
    let count = (img.width() * img.height()).max(1) as f32;

    for px in img.pixels_mut() {
        let (r, g, b) = saturate_brightness_contrast(px[0], px[1], px[2]);
        px[0] = r;
        px[1] = g;
        px[2] = b;

        let rf = r as f32;
        let gf = g as f32;
        let bf = b as f32;
        min_r = min_r.min(rf);
        min_g = min_g.min(gf);
        min_b = min_b.min(bf);
        max_r = max_r.max(rf);
        max_g = max_g.max(gf);
        max_b = max_b.max(bf);
        total_brightness += (rf + gf + bf) / 3.0;
    }

    let norm_strength = (total_brightness / count / 80.0).min(1.0);
    let target_min = 18.0f32;
    let target_range = 232.0f32;
    let range_r = (max_r - min_r).max(1.0);
    let range_g = (max_g - min_g).max(1.0);
    let range_b = (max_b - min_b).max(1.0);

    for px in img.pixels_mut() {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let norm_r = target_min + ((r - min_r) / range_r) * target_range;
        let norm_g = target_min + ((g - min_g) / range_g) * target_range;
        let norm_b = target_min + ((b - min_b) / range_b) * target_range;
        let lift_r = (r * 1.5).min(255.0);
        let lift_g = (g * 1.5).min(255.0);
        let lift_b = (b * 1.5).min(255.0);

        px[0] = (lift_r + (norm_r - lift_r) * norm_strength).mul_add(1.08, 0.0).min(255.0) as u8;
        px[1] = (lift_g + (norm_g - lift_g) * norm_strength).min(255.0) as u8;
        px[2] = (lift_b + (norm_b - lift_b) * norm_strength).min(255.0) as u8;
    }

    img
}

fn saturate_brightness_contrast(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let mut rf = r as f32;
    let mut gf = g as f32;
    let mut bf = b as f32;
    let gray = (rf + gf + bf) / 3.0;
    let sat = 2.45;
    rf = gray + (rf - gray) * sat;
    gf = gray + (gf - gray) * sat;
    bf = gray + (bf - gray) * sat;

    let brightness = 0.92;
    let contrast = 1.18;
    let adjust = |v: f32| ((v * brightness - 128.0) * contrast + 128.0).clamp(0.0, 255.0) as u8;
    (adjust(rf), adjust(gf), adjust(bf))
}

pub(super) fn vignette(mut img: RgbaImage, intensity: f32) -> RgbaImage {
    let w = img.width() as f32;
    let h = img.height() as f32;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let inner = w.min(h) * 0.20;
    let outer = w.min(h) * 0.70;

    for (x, y, px) in img.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let d = (dx * dx + dy * dy).sqrt();
        let t = ((d - inner) / (outer - inner)).clamp(0.0, 1.0);
        let factor = 1.0 - intensity * t;
        *px = Rgba([
            (px[0] as f32 * factor) as u8,
            (px[1] as f32 * factor) as u8,
            (px[2] as f32 * factor) as u8,
            px[3],
        ]);
    }
    img
}
