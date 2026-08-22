use super::convert::parse;
use super::{base_from_theme, theme_from_base, CustomThemeBase};
use crate::color::{contrast_ratio, Rgba};
use crate::colors::ALPHA_COUNT;

#[test]
fn default_seed_is_oled_dark() {
    let base = CustomThemeBase::default_oled();
    assert!(base.is_dark);
    // OLED surfaces are pure/near black.
    assert_eq!(base.surface_main, "#000000");
    assert_eq!(base.surface_card, "#0a0a0a");
    assert_eq!(base.accent, "#4285f4");
}

#[test]
fn base_tokens_map_straight_through() {
    let base = CustomThemeBase::default_oled();
    let c = theme_from_base(&base);
    assert_eq!(c.surface_main, Rgba::rgb(0, 0, 0));
    assert_eq!(c.accent, Rgba::rgb(0x42, 0x85, 0xf4));
    assert_eq!(c.danger, Rgba::rgb(0xef, 0x44, 0x44));
    // focus_ring == accent; favorite is its own base token.
    assert_eq!(c.focus_ring, c.accent);
    assert_eq!(c.favorite, parse(&base.favorite, Rgba::rgb(0, 0, 0)));
}

#[test]
fn derived_status_families_have_expected_tints() {
    let c = theme_from_base(&CustomThemeBase::default_oled());
    // dark => hover 0.2
    assert_eq!(c.danger_bg.a, (0.1f32 * 255.0 + 0.5) as u8);
    assert_eq!(c.danger_border.a, (0.3f32 * 255.0 + 0.5) as u8);
    assert_eq!(c.danger_hover.a, (0.2f32 * 255.0 + 0.5) as u8);
    assert_eq!(c.danger_bg.r, c.danger.r);
    // whole status group shares one hover strength
    assert_eq!(c.success_hover.a, c.danger_hover.a);
    assert_eq!(c.warning_hover.a, c.danger_hover.a);
}

#[test]
fn polarity_drives_alpha_and_edges() {
    let dark = theme_from_base(&CustomThemeBase::default_oled());
    assert_eq!(dark.alpha.len(), ALPHA_COUNT);
    assert_eq!(dark.alpha[ALPHA_COUNT - 1].r, 255); // dark -> white base
    assert_eq!(dark.surface_hover, Rgba::rgba(255, 255, 255, 0x10));
    assert_eq!(dark.border_muted, Rgba::rgba(255, 255, 255, 0x38));

    let mut light_base = CustomThemeBase::default_oled();
    light_base.is_dark = false;
    light_base.surface_main = "#ffffff".into();
    light_base.text_primary = "#0f0f0f".into();
    let light = theme_from_base(&light_base);
    assert_eq!(light.alpha[ALPHA_COUNT - 1].r, 0); // light -> black base
    assert_eq!(light.surface_hover, Rgba::rgba(0, 0, 0, 0x10));
    // light hover strength is 0.15
    assert_eq!(light.danger_hover.a, (0.15f32 * 255.0 + 0.5) as u8);
}

#[test]
fn seed_derive_roundtrip_is_coherent() {
    // For any theme this module authored, reducing the derived colors back to
    // a base reproduces every base token exactly (idempotent seed).
    let base = CustomThemeBase::default_oled();
    let derived = theme_from_base(&base);
    let base2 = base_from_theme(&derived, base.is_dark);
    assert_eq!(base2, base);
}

#[test]
fn deterministic() {
    let base = CustomThemeBase::default_oled();
    assert_eq!(theme_from_base(&base), theme_from_base(&base));
}

#[test]
fn accent_text_is_legible() {
    let c = theme_from_base(&CustomThemeBase::default_oled());
    // The picked accent-text is pure white or pure black...
    assert!(c.accent_text == Rgba::rgb(255, 255, 255) || c.accent_text == Rgba::rgb(0, 0, 0));
    // ...and it is the more legible of the two against the accent fill.
    let white = Rgba::rgb(255, 255, 255);
    let black = Rgba::rgb(0, 0, 0);
    let picked = contrast_ratio(c.accent_text, c.accent);
    let other = if c.accent_text == white {
        contrast_ratio(black, c.accent)
    } else {
        contrast_ratio(white, c.accent)
    };
    // pick_btn prefers white when it clears 3:1; otherwise the higher one.
    assert!(picked >= 3.0 || picked >= other);
}

#[test]
fn malformed_hex_falls_back_not_panics() {
    let mut base = CustomThemeBase::default_oled();
    base.accent = "not-a-color".into();
    let c = theme_from_base(&base);
    // Falls back to the :root Dark accent.
    assert_eq!(c.accent, Rgba::rgb(0x42, 0x85, 0xf4));
}

#[test]
fn json_roundtrip() {
    let base = CustomThemeBase::default_oled();
    let json = serde_json::to_string(&base).unwrap();
    let back: CustomThemeBase = serde_json::from_str(&json).unwrap();
    assert_eq!(back, base);
}
