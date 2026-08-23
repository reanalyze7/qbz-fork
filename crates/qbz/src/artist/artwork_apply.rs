use slint::ComponentHandle;

use crate::{AppWindow, ArtistState};

/// Apply decoded portrait artwork. Runs on the Slint event loop.
pub fn apply_artwork(window: &AppWindow, pixels: &[u8], width: u32, height: u32) {
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let dst = buffer.make_mut_bytes();
    if dst.len() != pixels.len() {
        return;
    }
    dst.copy_from_slice(pixels);
    let (r, g, b) = crate::artwork::header_tint(pixels);
    let state = window.global::<ArtistState>();
    state.set_artwork(slint::Image::from_rgba8(buffer));
    state.set_header_color(slint::Color::from_rgb_u8(r, g, b));
    if let Some(atmosphere) = crate::immersive::generate_atmosphere_image(pixels, width, height) {
        state.set_header_atmosphere(atmosphere);
    } else {
        state.set_header_atmosphere(slint::Image::default());
    }
}
