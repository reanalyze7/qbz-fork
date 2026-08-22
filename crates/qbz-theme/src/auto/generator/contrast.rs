//! Contrast helpers (ported 1:1 from `auto_theme::generator`).

use super::super::PaletteColor;

/// Pick the best foreground for text on the accent triplet (base, hover, active),
/// considering the worst case across all three so `:hover`/`:active` stay legible.
/// Shared with the custom-theme builder so `accent_text` is derived identically.
pub(crate) fn pick_btn_text_for_accent_set(
    accent: &PaletteColor,
    accent_hover: &PaletteColor,
    accent_active: &PaletteColor,
) -> PaletteColor {
    let white = PaletteColor::new(255, 255, 255);
    let black = PaletteColor::new(0, 0, 0);

    let white_worst = white
        .contrast_ratio(accent)
        .min(white.contrast_ratio(accent_hover))
        .min(white.contrast_ratio(accent_active));
    let black_worst = black
        .contrast_ratio(accent)
        .min(black.contrast_ratio(accent_hover))
        .min(black.contrast_ratio(accent_active));

    if white_worst >= 3.0 {
        white
    } else if black_worst > white_worst {
        black
    } else {
        white
    }
}

/// Ensure text has at least WCAG AA contrast (4.5:1) against the background.
pub(super) fn ensure_text_contrast(
    text: PaletteColor,
    bg: &PaletteColor,
    is_dark: bool,
) -> PaletteColor {
    ensure_text_contrast_target(text, bg, is_dark, 4.5)
}

/// Ensure text meets `target` contrast against `bg`, shifting lightness toward
/// white (dark) / black (light) up to 20 steps, then clamping to pure white/black.
/// Shared with the custom-theme builder for its derived muted/secondary tiers.
pub(crate) fn ensure_text_contrast_target(
    text: PaletteColor,
    bg: &PaletteColor,
    is_dark: bool,
    target: f64,
) -> PaletteColor {
    if text.contrast_ratio(bg) >= target {
        return text;
    }

    let (h, s, l) = text.to_hsl();
    let direction = if is_dark { 0.05 } else { -0.05 };
    let mut new_l = l;

    for _ in 0..20 {
        new_l = (new_l + direction).clamp(0.0, 1.0);
        let candidate = PaletteColor::from_hsl(h, s, new_l);
        if candidate.contrast_ratio(bg) >= target {
            return candidate;
        }
    }

    if is_dark {
        PaletteColor::new(255, 255, 255)
    } else {
        PaletteColor::new(0, 0, 0)
    }
}
