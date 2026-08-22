//! GNOME wallpaper, accent color, and full color-scheme probing (gsettings).

use std::path::PathBuf;
use std::process::Command;

use super::super::PaletteColor;
use super::parse::parse_gsettings_uri;
use super::super::SystemColorScheme;

pub(super) fn get_gnome_wallpaper() -> Result<String, String> {
    for key in &["picture-uri-dark", "picture-uri"] {
        let output = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.background", key])
            .output()
            .map_err(|e| format!("Failed to run gsettings: {}", e))?;

        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(path) = parse_gsettings_uri(&raw) {
                if PathBuf::from(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }
    Err("Could not determine GNOME wallpaper".into())
}

pub(super) fn get_gnome_accent() -> Result<PaletteColor, String> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "accent-color"])
        .output()
        .map_err(|e| format!("Failed to run gsettings: {}", e))?;

    if !output.status.success() {
        return Err("gsettings accent-color not available (requires GNOME 47+)".into());
    }

    let raw = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_matches('\'')
        .to_lowercase();

    let color = match raw.as_str() {
        "blue" => PaletteColor::new(53, 132, 228),
        "teal" => PaletteColor::new(38, 162, 105),
        "green" => PaletteColor::new(51, 209, 122),
        "yellow" => PaletteColor::new(246, 211, 45),
        "orange" => PaletteColor::new(255, 120, 0),
        "red" => PaletteColor::new(224, 27, 36),
        "pink" => PaletteColor::new(220, 138, 221),
        "purple" => PaletteColor::new(145, 65, 172),
        "slate" => PaletteColor::new(111, 131, 150),
        _ => return Err(format!("Unknown GNOME accent color: {}", raw)),
    };

    Ok(color)
}

pub(super) fn get_gnome_color_scheme() -> Result<SystemColorScheme, String> {
    // GNOME exposes little via dconf: detect dark/light + accent, fill the rest
    // with Adwaita defaults.
    let accent = get_gnome_accent().ok();

    let is_dark = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let val = String::from_utf8_lossy(&o.stdout).trim().to_lowercase();
                Some(val.contains("dark"))
            } else {
                None
            }
        })
        .unwrap_or(true);

    let (bg, bg_alt, view_bg, btn_bg, fg, fg_inactive) = if is_dark {
        (
            PaletteColor::new(36, 36, 36),
            PaletteColor::new(48, 48, 48),
            PaletteColor::new(30, 30, 30),
            PaletteColor::new(60, 60, 60),
            PaletteColor::new(255, 255, 255),
            PaletteColor::new(140, 140, 140),
        )
    } else {
        (
            PaletteColor::new(246, 245, 244),
            PaletteColor::new(235, 235, 235),
            PaletteColor::new(255, 255, 255),
            PaletteColor::new(225, 225, 225),
            PaletteColor::new(36, 36, 36),
            PaletteColor::new(120, 120, 120),
        )
    };

    Ok(SystemColorScheme {
        window_bg: Some(bg),
        window_bg_alt: Some(bg_alt),
        view_bg: Some(view_bg),
        button_bg: Some(btn_bg),
        header_bg: None,
        header_bg_inactive: None,
        tooltip_bg: None,
        window_fg: Some(fg),
        window_fg_inactive: Some(fg_inactive),
        view_fg: Some(fg),
        button_fg: Some(PaletteColor::new(255, 255, 255)),
        selection_bg: accent,
        selection_fg: Some(PaletteColor::new(255, 255, 255)),
        selection_hover: None,
        accent,
        fg_link: None,
        fg_negative: Some(PaletteColor::new(224, 27, 36)),
        fg_neutral: Some(PaletteColor::new(205, 147, 9)),
        fg_positive: Some(PaletteColor::new(38, 162, 105)),
        wm_active_bg: None,
        wm_active_fg: None,
        wm_inactive_bg: None,
    })
}
