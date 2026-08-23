//! Embedded tray icon assets + PNG -> ksni pixmap decoding.

use image::GenericImageView;
use ksni::Icon;

use super::dark_mode::prefer_dark_tray;

// Multiple pixmap sizes per StatusNotifierItem spec — panels pick the best
// match for their bar height (22 = base, 32/44/64 = HiDPI).
//
// Legacy filename note (shared with the Tauri assets): `tray-light-*` holds
// the BLACK glyph (for LIGHT panels) and `tray-dark-*` holds the WHITE glyph.
// The constants use glyph-colour names so the mapping is explicit.
const TRAY_ICON_MONO_BLACK_22: &[u8] = include_bytes!("../../../icons/tray-light-22.png");
const TRAY_ICON_MONO_BLACK_32: &[u8] = include_bytes!("../../../icons/tray-light-32.png");
const TRAY_ICON_MONO_BLACK_44: &[u8] = include_bytes!("../../../icons/tray-light-44.png");
const TRAY_ICON_MONO_BLACK_64: &[u8] = include_bytes!("../../../icons/tray-light-64.png");
const TRAY_ICON_MONO_WHITE_22: &[u8] = include_bytes!("../../../icons/tray-dark-22.png");
const TRAY_ICON_MONO_WHITE_32: &[u8] = include_bytes!("../../../icons/tray-dark-32.png");
const TRAY_ICON_MONO_WHITE_44: &[u8] = include_bytes!("../../../icons/tray-dark-44.png");
const TRAY_ICON_MONO_WHITE_64: &[u8] = include_bytes!("../../../icons/tray-dark-64.png");
const TRAY_ICON_COLOR_22: &[u8] = include_bytes!("../../../icons/tray-color-22.png");
const TRAY_ICON_COLOR_32: &[u8] = include_bytes!("../../../icons/tray-color-32.png");
const TRAY_ICON_COLOR_44: &[u8] = include_bytes!("../../../icons/tray-color-44.png");
const TRAY_ICON_COLOR_64: &[u8] = include_bytes!("../../../icons/tray-color-64.png");

/// Convert an embedded RGBA PNG to the ARGB32 big-endian layout ksni expects.
fn decode_pixmap(bytes: &[u8]) -> Result<Icon, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("decode tray icon: {e}"))?;
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    // ksni spec: ARGB32 with A, R, G, B order per pixel. `image` gives us
    // RGBA; rotate_right(1) moves the alpha byte from the last slot to the
    // first.
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    Ok(Icon {
        width: width as i32,
        height: height as i32,
        data,
    })
}

#[derive(Clone, Copy, Debug)]
enum IconVariant {
    /// Black glyph — for light panels.
    MonoBlack,
    /// White glyph — for dark panels (Plasma dark, GNOME top bar).
    MonoWhite,
    /// Full color vinyl.
    Color,
}

/// Resolve which icon variant to load. `theme_override`:
///   - "auto" (or unrecognised) — system color-scheme detection
///   - "mono-light" — white (light-coloured) glyph
///   - "mono-dark"  — black (dark-coloured) glyph
///   - "color"      — full vinyl logo
fn resolve_variant(theme_override: Option<&str>) -> IconVariant {
    match theme_override {
        Some("mono-light") => IconVariant::MonoWhite,
        Some("mono-dark") => IconVariant::MonoBlack,
        Some("color") => IconVariant::Color,
        _ => {
            if prefer_dark_tray() {
                IconVariant::MonoWhite
            } else {
                IconVariant::MonoBlack
            }
        }
    }
}

/// Decode pixmaps (22/32/44/64) for the resolved variant.
pub(super) fn decode_tray_icons(theme_override: Option<&str>) -> Result<Vec<Icon>, String> {
    let sources: [&[u8]; 4] = match resolve_variant(theme_override) {
        IconVariant::MonoBlack => [
            TRAY_ICON_MONO_BLACK_22,
            TRAY_ICON_MONO_BLACK_32,
            TRAY_ICON_MONO_BLACK_44,
            TRAY_ICON_MONO_BLACK_64,
        ],
        IconVariant::MonoWhite => [
            TRAY_ICON_MONO_WHITE_22,
            TRAY_ICON_MONO_WHITE_32,
            TRAY_ICON_MONO_WHITE_44,
            TRAY_ICON_MONO_WHITE_64,
        ],
        IconVariant::Color => [
            TRAY_ICON_COLOR_22,
            TRAY_ICON_COLOR_32,
            TRAY_ICON_COLOR_44,
            TRAY_ICON_COLOR_64,
        ],
    };
    sources.iter().map(|b| decode_pixmap(b)).collect()
}
