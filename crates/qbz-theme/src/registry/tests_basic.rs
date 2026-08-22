use super::*;
use crate::colors::ALPHA_COUNT;
use crate::id::ALL;

/// Sentinel for "field was never set" — the all-zero opaque/transparent
/// black the `StdSpec::default()` placeholder uses. A fully-materialized row
/// must not leave a meaningful color at this sentinel by accident; we instead
/// assert the named hero tokens are the EXACT transcribed values per theme in
/// the dedicated tests below, and assert global completeness here.
fn fully_populated(c: &ThemeColors) {
    assert_eq!(c.alpha.len(), ALPHA_COUNT);
    // every alpha tier carries opacity
    assert!(c.alpha.iter().all(|a| a.a > 0));
    // status families are present (non-degenerate alpha)
    assert!(c.danger_bg.a > 0 && c.danger_border.a > 0 && c.danger_hover.a > 0);
    assert!(c.warning_bg.a > 0 && c.warning_border.a > 0 && c.warning_hover.a > 0);
    assert!(c.success_bg.a > 0 && c.success_border.a > 0 && c.success_hover.a > 0);
    // surfaces/text/accent are opaque
    assert_eq!(c.surface_main.a, 255);
    assert_eq!(c.text_primary.a, 255);
    assert_eq!(c.accent.a, 255);
    assert_eq!(c.accent_text.a, 255);
    assert_eq!(c.border_strong.a, 255);
    assert_eq!(c.focus_ring.a, 255);
}

#[test]
fn every_registered_theme_is_fully_populated() {
    for &id in ALL {
        let c = palette(id);
        fully_populated(&c);
    }
}

#[test]
fn p1_rows_are_fully_populated() {
    for id in [ThemeId::Dark, ThemeId::Oled, ThemeId::TokyoNight, ThemeId::System] {
        let c = palette(id);
        assert_eq!(c.alpha.len(), ALPHA_COUNT);
        assert_ne!(c.surface_main, Rgba::rgba(0, 0, 0, 0));
        assert_ne!(c.text_primary, Rgba::rgba(0, 0, 0, 0));
        assert_ne!(c.accent, Rgba::rgba(0, 0, 0, 0));
        assert!(c.alpha.iter().all(|a| a.a > 0));
    }
}

#[test]
fn dark_matches_root_css() {
    let c = palette(ThemeId::Dark);
    assert_eq!(c.surface_main, Rgba::rgb(0x0f, 0x0f, 0x0f));
    assert_eq!(c.surface_card, Rgba::rgb(0x1a, 0x1a, 0x1a));
    assert_eq!(c.surface_elevated, Rgba::rgb(0x2a, 0x2a, 0x2a));
    assert_eq!(c.text_primary, Rgba::rgb(0xff, 0xff, 0xff));
    assert_eq!(c.accent, Rgba::rgb(0x42, 0x85, 0xf4));
    assert_eq!(c.border_strong, Rgba::rgb(0x3a, 0x3a, 0x3a));
    assert_eq!(c.favorite, c.danger);
}

#[test]
fn oled_overrides_only_surfaces_and_borders() {
    let d = palette(ThemeId::Dark);
    let o = palette(ThemeId::Oled);
    assert_eq!(o.surface_main, Rgba::rgb(0, 0, 0));
    assert_eq!(o.surface_card, Rgba::rgb(0x0a, 0x0a, 0x0a));
    assert_eq!(o.surface_elevated, Rgba::rgb(0x1a, 0x1a, 0x1a));
    assert_eq!(o.bg_hover, Rgba::rgb(0x11, 0x11, 0x11));
    assert_eq!(o.border_strong, Rgba::rgb(0x2a, 0x2a, 0x2a));
    assert_eq!(o.accent, d.accent);
    assert_eq!(o.text_primary, d.text_primary);
    assert_eq!(o.danger, d.danger);
}

#[test]
fn tokyo_legacy_values_preserved() {
    let c = palette(ThemeId::TokyoNight);
    assert_eq!(c.surface_main, Rgba::rgb(0x1a, 0x1b, 0x26));
    assert_eq!(c.surface_card, Rgba::rgb(0x16, 0x16, 0x1e));
    assert_eq!(c.surface_elevated, Rgba::rgb(0x1c, 0x1d, 0x29));
    assert_eq!(c.text_primary, Rgba::rgb(0xa9, 0xb1, 0xd6));
    assert_eq!(c.accent, Rgba::rgb(0x7a, 0xa2, 0xf7));
    assert_eq!(c.accent_text, Rgba::rgb(0x1a, 0x1b, 0x26));
}

#[test]
fn legacy_alpha_aliases_unchanged() {
    // The exact translucent values the old Slint Theme exposed (dark themes).
    for id in [ThemeId::Dark, ThemeId::Oled, ThemeId::TokyoNight] {
        let c = palette(id);
        assert_eq!(c.surface_hover, Rgba::rgba(255, 255, 255, 0x10));
        assert_eq!(c.border_subtle, Rgba::rgba(255, 255, 255, 0x14));
        assert_eq!(c.border_muted, Rgba::rgba(255, 255, 255, 0x38));
        assert_eq!(c.card_shadow, Rgba::rgba(0, 0, 0, 0x66));
        assert_eq!(c.alpha_pct(8), Rgba::rgba(255, 255, 255, 0x14));
        assert_eq!(c.alpha_pct(10), Rgba::rgba(255, 255, 255, 0x1a));
        assert_eq!(c.alpha_pct(12), Rgba::rgba(255, 255, 255, 0x1f));
        assert_eq!(c.alpha_pct(18), Rgba::rgba(255, 255, 255, 0x2e));
        assert_eq!(c.alpha_pct(55), Rgba::rgba(255, 255, 255, 0x8c));
        assert_eq!(c.alpha_pct(65), Rgba::rgba(255, 255, 255, 0xa6));
        assert_eq!(c.alpha_pct(70), Rgba::rgba(255, 255, 255, 0xb3));
        assert_eq!(c.alpha_pct(75), Rgba::rgba(255, 255, 255, 0xbf));
    }
}
