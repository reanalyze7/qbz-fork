//! `qbz-theme` — the frontend-agnostic theme/palette registry (ADR-006).
//!
//! Pure Rust data + hand-rolled color/contrast math. NO Slint, NO Tauri, NO
//! heavy deps, so it compiles and unit-tests fast on its own and can be reused
//! by any frontend (Slint, the Tauri build, a TUI, contrast unit tests).
//!
//! The contract: a [`ThemeId`] maps to one fully-materialized [`ThemeColors`]
//! struct (no CSS cascade — every field is populated). The frontend converts
//! each [`Rgba`] to its own color type and pushes the struct into a single
//! theme global on theme change.
//!
//! Phase 1 materializes only the four existing themes; [`ThemeId::is_implemented`]
//! reports which rows are ready so the Settings list can expose only those.

pub mod auto;
mod color;
mod colors;
pub mod custom;
mod id;
mod registry;

pub use auto::{generate as generate_auto_theme, AutoSource};
pub use color::{apca_lc, contrast_ratio, relative_luminance, Rgba};
pub use colors::{alpha_byte, alpha_index, alpha_ramp, ThemeColors, ALPHA_COUNT, ALPHA_PERCENTS};
pub use custom::{base_from_theme, theme_from_base, CustomThemeBase};
pub use id::{default_slug, ThemeCategory, ThemeId, ALL};
pub use registry::palette;

/// A single entry in the Settings theme list: the stable id plus the data the
/// dropdown needs (display name, category, light/dark, ready flag).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeListEntry {
    pub id: ThemeId,
    pub display_name: &'static str,
    pub slug: &'static str,
    pub category: ThemeCategory,
    /// Luminance-derived (NOT the unreliable Tauri `type` flag): `true` when the
    /// theme's `surface_main` is light. Drives the dark/light list filter.
    pub is_light: bool,
    /// Whether the registry fully materializes this row yet (P1 gating).
    pub implemented: bool,
}

/// The default theme on a fresh profile (owner decision 2026-06-20: OLED Dark).
pub fn default_theme_id() -> ThemeId {
    ThemeId::default_id()
}

/// Whether a theme reads as "light" from its actual base surface luminance.
/// This is the corrected light/dark flag the plan mandates (Frost/Langley are
/// registered light in Tauri but are visually dark; Alucard is genuinely light).
pub fn is_light(id: ThemeId) -> bool {
    // System has no static palette; treat it as dark for filter purposes (it
    // follows the OS at runtime).
    if id == ThemeId::System {
        return false;
    }
    relative_luminance(palette(id).surface_main) >= 0.5
}

/// Whether a theme is one of the two High-Contrast accessibility themes.
/// Drives the Slint `Theme.is-high-contrast` flag, which gates HC-only
/// redundant-encoding affordances (1px control borders, slider-thumb borders)
/// so they never leak into the polished normal themes (P4 a11y pass).
pub fn is_high_contrast(id: ThemeId) -> bool {
    matches!(id, ThemeId::HighContrast | ThemeId::HighContrastLight)
}

/// Build the full Settings theme list in display order. The frontend filters by
/// `is_light` and may hide `!implemented` rows during P1/P2.
pub fn theme_list() -> Vec<ThemeListEntry> {
    ALL.iter()
        .map(|&id| ThemeListEntry {
            id,
            display_name: id.display_name(),
            slug: id.slug(),
            category: id.category(),
            is_light: is_light(id),
            implemented: id.is_implemented(),
        })
        .collect()
}

/// The implemented-only theme list (what the Settings dropdown shows in P1).
pub fn implemented_theme_list() -> Vec<ThemeListEntry> {
    theme_list().into_iter().filter(|e| e.implemented).collect()
}

#[cfg(test)]
mod tests;
