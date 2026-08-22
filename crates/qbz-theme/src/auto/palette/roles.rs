//! Role assignment: turn k-means clusters into background/accent roles with
//! WCAG AA contrast enforcement.

use super::super::{PaletteColor, ThemePalette};
use super::kmeans::Cluster;

/// Build a [`ThemePalette`] from k-means clusters.
pub(super) fn build_palette(mut clusters: Vec<Cluster>) -> Result<ThemePalette, String> {
    // Sort by pixel count (dominant first).
    clusters.sort_by(|a, b| b.count.cmp(&a.count));

    let all_colors: Vec<PaletteColor> = clusters
        .iter()
        .map(|c| {
            PaletteColor::new(
                c.centroid[0].round() as u8,
                c.centroid[1].round() as u8,
                c.centroid[2].round() as u8,
            )
        })
        .collect();

    let dominant = all_colors[0];
    let is_dark = dominant.luminance() < 0.5;

    // Monochromatic check (all clusters very similar to the dominant).
    let max_distance = all_colors
        .iter()
        .skip(1)
        .map(|c| dominant.distance(c))
        .fold(0.0f64, f64::max);
    let is_monochrome = max_distance < 40.0;

    // Background roles from the dominant color with lightness shifts.
    let (bg_primary, bg_secondary, bg_tertiary, bg_hover) = if is_dark {
        (
            dominant.shift_lightness(-0.05),
            dominant.shift_lightness(0.03),
            dominant.shift_lightness(0.08),
            dominant.shift_lightness(0.02),
        )
    } else {
        (
            dominant.shift_lightness(0.05),
            dominant.shift_lightness(-0.03),
            dominant.shift_lightness(-0.08),
            dominant.shift_lightness(-0.02),
        )
    };

    // Accent: most saturated cluster with AA contrast, monochrome → standard blue.
    let accent = if is_monochrome {
        if is_dark {
            PaletteColor::new(66, 133, 244)
        } else {
            PaletteColor::new(26, 115, 232)
        }
    } else {
        find_best_accent(&all_colors, &bg_primary, is_dark)
    };

    Ok(ThemePalette {
        bg_primary,
        bg_secondary,
        bg_tertiary,
        bg_hover,
        accent,
        is_dark,
        all_colors,
    })
}

/// Find the best accent: prioritize saturation, then ensure WCAG AA contrast.
fn find_best_accent(colors: &[PaletteColor], bg: &PaletteColor, is_dark: bool) -> PaletteColor {
    let mut candidates: Vec<(usize, f64)> = colors
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, c)| (i, c.saturation()))
        .collect();
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (idx, _sat) in &candidates {
        let color = colors[*idx];
        if color.contrast_ratio(bg) >= 4.5 {
            return color;
        }
        let adjusted = adjust_for_contrast(&color, bg, is_dark);
        if adjusted.contrast_ratio(bg) >= 4.5 {
            return adjusted;
        }
    }

    if let Some((idx, _)) = candidates.first() {
        let color = colors[*idx];
        adjust_for_contrast(&color, bg, is_dark)
    } else if is_dark {
        PaletteColor::new(66, 133, 244)
    } else {
        PaletteColor::new(26, 115, 232)
    }
}

/// Adjust a color's lightness to reach WCAG AA contrast against a background.
fn adjust_for_contrast(color: &PaletteColor, bg: &PaletteColor, is_dark: bool) -> PaletteColor {
    let (h, s, l) = color.to_hsl();
    let direction = if is_dark { 0.05 } else { -0.05 };

    let mut new_l = l;
    for _ in 0..20 {
        new_l = (new_l + direction).clamp(0.0, 1.0);
        let candidate = PaletteColor::from_hsl(h, s, new_l);
        if candidate.contrast_ratio(bg) >= 4.5 {
            return candidate;
        }
    }

    PaletteColor::from_hsl(h, s, new_l)
}
