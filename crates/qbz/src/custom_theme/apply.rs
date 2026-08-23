//! Live palette derivation + push, and startup/seed entry points.

use qbz_theme::CustomThemeBase;

use crate::AppWindow;

use super::persist::{load, load_or_seed, save};
use super::state::push_base_to_state;

/// Derive `base`, push the palette live, and persist — WITHOUT re-seeding the
/// editor swatches (the caller updates only the touched swatch). Used by the
/// per-token and polarity edits so the inline picker stays open.
pub(super) fn apply_live(window: &AppWindow, base: &CustomThemeBase) {
    let colors = qbz_theme::theme_from_base(base);
    crate::theme::push_colors(window, &colors, false, false);
    save(base);
}

/// Seed the custom-theme editor swatches from the persisted base (or the OLED
/// default when none exists). Runs at startup for every user, so it uses the
/// non-persisting [`load`] — the `custom_theme.json` file is only created once
/// the user actually selects/edits the Custom theme (via [`load_or_seed`]).
pub fn seed_state(window: &AppWindow) {
    let base = load();
    push_base_to_state(window, &base);
}

/// Startup apply: derive the persisted (or freshly seeded) custom base and push
/// the palette. Runs inline on the event-loop thread during window init so the
/// first paint is already the custom palette.
pub fn apply_startup(window: &AppWindow) {
    let base = load_or_seed();
    let colors = qbz_theme::theme_from_base(&base);
    crate::theme::push_colors(window, &colors, false, false);
    log::info!("[qbz-slint] applied custom theme");
}
