use super::super::{PaletteColor, ThemePalette};
use super::assemble::{assemble, opaque};
use super::contrast::{ensure_text_contrast, ensure_text_contrast_target, pick_btn_text_for_accent_set};
use crate::colors::ThemeColors;

/// Build a [`ThemeColors`] from a k-means–extracted image/wallpaper palette.
///
/// Port of `auto_theme::generator::generate_theme`.
pub fn theme_from_palette(palette: &ThemePalette) -> ThemeColors {
    let is_dark = palette.is_dark;

    // Text tiers by polarity, then contrast-enforced against bg_primary. Disabled
    // stays intentionally low-contrast (a visual cue), so it is not adjusted.
    let (text_primary, text_secondary, text_muted, text_disabled) = if is_dark {
        (
            PaletteColor::new(255, 255, 255),
            PaletteColor::new(204, 204, 204),
            PaletteColor::new(136, 136, 136),
            PaletteColor::new(85, 85, 85),
        )
    } else {
        (
            PaletteColor::new(15, 15, 15),
            PaletteColor::new(68, 68, 68),
            PaletteColor::new(102, 102, 102),
            PaletteColor::new(153, 153, 153),
        )
    };
    let text_primary = ensure_text_contrast(text_primary, &palette.bg_primary, is_dark);
    let text_secondary =
        ensure_text_contrast_target(text_secondary, &palette.bg_primary, is_dark, 4.5);
    let text_muted = ensure_text_contrast_target(text_muted, &palette.bg_primary, is_dark, 3.0);

    // Accent triplet — hover +10% L, active -10% L. Text picked across the whole
    // triplet so hover/active stay legible.
    let accent = palette.accent;
    let accent_hover = accent.shift_lightness(0.10);
    let accent_active = accent.shift_lightness(-0.10);
    let accent_text = pick_btn_text_for_accent_set(&accent, &accent_hover, &accent_active);

    // Status hues by polarity (identical to generate_theme).
    let (danger, warning) = if is_dark {
        (PaletteColor::new(239, 68, 68), PaletteColor::new(251, 191, 36))
    } else {
        (PaletteColor::new(220, 38, 38), PaletteColor::new(217, 119, 6))
    };
    let status_hover = if is_dark { 0.2 } else { 0.15 };

    // Borders: subtle/strong lightness shifts from bg_primary.
    let border_subtle = if is_dark {
        palette.bg_primary.shift_lightness(0.08)
    } else {
        palette.bg_primary.shift_lightness(-0.08)
    };
    let border_strong = if is_dark {
        palette.bg_primary.shift_lightness(0.14)
    } else {
        palette.bg_primary.shift_lightness(-0.14)
    };

    assemble(
        is_dark,
        opaque(palette.bg_primary),
        opaque(palette.bg_secondary),
        opaque(palette.bg_tertiary),
        opaque(palette.bg_hover),
        opaque(text_primary),
        opaque(text_secondary),
        opaque(text_muted),
        opaque(text_disabled),
        opaque(accent),
        opaque(accent_hover),
        opaque(accent_active),
        opaque(accent_text),
        opaque(danger),
        opaque(warning),
        status_hover,
        opaque(border_subtle),
        opaque(border_strong),
    )
}
