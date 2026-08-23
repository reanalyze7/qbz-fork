//! Decode + in-memory LRU cache — "pure computation over bytes".

mod lru;

use std::sync::Arc;

pub use lru::decoded_pixels;
pub(in crate::artwork) use lru::store_decoded;

/// Build a Slint image from decoded RGBA8 pixels. Returns an empty image
/// if the buffer length does not match the dimensions.
pub fn pixels_to_image(pixels: &[u8], width: u32, height: u32) -> slint::Image {
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let dst = buffer.make_mut_bytes();
    if dst.len() != pixels.len() {
        return slint::Image::default();
    }
    dst.copy_from_slice(pixels);
    slint::Image::from_rgba8(buffer)
}

/// Decode raw image bytes to RGBA8, downscaled to `decode_size`.
pub(super) fn decode_rgba(bytes: &[u8], decode_size: u32) -> Option<(Vec<u8>, u32, u32)> {
    let rgba = image::load_from_memory(bytes)
        .ok()?
        .thumbnail(decode_size, decode_size)
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    Some((rgba.into_raw(), width, height))
}

/// A decoded cover, ready for `pixels_to_image`. `slint::Image` is `!Send`, so
/// the cache stores the RGBA tuple (which IS `Send`) and the event loop builds
/// the `slint::Image` on demand. `Arc`-wrapped so cache hits clone a pointer,
/// not the ~36KB buffer.
pub type DecodedPixels = (Arc<Vec<u8>>, u32, u32);

/// Decode a local cover file to `decode_size` RGBA pixels, through the
/// decoded-pixel cache (keyed by path). Synchronous and `Send`-safe (no
/// `slint::Image`), so worker threads can pre-decode row covers and the
/// event loop builds the image via [`pixels_to_image`].
pub fn decode_local_pixels(path: &str, decode_size: u32) -> Option<DecodedPixels> {
    if path.is_empty() {
        return None;
    }
    if let Some(hit) = decoded_pixels(path, decode_size) {
        return Some(hit);
    }
    let bytes = std::fs::read(path).ok()?;
    let (pixels, w, h) = decode_rgba(&bytes, decode_size)?;
    let entry: DecodedPixels = (Arc::new(pixels), w, h);
    store_decoded(path, decode_size, &entry);
    Some(entry)
}

/// Bounded replacement for `slint::Image::load_from_path` on synchronous
/// UI-thread call sites: decodes to `decode_size` so only thumbnail-sized
/// pixels are retained for the model row's lifetime (`load_from_path` keeps
/// the full-resolution source in the image buffer). `None` on empty path /
/// missing file / decode failure — callers keep their fallback semantics.
pub fn load_local_cover(path: &str, decode_size: u32) -> Option<slint::Image> {
    decode_local_pixels(path, decode_size).map(|(pixels, w, h)| pixels_to_image(&pixels, w, h))
}

/// Representative color of decoded RGBA pixels for the header gradient.
///
/// A plain average desaturates badly (everything trends grey), so the
/// average is saturation-boosted off its own mean and then normalized to
/// a fixed peak brightness — the result keeps the cover's hue and reads
/// as a clear tinted band against the dark surface. Dark fallback for
/// empty input.
pub fn header_tint(pixels: &[u8]) -> (u8, u8, u8) {
    let (mut r, mut g, mut b, mut n) = (0f64, 0f64, 0f64, 0u64);
    for px in pixels.chunks_exact(4) {
        if px[3] < 16 {
            continue;
        }
        r += px[0] as f64;
        g += px[1] as f64;
        b += px[2] as f64;
        n += 1;
    }
    if n == 0 {
        return (34, 34, 42);
    }
    let nf = n as f64;
    let (mut r, mut g, mut b) = (r / nf, g / nf, b / nf);

    // Saturation boost: push each channel away from the average's mean.
    let mean = (r + g + b) / 3.0;
    let boost = 2.1;
    let saturate = |c: f64| (mean + (c - mean) * boost).clamp(0.0, 255.0);
    r = saturate(r);
    g = saturate(g);
    b = saturate(b);

    // Normalize the brightest channel to a fixed peak so the tint is
    // always clearly visible — bright enough to perceive, dark enough to
    // keep white text readable. Caps the scale so a near-black cover is
    // only modestly lifted.
    let peak = r.max(g).max(b).max(1.0);
    let scale = (138.0 / peak).min(1.7);
    (
        (r * scale) as u8,
        (g * scale) as u8,
        (b * scale) as u8,
    )
}
