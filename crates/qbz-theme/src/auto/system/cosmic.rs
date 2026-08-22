//! COSMIC wallpaper + accent color probing (config-file parsing, no CLI tool).

use std::env;
use std::fs;
use std::path::PathBuf;

use super::super::PaletteColor;
use super::parse::{is_image_path, parse_file_uri};

pub(super) fn get_cosmic_wallpaper() -> Result<String, String> {
    let home = env::var("HOME").map_err(|_| "HOME not set".to_string())?;

    let config_paths = [
        format!(
            "{}/.config/cosmic/com.system76.CosmicBackground/v1/all",
            home
        ),
        format!(
            "{}/.config/cosmic/com.system76.CosmicBackground/v1/backgrounds",
            home
        ),
    ];

    for config_path in &config_paths {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Some(path) = extract_path_from_cosmic_config(&content) {
                if PathBuf::from(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    Err("Could not find wallpaper in COSMIC config".into())
}

pub(super) fn get_cosmic_accent() -> Result<PaletteColor, String> {
    let home = env::var("HOME").map_err(|_| "HOME not set".to_string())?;

    let accent_paths = [
        format!(
            "{}/.config/cosmic/com.system76.CosmicTheme.Dark/v1/accent",
            home
        ),
        format!(
            "{}/.config/cosmic/com.system76.CosmicTheme.Light/v1/accent",
            home
        ),
    ];

    for path in &accent_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some(color) = parse_cosmic_color(&content) {
                return Ok(color);
            }
        }
    }

    Err("Could not read COSMIC accent color".into())
}

/// Best-effort extraction of an image path from COSMIC config content.
fn extract_path_from_cosmic_config(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim().trim_matches('"').trim_matches('\'');

        if let Some(path) = parse_file_uri(trimmed) {
            if is_image_path(&path) {
                return Some(path);
            }
        }

        if trimmed.starts_with('/') && is_image_path(trimmed) {
            return Some(trimmed.to_string());
        }

        if let Some(start) = trimmed.find('/') {
            let potential = &trimmed[start..];
            let end = potential
                .find(|c: char| c == '"' || c == '\'' || c == ')' || c == ',')
                .unwrap_or(potential.len());
            let path = &potential[..end];
            if is_image_path(path) && PathBuf::from(path).is_absolute() {
                return Some(path.to_string());
            }
        }
    }
    None
}

/// Parse a COSMIC color (RON-like, RGBA floats or ints).
pub(super) fn parse_cosmic_color(content: &str) -> Option<PaletteColor> {
    let nums: Vec<f64> = content
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    if nums.len() >= 3 {
        let (r, g, b) = if nums[0] <= 1.0 && nums[1] <= 1.0 && nums[2] <= 1.0 {
            (
                (nums[0] * 255.0).round() as u8,
                (nums[1] * 255.0).round() as u8,
                (nums[2] * 255.0).round() as u8,
            )
        } else {
            (nums[0] as u8, nums[1] as u8, nums[2] as u8)
        };
        Some(PaletteColor::new(r, g, b))
    } else {
        None
    }
}
