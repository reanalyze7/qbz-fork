//! KDE Plasma wallpaper, accent color, and full color-scheme probing
//! (kdeglobals / plasma-org.kde.plasma.desktop-appletsrc parsing).

use std::env;
use std::fs;
use std::path::PathBuf;

use super::super::PaletteColor;
use super::parse::{parse_file_uri, parse_rgb_csv};

pub(super) fn get_kde_wallpaper() -> Result<String, String> {
    let home = env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let config_path = format!("{}/.config/plasma-org.kde.plasma.desktop-appletsrc", home);

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Cannot read Plasma config: {}", e))?;

    let mut in_wallpaper_section = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_wallpaper_section = trimmed.contains("Wallpaper")
                && trimmed.contains("org.kde.image")
                && trimmed.contains("General");
        }

        if in_wallpaper_section && trimmed.starts_with("Image=") {
            let value = trimmed.trim_start_matches("Image=").trim();
            if let Some(path) = parse_file_uri(value) {
                if PathBuf::from(&path).exists() {
                    return Ok(path);
                }
            }
            if PathBuf::from(value).exists() {
                return Ok(value.to_string());
            }
        }
    }

    Err("Could not find wallpaper in Plasma config".into())
}

pub(super) fn get_kde_accent() -> Result<PaletteColor, String> {
    let home = env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let config_path = format!("{}/.config/kdeglobals", home);

    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Cannot read kdeglobals: {}", e))?;

    // 1. Explicit AccentColor in [General] (Plasma 6 custom accent).
    let mut in_general = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[General]" {
            in_general = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_general = false;
            continue;
        }

        if in_general && trimmed.starts_with("AccentColor=") {
            let value = trimmed.trim_start_matches("AccentColor=").trim();
            return parse_rgb_csv(value);
        }
    }

    // 2. Fallback: color-scheme sections (DecorationFocus / Selection background).
    let fallback_sections = [
        ("[Colors:Selection]", "DecorationFocus"),
        ("[Colors:Selection]", "BackgroundNormal"),
        ("[Colors:View]", "DecorationFocus"),
    ];

    for (section, key) in &fallback_sections {
        if let Some(color) = read_kde_color_key(&content, section, key) {
            return Ok(color);
        }
    }

    Err("AccentColor not found in kdeglobals (no explicit accent or color scheme)".into())
}

/// Read a specific key from a KDE config section, parsing "r,g,b" format.
pub(super) fn read_kde_color_key(content: &str, section: &str, key: &str) -> Option<PaletteColor> {
    let mut in_section = false;
    let prefix = format!("{}=", key);

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == section {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_section = false;
            continue;
        }

        if in_section && trimmed.starts_with(&prefix) {
            let value = trimmed[prefix.len()..].trim();
            return parse_rgb_csv(value).ok();
        }
    }

    None
}
