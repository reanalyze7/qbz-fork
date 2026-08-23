//! Shared tiny-downscale step + the blurred "atmosphere" texture pipeline.

use image::{imageops, RgbaImage};

use super::image_adjust::{color_adjust, vignette};

/// One-pass downscale of a decoded cover to the two tiny sampling sizes the
/// cover-derived visuals below consume (8x8: atmosphere + glow; 16x16:
/// spectrum + lyrics accent). Each helper used to redo its own
/// full-size-to-tiny resize plus a full pixel-buffer copy — on the
/// now-playing track-change path that happened four times ON the UI thread.
/// Computing the tinies once (off-thread) preserves each helper's exact
/// sampling input. Takes the pixel Vec by value: no copy on the hot path.
pub fn cover_tiny_samples(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
) -> Option<(RgbaImage, RgbaImage)> {
    let src = RgbaImage::from_raw(width, height, pixels)?;
    let tiny8 = imageops::resize(&src, 8, 8, imageops::FilterType::Triangle);
    let tiny16 = imageops::resize(&src, 16, 16, imageops::FilterType::Triangle);
    Some((tiny8, tiny16))
}

/// Generate a 128x128 atmospheric image from decoded RGBA artwork pixels.
/// Mirrors `src/lib/immersive/utils/texture-loader.ts::generateAtmosphere`.
pub fn generate_atmosphere(pixels: &[u8], width: u32, height: u32) -> Option<(Vec<u8>, u32, u32)> {
    let src = RgbaImage::from_raw(width, height, pixels.to_vec())?;
    let tiny = imageops::resize(&src, 8, 8, imageops::FilterType::Triangle);
    Some(atmosphere_from_tiny8(&tiny))
}

/// Atmosphere pipeline over a pre-downscaled 8x8 sample (`cover_tiny_samples`).
/// Same stages as [`generate_atmosphere`] past its own 8x8 resize.
pub fn atmosphere_from_tiny8(tiny8: &RgbaImage) -> (Vec<u8>, u32, u32) {
    let scaled = imageops::resize(tiny8, 128, 128, imageops::FilterType::CatmullRom);
    let blurred = imageops::blur(&scaled, 16.0);
    let adjusted = color_adjust(blurred);
    let final_img = vignette(adjusted, 0.20);
    (final_img.into_raw(), 128, 128)
}

/// Static Slint image for album/artist headers that reuse Immersive's blurred
/// artwork atmosphere without mounting the animated full-screen scene.
pub fn generate_atmosphere_image(pixels: &[u8], width: u32, height: u32) -> Option<slint::Image> {
    let (bg_pixels, bg_w, bg_h) = generate_atmosphere(pixels, width, height)?;
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(bg_w, bg_h);
    let dst = buffer.make_mut_bytes();
    if dst.len() != bg_pixels.len() {
        return None;
    }
    dst.copy_from_slice(&bg_pixels);
    Some(slint::Image::from_rgba8(buffer))
}
