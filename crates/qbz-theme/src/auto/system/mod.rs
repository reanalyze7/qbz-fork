//! Desktop-environment detection, wallpaper path retrieval, and system accent /
//! color-scheme reading. 1:1 logic port of the Tauri `auto_theme::system`
//! module (gsettings / kdeglobals / COSMIC / xfconf probing via `Command`).

mod cosmic;
mod gnome;
mod kde;
mod kde_scheme;
mod parse;
mod xfce_cinnamon;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::env;

use super::{PaletteColor, SystemColorScheme};

/// Supported desktop environments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesktopEnvironment {
    Gnome,
    KdePlasma,
    Cosmic,
    Xfce,
    Cinnamon,
    Unknown(String),
}

impl DesktopEnvironment {
    /// Human-readable name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Gnome => "GNOME",
            Self::KdePlasma => "KDE Plasma",
            Self::Cosmic => "COSMIC",
            Self::Xfce => "Xfce",
            Self::Cinnamon => "Cinnamon",
            Self::Unknown(name) => name.as_str(),
        }
    }
}

/// Detect the current desktop environment.
pub fn detect_desktop_environment() -> DesktopEnvironment {
    let candidates = [
        env::var("XDG_CURRENT_DESKTOP"),
        env::var("XDG_SESSION_DESKTOP"),
        env::var("DESKTOP_SESSION"),
    ];

    for candidate in &candidates {
        if let Ok(val) = candidate {
            let upper = val.to_uppercase();
            if upper.contains("GNOME") || upper.contains("UNITY") || upper.contains("UBUNTU") {
                return DesktopEnvironment::Gnome;
            }
            if upper.contains("KDE") || upper.contains("PLASMA") {
                return DesktopEnvironment::KdePlasma;
            }
            if upper.contains("COSMIC") {
                return DesktopEnvironment::Cosmic;
            }
            if upper.contains("XFCE") {
                return DesktopEnvironment::Xfce;
            }
            if upper.contains("CINNAMON") || upper.contains("X-CINNAMON") {
                return DesktopEnvironment::Cinnamon;
            }
        }
    }

    let name = candidates
        .iter()
        .find_map(|c| c.as_ref().ok().cloned())
        .unwrap_or_else(|| "unknown".to_string());

    DesktopEnvironment::Unknown(name)
}

/// Get the current wallpaper path for the detected DE.
pub fn get_system_wallpaper() -> Result<String, String> {
    let de = detect_desktop_environment();
    get_wallpaper_for_de(&de)
}

fn get_wallpaper_for_de(de: &DesktopEnvironment) -> Result<String, String> {
    match de {
        DesktopEnvironment::Gnome => gnome::get_gnome_wallpaper(),
        DesktopEnvironment::KdePlasma => kde::get_kde_wallpaper(),
        DesktopEnvironment::Cosmic => cosmic::get_cosmic_wallpaper(),
        DesktopEnvironment::Cinnamon => xfce_cinnamon::get_cinnamon_wallpaper(),
        DesktopEnvironment::Xfce => xfce_cinnamon::get_xfce_wallpaper(),
        DesktopEnvironment::Unknown(name) => {
            Err(format!("Unsupported desktop environment: {}", name))
        }
    }
}

/// Get the system accent color for the detected DE.
pub fn get_system_accent_color() -> Result<PaletteColor, String> {
    let de = detect_desktop_environment();
    get_accent_for_de(&de)
}

fn get_accent_for_de(de: &DesktopEnvironment) -> Result<PaletteColor, String> {
    match de {
        DesktopEnvironment::Gnome => gnome::get_gnome_accent(),
        DesktopEnvironment::KdePlasma => kde::get_kde_accent(),
        DesktopEnvironment::Cosmic => cosmic::get_cosmic_accent(),
        _ => Err(format!(
            "Accent color not supported for {}",
            de.display_name()
        )),
    }
}

/// Read the full system color scheme from the current DE (KDE / GNOME only).
pub fn get_system_color_scheme() -> Result<SystemColorScheme, String> {
    let de = detect_desktop_environment();
    match de {
        DesktopEnvironment::KdePlasma => kde_scheme::get_kde_color_scheme(),
        DesktopEnvironment::Gnome => gnome::get_gnome_color_scheme(),
        _ => Err(format!(
            "Full color scheme not supported for {}",
            de.display_name()
        )),
    }
}
