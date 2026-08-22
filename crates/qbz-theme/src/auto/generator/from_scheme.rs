use super::super::{PaletteColor, SystemColorScheme};
use super::assemble::{assemble, opaque};
use super::contrast::{ensure_text_contrast, ensure_text_contrast_target, pick_btn_text_for_accent_set};
use crate::colors::ThemeColors;

/// Build a [`ThemeColors`] directly from a DE color scheme (KDE/GNOME).
///
/// Port of `auto_theme::generator::generate_theme_from_scheme`.
pub fn theme_from_scheme(scheme: &SystemColorScheme) -> ThemeColors {
    let window_bg = scheme.window_bg.unwrap_or(PaletteColor::new(40, 40, 40));
    let is_dark = window_bg.luminance() < 0.5;

    // Surfaces
    let bg_secondary = scheme.view_bg.unwrap_or_else(|| {
        if is_dark {
            window_bg.shift_lightness(0.03)
        } else {
            window_bg.shift_lightness(-0.03)
        }
    });
    let bg_tertiary = scheme.button_bg.unwrap_or_else(|| {
        if is_dark {
            window_bg.shift_lightness(0.10)
        } else {
            window_bg.shift_lightness(-0.10)
        }
    });
    let bg_hover = scheme.window_bg_alt.unwrap_or_else(|| {
        PaletteColor::new(
            ((window_bg.r as u16 + bg_secondary.r as u16) / 2) as u8,
            ((window_bg.g as u16 + bg_secondary.g as u16) / 2) as u8,
            ((window_bg.b as u16 + bg_secondary.b as u16) / 2) as u8,
        )
    });

    // Text
    let text_primary = scheme.window_fg.unwrap_or(if is_dark {
        PaletteColor::new(223, 223, 223)
    } else {
        PaletteColor::new(36, 36, 36)
    });
    let text_primary = ensure_text_contrast(text_primary, &window_bg, is_dark);

    let text_secondary_raw = scheme
        .view_fg
        .unwrap_or_else(|| text_primary.shift_lightness(if is_dark { -0.10 } else { 0.10 }));
    let text_secondary = ensure_text_contrast_target(text_secondary_raw, &window_bg, is_dark, 4.5);

    let text_muted_raw = scheme
        .window_fg_inactive
        .unwrap_or_else(|| text_primary.shift_lightness(if is_dark { -0.25 } else { 0.25 }));
    let text_muted = ensure_text_contrast_target(text_muted_raw, &window_bg, is_dark, 3.0);

    let text_disabled = text_muted.shift_lightness(if is_dark { -0.10 } else { 0.10 });

    // Accent triplet (selection)
    let accent = scheme
        .accent
        .or(scheme.selection_bg)
        .unwrap_or(PaletteColor::new(0, 120, 215));
    let accent_hover = scheme
        .selection_hover
        .unwrap_or_else(|| accent.shift_lightness(0.10));
    let accent_active = accent.shift_lightness(-0.10);
    // Trust DE selection_fg if present, else compute across the triplet.
    let accent_text = scheme
        .selection_fg
        .unwrap_or_else(|| pick_btn_text_for_accent_set(&accent, &accent_hover, &accent_active));

    // Status hues from system negative/neutral, else polarity fallbacks.
    let danger = scheme.fg_negative.unwrap_or(if is_dark {
        PaletteColor::new(239, 68, 68)
    } else {
        PaletteColor::new(220, 38, 38)
    });
    let warning = scheme.fg_neutral.unwrap_or(if is_dark {
        PaletteColor::new(251, 191, 36)
    } else {
        PaletteColor::new(217, 119, 6)
    });
    let status_hover = if is_dark { 0.2 } else { 0.15 };

    // Borders
    let border_subtle = if is_dark {
        window_bg.shift_lightness(0.06)
    } else {
        window_bg.shift_lightness(-0.06)
    };
    let border_strong = if is_dark {
        window_bg.shift_lightness(0.12)
    } else {
        window_bg.shift_lightness(-0.12)
    };

    assemble(
        is_dark,
        opaque(window_bg),
        opaque(bg_secondary),
        opaque(bg_tertiary),
        opaque(bg_hover),
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
