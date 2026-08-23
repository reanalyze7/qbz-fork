use image::RgbaImage;
use slint::Color;

/// AlbumReactive glow color: the most saturated non-extreme 8x8 sample.
/// Takes the pre-downscaled 8x8 sample from [`super::super::cover_tiny_samples`].
pub fn glow_color(tiny: &RgbaImage) -> Color {
    let mut best_sat = 0.0f32;
    let mut best = (100u8, 100u8, 255u8);

    for px in tiny.pixels() {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let lum = (max + min) / 2.0;
        let sat = if (max - min).abs() < f32::EPSILON {
            0.0
        } else if lum > 127.0 {
            (max - min) / (510.0 - max - min).max(1.0)
        } else {
            (max - min) / (max + min).max(1.0)
        };
        if lum > 50.0 && lum < 220.0 && sat > best_sat {
            best_sat = sat;
            best = (px[0], px[1], px[2]);
        }
    }

    Color::from_argb_u8(0x59, best.0, best.1, best.2)
}
