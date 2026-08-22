use super::*;

#[test]
fn default_is_oled() {
    assert_eq!(default_theme_id(), ThemeId::Oled);
}

#[test]
fn registry_returns_populated_struct_for_default() {
    let c = palette(default_theme_id());
    // OLED: pure-black base, white text, full alpha ramp.
    assert_eq!(c.surface_main, Rgba::rgb(0, 0, 0));
    assert_eq!(c.text_primary, Rgba::rgb(0xff, 0xff, 0xff));
    assert_eq!(c.alpha.len(), ALPHA_COUNT);
}

#[test]
fn light_dark_flag_from_luminance() {
    // The 4 P1 themes are all dark.
    assert!(!is_light(ThemeId::Dark));
    assert!(!is_light(ThemeId::Oled));
    assert!(!is_light(ThemeId::TokyoNight));
    assert!(!is_light(ThemeId::System));
}

#[test]
fn implemented_list_is_every_theme() {
    // After P3 every theme is materialized — the standard rows AND the 5
    // redesigned accessibility themes (WcagLight/WcagDark/HighContrast/
    // HighContrastLight/Colorblind).
    let list = implemented_theme_list();
    assert_eq!(list.len(), ALL.len());
    let slugs: Vec<&str> = list.iter().map(|e| e.slug).collect();
    // P1 originals still present:
    assert!(slugs.contains(&"dark"));
    assert!(slugs.contains(&"oled"));
    assert!(slugs.contains(&"tokyo-night"));
    assert!(slugs.contains(&"system"));
    // P2 additions (spot-check across categories):
    assert!(slugs.contains(&"light"));
    assert!(slugs.contains(&"nord"));
    assert!(slugs.contains(&"dracula"));
    assert!(slugs.contains(&"frost"));
    assert!(slugs.contains(&"langley"));
    assert!(slugs.contains(&"alucard"));
    assert!(slugs.contains(&"kurosaki"));
    // P3 accessibility themes now implemented:
    assert!(slugs.contains(&"wcag-light"));
    assert!(slugs.contains(&"wcag-dark"));
    assert!(slugs.contains(&"high-contrast"));
    assert!(slugs.contains(&"high-contrast-light"));
    assert!(slugs.contains(&"colorblind"));
}

#[test]
fn light_dark_filter_is_luminance_correct() {
    // Corrected flags: Alucard light; Frost/Langley dark despite Tauri type.
    assert!(is_light(ThemeId::Alucard));
    assert!(is_light(ThemeId::Light));
    assert!(is_light(ThemeId::SnowStorm));
    assert!(!is_light(ThemeId::Frost));
    assert!(!is_light(ThemeId::Langley));
    assert!(!is_light(ThemeId::Nord));
}

#[test]
fn full_list_has_all_entries() {
    assert_eq!(theme_list().len(), ALL.len());
}

#[test]
fn high_contrast_flag_only_for_hc_themes() {
    // True for exactly the two High-Contrast themes.
    assert!(is_high_contrast(ThemeId::HighContrast));
    assert!(is_high_contrast(ThemeId::HighContrastLight));
    // False for everything else, including the other a11y themes.
    assert!(!is_high_contrast(ThemeId::WcagLight));
    assert!(!is_high_contrast(ThemeId::WcagDark));
    assert!(!is_high_contrast(ThemeId::Colorblind));
    assert!(!is_high_contrast(ThemeId::Dark));
    assert!(!is_high_contrast(ThemeId::Oled));
    assert!(!is_high_contrast(ThemeId::System));
    // Exactly two themes in ALL are high-contrast.
    assert_eq!(ALL.iter().filter(|&&id| is_high_contrast(id)).count(), 2);
}
