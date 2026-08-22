//! User-authored custom theme: derive a full [`ThemeColors`] set from a small
//! set of hand-picked base tokens.
//!
//! This is the EXTENDED port of the Tauri "edit theme colors" feature. Tauri
//! only let the user override 8 raw CSS variables on top of the active
//! auto-theme, with no derivation — editing the accent left hover/pressed/tints
//! stale. Here the user edits ~12 semantic BASE tokens and the whole rest of the
//! contract is DERIVED from them (accent triplet, status families, muted/disabled
//! text tiers, focus ring, translucent edges, polarity alpha ramp), reusing the
//! exact same math the auto-theme generator and the static registry already use
//! (`generator::{tint, pick_btn_text_for_accent_set, ensure_text_contrast_target}`,
//! `PaletteColor::shift_lightness`, `alpha_ramp`).
//!
//! Colors are stored as `#rrggbb` HEX STRINGS (opaque; alpha is never part of a
//! base token). Rationale: the on-disk `custom_theme.json` stays human-readable
//! and greppable, and the value round-trips 1:1 with the Slint ColorPicker's HEX
//! field. Malformed strings fall back to the dark-theme default for that token,
//! so a hand-edited file can never panic the app.

mod convert;
mod derive;
mod reduce;
#[cfg(test)]
mod tests;

pub use derive::theme_from_base;
pub use reduce::base_from_theme;

use crate::id::ThemeId;
use serde::{Deserialize, Serialize};

/// The user-editable BASE of a custom theme. Twelve semantic tokens the editor
/// exposes as swatches; everything else in [`crate::colors::ThemeColors`] is
/// derived from these by [`theme_from_base`]. All colors are opaque `#rrggbb`
/// hex strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomThemeBase {
    /// Polarity. Drives the alpha-ramp base (white on dark / black on light), the
    /// translucent edge/hover bases, and the direction every derived shift takes.
    pub is_dark: bool,

    // --- surfaces (SURFACES group in the editor) ---
    pub surface_main: String,
    pub surface_card: String,
    pub surface_elevated: String,

    // --- text (TEXT group) ---
    pub text_primary: String,
    pub text_secondary: String,

    // --- accent (ACCENT group) ---
    pub accent: String,

    // --- status (STATUS group) ---
    pub danger: String,
    pub warning: String,
    pub success: String,

    // --- other (OTHER group) ---
    pub border: String,
    pub favorite: String,
}

impl CustomThemeBase {
    /// The seed a fresh custom theme starts from: the default OLED Black palette
    /// reduced to its base tokens. Used when the user first selects "Custom" and
    /// no `custom_theme.json` exists yet.
    pub fn default_oled() -> Self {
        base_from_theme(&crate::registry::palette(ThemeId::Oled), true)
    }
}
