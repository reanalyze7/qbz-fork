//! Theme application: bridge the frontend-agnostic `qbz-theme` registry to the
//! Slint `Theme` global.
//!
//! `qbz-theme` owns all palette data (ADR-006). This module is the only place
//! that knows about both `qbz_theme::Rgba` and the generated Slint `ThemeColors`
//! struct: it converts one to the other and pushes it into `Theme.c`, plus sets
//! the `Theme.is-system` flag so the System theme keeps following the OS.
//!
//! The dropdown shows a filtered/ordered list of themes; the stable slug is the
//! source of truth (persisted in `ui_prefs.theme`), and the dropdown index is
//! DERIVED from it — never the reverse.

use crate::{AppWindow, Theme as SlintTheme};
use qbz_theme::ThemeId;
use slint::ComponentHandle;

mod dropdown;
mod dropdown_labels;
mod id_lookup;
mod palette_map;

#[cfg(test)]
mod tests;

pub use dropdown::*;
pub use dropdown_labels::*;
pub use id_lookup::*;

use palette_map::to_slint;

/// Push a fully-materialized registry `ThemeColors` into the running window's
/// `Theme` global. Shared by [`apply_theme`] (static themes) and the auto-theme
/// path (`crate::auto_theme`), so both go through the exact same conversion +
/// global-set sequence.
pub fn push_colors(
    window: &AppWindow,
    colors: &qbz_theme::ThemeColors,
    is_system: bool,
    is_high_contrast: bool,
) {
    let theme = window.global::<SlintTheme>();
    theme.set_c(to_slint(colors));
    theme.set_is_system(is_system);
    theme.set_is_high_contrast(is_high_contrast);
    // Relative luminance (BT.709) of the base surface -> is-dark. Drives the
    // std-widgets `Palette.color-scheme` in app.slint so native inputs follow
    // the QBZ theme; computed here (not from ThemeId) so it's correct for the
    // auto/custom themes too. System keeps following the OS (app.slint sets the
    // scheme to `unknown` when is-system, ignoring this flag).
    let s = colors.surface_main;
    let luma = 0.2126 * s.r as f64 + 0.7152 * s.g as f64 + 0.0722 * s.b as f64;
    theme.set_is_dark(luma < 128.0);
}

/// Push the palette for `id` into the running window's `Theme` global. Sets
/// `is-system` so the System theme follows the OS (the struct is still pushed as
/// a sane fallback for any non-System-overridden tokens).
pub fn apply_theme(window: &AppWindow, id: ThemeId) {
    let colors = qbz_theme::palette(id);
    push_colors(window, &colors, id == ThemeId::System, qbz_theme::is_high_contrast(id));
    log::info!("[qbz-slint] applied theme '{}'", id.slug());
}
