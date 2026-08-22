//! K-means palette extraction from images. 1:1 logic port of the Tauri
//! `auto_theme::palette` module (downsample → k-means → role assignment).

mod kmeans;
mod roles;

use image::GenericImageView;
use std::path::Path;

use super::ThemePalette;
use kmeans::kmeans;
use roles::build_palette;

/// Load an image, downsample, and extract dominant colors via k-means.
pub fn extract_palette(image_path: &str) -> Result<ThemePalette, String> {
    let path = Path::new(image_path);
    if !path.exists() {
        return Err(format!("Image not found: {}", image_path));
    }

    // Bounded decode: a decompression-bomb PNG (65k x 65k) would otherwise
    // expand to tens of GB before the 100x100 downsample. 12k x 12k / 512 MB
    // covers any real wallpaper while keeping the worst case survivable.
    let mut reader = image::ImageReader::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image: {}", e))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let img = reader.decode().map_err(|e| format!("Failed to decode image: {}", e))?;

    // Downsample to 100x100 for fast processing.
    let thumb = img.resize_exact(100, 100, image::imageops::FilterType::Lanczos3);

    let mut pixels: Vec<[f64; 3]> = Vec::with_capacity(10_000);
    for (_x, _y, rgba) in thumb.pixels() {
        // Skip semi-transparent pixels.
        if rgba[3] < 200 {
            continue;
        }
        pixels.push([rgba[0] as f64, rgba[1] as f64, rgba[2] as f64]);
    }

    if pixels.is_empty() {
        return Err("Image contains no opaque pixels".to_string());
    }

    let clusters = kmeans(&pixels, 6, 25);
    if clusters.is_empty() {
        return Err("K-means produced no clusters".to_string());
    }

    build_palette(clusters)
}

/// Extract a palette directly from raw RGB pixel data (testing / custom sources).
pub fn extract_palette_from_pixels(pixels: &[[f64; 3]]) -> Result<ThemePalette, String> {
    if pixels.is_empty() {
        return Err("No pixels provided".to_string());
    }
    let clusters = kmeans(pixels, 6, 25);
    if clusters.is_empty() {
        return Err("K-means produced no clusters".to_string());
    }
    build_palette(clusters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_from_pixels_dark_dominant() {
        let mut pixels = Vec::new();
        for _ in 0..500 {
            pixels.push([20.0, 30.0, 80.0]);
        }
        for _ in 0..300 {
            pixels.push([240.0, 160.0, 40.0]);
        }
        for _ in 0..200 {
            pixels.push([10.0, 10.0, 15.0]);
        }
        let palette = extract_palette_from_pixels(&pixels).unwrap();
        assert!(palette.is_dark);
        assert!(palette.accent.saturation() > 0.1);
    }

    #[test]
    fn monochrome_fallback_accent() {
        let pixels: Vec<[f64; 3]> = (0..1000).map(|_| [50.0, 52.0, 48.0]).collect();
        let palette = extract_palette_from_pixels(&pixels).unwrap();
        assert!(palette.accent.saturation() > 0.1 || palette.accent.luminance() > 0.3);
    }
}
