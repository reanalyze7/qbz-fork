//! Display-label lists and filtered index helpers layered on `dropdown.rs`.

use super::dropdown::{
    filtered_dropdown_themes, AUTO_LABEL, AUTO_SLUG, CUSTOM_LABEL, CUSTOM_SLUG, FILTER_ALL,
};
use super::id_lookup::id_for_slug;
use qbz_theme::ThemeId;

/// Display names for [`filtered_dropdown_themes`]. The synthetic "Auto
/// (dynamic)" and "Custom" entries are appended ONLY in the `All` view — they
/// have no fixed light/dark polarity, so a narrowed Dark/Light list omits them.
pub fn filtered_dropdown_labels(filter: i32) -> Vec<String> {
    let mut labels: Vec<String> = filtered_dropdown_themes(filter)
        .into_iter()
        .map(|id| id.display_name().to_string())
        .collect();
    if filter == FILTER_ALL {
        labels.push(AUTO_LABEL.to_string());
        labels.push(CUSTOM_LABEL.to_string());
    }
    labels
}

/// Dropdown index of the "Auto (dynamic)" entry within a filtered list, or `-1`
/// when the filter is not `All` (Auto/Custom are only shown in the `All` view).
pub fn filtered_auto_index(filter: i32) -> i32 {
    if filter == FILTER_ALL {
        filtered_dropdown_themes(filter).len() as i32
    } else {
        -1
    }
}

/// Dropdown index of the "Custom" entry within a filtered list, or `-1` when the
/// filter is not `All`.
pub fn filtered_custom_index(filter: i32) -> i32 {
    if filter == FILTER_ALL {
        filtered_auto_index(filter) + 1
    } else {
        -1
    }
}

/// Map a filtered-dropdown index to a `ThemeId`. Out-of-range indices (including
/// the Auto/Custom rows, which the caller handles separately) fall back to the
/// default theme.
pub fn filtered_id_for_index(index: i32, filter: i32) -> ThemeId {
    filtered_dropdown_themes(filter)
        .get(index as usize)
        .copied()
        .unwrap_or_else(qbz_theme::default_theme_id)
}

/// Position of a persisted slug within a filtered dropdown, auto/custom-aware.
/// Returns `-1` when the theme is not present under this filter (e.g. a dark
/// theme while the Light filter is active): the dropdown then highlights no row
/// while the theme itself stays applied (selection is slug-driven, not index).
pub fn filtered_selected_index_for_slug(slug: &str, filter: i32) -> i32 {
    if slug == AUTO_SLUG {
        filtered_auto_index(filter)
    } else if slug == CUSTOM_SLUG {
        filtered_custom_index(filter)
    } else {
        let id = id_for_slug(slug);
        filtered_dropdown_themes(filter)
            .iter()
            .position(|&t| t == id)
            .map(|p| p as i32)
            .unwrap_or(-1)
    }
}
