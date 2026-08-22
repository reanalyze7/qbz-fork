//! Cinnamon and XFCE wallpaper probing (minor DEs, no accent-color support).

use std::path::PathBuf;
use std::process::Command;

use super::parse::parse_gsettings_uri;

pub(super) fn get_cinnamon_wallpaper() -> Result<String, String> {
    let output = Command::new("gsettings")
        .args(["get", "org.cinnamon.desktop.background", "picture-uri"])
        .output()
        .map_err(|e| format!("Failed to run gsettings: {}", e))?;

    if !output.status.success() {
        return Err("Could not get Cinnamon wallpaper via gsettings".into());
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if let Some(path) = parse_gsettings_uri(&raw) {
        if PathBuf::from(&path).exists() {
            return Ok(path);
        }
    }

    Err("Could not determine Cinnamon wallpaper".into())
}

pub(super) fn get_xfce_wallpaper() -> Result<String, String> {
    let output = Command::new("xfconf-query")
        .args([
            "-c",
            "xfce4-desktop",
            "-p",
            "/backdrop/screen0/monitoreDP-1/workspace0/last-image",
        ])
        .output()
        .map_err(|e| format!("Failed to run xfconf-query: {}", e))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if PathBuf::from(&path).exists() {
            return Ok(path);
        }
    }

    let output = Command::new("xfconf-query")
        .args(["-c", "xfce4-desktop", "-l", "-v"])
        .output()
        .map_err(|e| format!("Failed to list xfce4-desktop properties: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("last-image") {
                let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
                if parts.len() == 2 {
                    let path = parts[1].trim();
                    if PathBuf::from(path).exists() {
                        return Ok(path.to_string());
                    }
                }
            }
        }
    }

    Err("Could not determine XFCE wallpaper".into())
}
