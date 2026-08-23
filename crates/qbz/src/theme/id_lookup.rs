//! Slug/index/id lookups over the (unfiltered) dropdown list.

use qbz_theme::ThemeId;

use super::dropdown::dropdown_themes;

/// Map a persisted slug to a `ThemeId`, falling back to the default (OLED) when
/// the slug is unknown or absent.
pub fn id_for_slug(slug: &str) -> ThemeId {
    ThemeId::from_slug(slug).unwrap_or_else(qbz_theme::default_theme_id)
}

/// Map a dropdown index to a `ThemeId`. Out-of-range indices fall back to the
/// default theme.
pub fn id_for_index(index: i32) -> ThemeId {
    let list = dropdown_themes();
    list.get(index as usize)
        .copied()
        .unwrap_or_else(qbz_theme::default_theme_id)
}

/// Derive the dropdown index for a `ThemeId` (position in [`dropdown_themes`]).
/// Returns `0` if the id is not in the dropdown list (e.g. a P2/P3 theme not yet
/// exposed) — the caller should treat that as "no explicit selection".
pub fn index_for_id(id: ThemeId) -> i32 {
    dropdown_themes()
        .iter()
        .position(|&t| t == id)
        .map(|p| p as i32)
        .unwrap_or(0)
}
