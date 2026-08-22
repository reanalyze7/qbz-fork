use super::{default_slug, ThemeId, ALL};

#[test]
fn slug_roundtrip_all() {
    for &id in ALL {
        assert_eq!(ThemeId::from_slug(id.slug()), Some(id), "slug {} failed", id.slug());
    }
}

#[test]
fn slugs_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for &id in ALL {
        assert!(seen.insert(id.slug()), "duplicate slug {}", id.slug());
    }
    assert_eq!(seen.len(), ALL.len());
}

#[test]
fn default_is_oled() {
    assert_eq!(ThemeId::default_id(), ThemeId::Oled);
    assert_eq!(default_slug(), "oled");
}

#[test]
fn unknown_slug_is_none() {
    assert_eq!(ThemeId::from_slug("does-not-exist"), None);
}

#[test]
fn p1_themes_implemented() {
    for id in [ThemeId::Dark, ThemeId::Oled, ThemeId::TokyoNight, ThemeId::System] {
        assert!(id.is_implemented(), "{:?} should be P1-implemented", id);
    }
}
