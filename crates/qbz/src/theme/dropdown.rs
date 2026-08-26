//! Auto/Custom synthetic entries, the Dark/Light filter, and the core
//! dropdown-index math over the registry's implemented theme list.

use qbz_theme::ThemeId;

use super::id_lookup::{id_for_slug, index_for_id};

/// Stable slug persisted for the dynamic "Auto" theme option. Distinct from the
/// registry slugs (it has no static `ThemeId`): the dropdown appends it after
/// the registry rows and `crate::auto_theme` generates the palette at runtime.
pub const AUTO_SLUG: &str = "auto";

/// Display label for the appended "Auto (dynamic)" dropdown entry. Like the
/// registry display names (`"System"`, `"Nord"`, …) this is proper-noun-style
/// UI data pushed from Rust, not a `@tr` catalog string.
pub const AUTO_LABEL: &str = "Auto (dynamic)";

/// Stable slug persisted for the user-authored "Custom" theme. Like `AUTO_SLUG`
/// it has no static `ThemeId`: the dropdown appends it after "Auto (dynamic)"
/// and `crate::custom_theme` derives the palette from `custom_theme.json`.
pub const CUSTOM_SLUG: &str = "custom";

/// Display label for the appended "Custom" dropdown entry. Proper-noun-style UI
/// data pushed from Rust, not a `@tr` catalog string (matches `AUTO_LABEL`).
pub const CUSTOM_LABEL: &str = "Custom";

/// Theme-list filter (persisted in `ui_prefs.theme_filter`, mirrored to the
/// Slint `AppearanceState.theme-filter`): `All` shows every theme, `Dark`/`Light`
/// narrow the dropdown by luminance (`ThemeListEntry.is_light`).
pub const FILTER_ALL: i32 = 0;
pub const FILTER_DARK: i32 = 1;
pub const FILTER_LIGHT: i32 = 2;

/// Dropdown index of the appended "Auto (dynamic)" entry (right after every
/// registry theme; the "Custom" entry follows it).
pub fn auto_index() -> i32 {
    dropdown_themes().len() as i32
}

/// Dropdown index of the appended "Custom" entry (last position, right after
/// "Auto (dynamic)").
pub fn custom_index() -> i32 {
    auto_index() + 1
}

/// The dropdown index for a persisted theme slug, auto/custom-aware: the two
/// synthetic slugs map to their appended entries, everything else through the
/// registry.
pub fn selected_index_for_slug(slug: &str) -> i32 {
    if slug == AUTO_SLUG {
        auto_index()
    } else if slug == CUSTOM_SLUG {
        custom_index()
    } else {
        index_for_id(id_for_slug(slug))
    }
}

/// The themes shown in the Settings dropdown, in display order. P1 exposes only
/// the implemented rows. The dropdown index is just a position in THIS list.
pub fn dropdown_themes() -> Vec<ThemeId> {
    filtered_dropdown_themes(FILTER_ALL)
}

/// The implemented themes for a given [filter](FILTER_ALL): `All` keeps every
/// theme; `Dark`/`Light` narrow by luminance (`ThemeListEntry.is_light`). The
/// order within each subset matches the full display order.
pub fn filtered_dropdown_themes(filter: i32) -> Vec<ThemeId> {
    qbz_theme::implemented_theme_list()
        .into_iter()
        .filter(|e| match filter {
            FILTER_DARK => !e.is_light,
            FILTER_LIGHT => e.is_light,
            _ => true,
        })
        .map(|e| e.id)
        .collect()
}
