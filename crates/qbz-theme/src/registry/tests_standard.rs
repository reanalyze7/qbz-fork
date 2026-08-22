//! Standard-theme spot-checks: exact transcribed hero values per theme, plus
//! polarity sanity checks. Split out of `tests_basic.rs` for the 130-line
//! budget.

use super::*;

#[test]
fn light_core_values() {
    let c = palette(ThemeId::Light);
    assert_eq!(c.surface_main, Rgba::rgb(0xff, 0xff, 0xff));
    assert_eq!(c.text_primary, Rgba::rgb(0x0f, 0x0f, 0x0f));
    // accent trio inherited from :root Dark:
    assert_eq!(c.accent, Rgba::rgb(0x42, 0x85, 0xf4));
    assert_eq!(c.accent_text, Rgba::rgb(0xff, 0xff, 0xff));
    assert_eq!(c.danger, Rgba::rgb(0xdc, 0x26, 0x26));
    assert_eq!(c.border_subtle, Rgba::rgb(0xe0, 0xe0, 0xe0));
    // light theme -> black alpha base
    assert_eq!(c.alpha_pct(8), Rgba::rgba(0, 0, 0, 0x14));
    // light hover tint is 0.15 (faithful to app.css)
    assert_eq!(c.danger_hover, with_alpha(Rgba::rgb(0xdc, 0x26, 0x26), 0.15));
}

#[test]
fn dracula_nonstandard_tints() {
    let c = palette(ThemeId::Dracula);
    assert_eq!(c.surface_main, Rgba::rgb(0x28, 0x2a, 0x36));
    assert_eq!(c.accent, Rgba::rgb(0xbd, 0x93, 0xf9));
    let danger = Rgba::rgb(0xff, 0x55, 0x55);
    assert_eq!(c.danger_bg, with_alpha(danger, 0.15));
    assert_eq!(c.danger_border, with_alpha(danger, 0.4));
    assert_eq!(c.danger_hover, with_alpha(danger, 0.25));
}

#[test]
fn breeze_dark_inherits_root_status_hues() {
    let c = palette(ThemeId::BreezeDark);
    // danger/warning inherited from :root Dark
    assert_eq!(c.danger, Rgba::rgb(0xef, 0x44, 0x44));
    assert_eq!(c.warning, Rgba::rgb(0xfb, 0xbf, 0x24));
    assert_eq!(c.accent, Rgba::rgb(0x3d, 0xae, 0xe9));
}

#[test]
fn frost_langley_are_dark_polarity() {
    // Both are registered type:light in Tauri but are DARK canvases.
    let frost = palette(ThemeId::Frost);
    let langley = palette(ThemeId::Langley);
    // white alpha base (dark polarity), NOT black:
    assert_eq!(frost.alpha_pct(8), Rgba::rgba(255, 255, 255, 0x14));
    assert_eq!(langley.alpha_pct(8), Rgba::rgba(255, 255, 255, 0x14));
    assert!(!bg_is_light(frost.surface_main));
    assert!(!bg_is_light(langley.surface_main));
}

#[test]
fn alucard_is_light_polarity() {
    let c = palette(ThemeId::Alucard);
    assert_eq!(c.surface_main, Rgba::rgb(0xff, 0xfb, 0xeb));
    // black alpha base (light polarity):
    assert_eq!(c.alpha_pct(8), Rgba::rgba(0, 0, 0, 0x14));
    assert!(bg_is_light(c.surface_main));
    // success on a light theme is the darker green:
    assert_eq!(c.success, Rgba::rgb(0x1f, 0x8a, 0x4c));
}

#[test]
fn light_themes_use_black_alpha_base() {
    for id in [
        ThemeId::Light,
        ThemeId::Alucard,
        ThemeId::RosePineDawn,
        ThemeId::BreezeLight,
        ThemeId::AdwaitaLight,
        ThemeId::DuotoneSnow,
        ThemeId::SnowStorm,
        ThemeId::Kurosaki,
    ] {
        let c = palette(id);
        assert_eq!(c.alpha_pct(8), Rgba::rgba(0, 0, 0, 0x14), "{id:?} should be black-base");
        assert_eq!(c.surface_hover, Rgba::rgba(0, 0, 0, 0x10), "{id:?} hover base");
    }
}

#[test]
fn dark_themes_use_white_alpha_base() {
    for id in [
        ThemeId::Warm,
        ThemeId::Nord,
        ThemeId::Dracula,
        ThemeId::CatppuccinMocha,
        ThemeId::BreezeDark,
        ThemeId::AdwaitaDark,
        ThemeId::Aurora,
        ThemeId::Ikari,
        ThemeId::Ayanami,
        ThemeId::Iscariot,
        ThemeId::Stratego,
        ThemeId::Rumi,
        ThemeId::Zoey,
        ThemeId::Mira,
        ThemeId::Frost,
        ThemeId::Langley,
    ] {
        let c = palette(id);
        assert_eq!(c.alpha_pct(8), Rgba::rgba(255, 255, 255, 0x14), "{id:?} should be white-base");
    }
}

#[test]
fn standard_theme_focus_ring_equals_accent() {
    for id in [
        ThemeId::Warm,
        ThemeId::Nord,
        ThemeId::Stratego,
        ThemeId::Alucard,
        ThemeId::Kurosaki,
    ] {
        let c = palette(id);
        assert_eq!(c.focus_ring, c.accent, "{id:?} focus_ring should equal accent");
        assert_eq!(c.favorite, c.danger, "{id:?} favorite should equal danger");
    }
}

#[test]
fn alpha_byte_helper_matches_with_alpha() {
    // sanity: with_alpha(.., 0.1) == alpha_byte(10)
    let c = Rgba::rgb(0x10, 0x20, 0x30);
    assert_eq!(with_alpha(c, 0.1).a, crate::colors::alpha_byte(10));
}
