//! Shared pure parsing helpers used across the GNOME/KDE/COSMIC/Cinnamon
//! probes: gsettings/file URI parsing, RGB CSV, and image-path detection.

use std::path::PathBuf;

use super::super::PaletteColor;

/// Parse gsettings output like `'file:///path/to/wallpaper.jpg'` into a path.
pub(super) fn parse_gsettings_uri(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches('\'').trim_matches('"');
    parse_file_uri(trimmed).or_else(|| {
        if PathBuf::from(trimmed).is_absolute() {
            Some(trimmed.to_string())
        } else {
            None
        }
    })
}

/// Extract filesystem path from a `file:///path` URI.
pub(super) fn parse_file_uri(uri: &str) -> Option<String> {
    uri.strip_prefix("file://").map(|path| path.replace("%20", " "))
}

/// Parse "r,g,b" CSV format (KDE).
pub(super) fn parse_rgb_csv(value: &str) -> Result<PaletteColor, String> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() < 3 {
        return Err(format!("Invalid RGB CSV: {}", value));
    }
    let r = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid R value: {}", parts[0]))?;
    let g = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid G value: {}", parts[1]))?;
    let b = parts[2]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid B value: {}", parts[2]))?;
    Ok(PaletteColor::new(r, g, b))
}

pub(super) fn is_image_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
        || lower.ends_with(".tiff")
        || lower.ends_with(".tif")
}
