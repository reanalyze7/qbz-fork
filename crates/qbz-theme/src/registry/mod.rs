//! The theme registry: `ThemeId` -> fully-materialized [`ThemeColors`].
//!
//! P1 materialized the four existing themes (Dark / OLED / Tokyo Night /
//! System-fallback). P2 (this file) transcribes the remaining **standard**
//! (non-accessibility) themes from `src/app.css` — every value cites the
//! `qbz-nix-docs/.../01-tauri-themes-inventory.md` table (which was read 1:1
//! from `src/app.css`). P3 will add the redesigned accessibility rows.
//!
//! No CSS cascade on the Slint side: every row is FULLY materialized. Tauri
//! themes that OMIT tokens (e.g. `light` omits the accent trio; `oled`/
//! `breeze-dark`/`adwaita-dark` omit whole danger/warning families) inherit
//! those from `:root` Dark — so the omissions are resolved against `dark()` at
//! transcription time, here, not at runtime.
//!
//! Derived (no Tauri parity) tokens for the standard rows:
//!   - `success` family: NEW. Tauri has no `--success`. We use the project green
//!     `#3fae6a` for dark themes (matches P1) and a darker `#1f8a4c` for light
//!     themes (so success text clears >=3:1 on a light surface), with the same
//!     0.1/0.3/0.2 tint shape for bg/border/hover. Polished in P4.
//!   - `focus_ring`: NEW (WCAG 2.4.7). Uses the theme accent (high-visibility,
//!     matches P1). Polished in P4.
//!   - `favorite`: the loved-heart uses the theme `danger` hue (matches P1).
//!   - `danger_bg/border/hover`, `warning_*`: Tauri expresses these as `rgba()`
//!     tints of the solid hue at 0.1/0.3/0.2 (dracula uses 0.15/0.4/0.25). We
//!     bake the same straight-alpha overlays so they composite identically.
//!   - `border_muted`: legacy Slint-only token (no Tauri var). Polarity-aware
//!     translucent edge (white ~22% on dark, black ~22% on light).
//!   - `surface_hover`: alpha-based hover overlay, polarity-aware (white ~6% on
//!     dark), distinct from the opaque theme `--bg-hover`.

mod a11y_colorblind;
mod a11y_dark;
mod a11y_light;
mod core_themes;
mod core_themes_2;
mod dark_themes_1;
mod dark_themes_2;
mod dark_themes_3;
mod dark_themes_4;
mod dark_themes_5;
mod legacy;
mod light_themes_1;
mod light_themes_2;
mod std_spec;
mod std_spec_default;
#[cfg(test)]
mod tests_a11y;
#[cfg(test)]
mod tests_a11y_colorblind;
#[cfg(test)]
mod tests_a11y_global;
#[cfg(test)]
mod tests_basic;
#[cfg(test)]
mod tests_standard;

#[allow(unused_imports)] // used by the `#[cfg(test)]` submodules via `super::*`
use crate::color::Rgba;
use crate::colors::ThemeColors;
use crate::id::ThemeId;

use a11y_colorblind::colorblind;
use a11y_dark::{high_contrast, wcag_dark};
use a11y_light::{high_contrast_light, wcag_light};
use core_themes::{dark, oled};
use core_themes_2::{light, sepia, tokyo_night};
use dark_themes_1::{catppuccin_mocha, dracula, nord, warm};
use dark_themes_2::{breeze_dark, catppuccin_frappe, catppuccin_latte, catppuccin_macchiato};
use dark_themes_3::{adwaita_dark, ayanami, aurora, ikari};
use dark_themes_4::{iscariot, rumi, stratego, zoey};
use dark_themes_5::{frost, langley, mira};
use light_themes_1::{alucard, breeze_light, rose_pine_dawn};
use light_themes_2::{adwaita_light, duotone_snow, kurosaki, snow_storm};
use legacy::{LEGACY_BORDER_MUTED, LEGACY_BORDER_SUBTLE, LEGACY_CARD_SHADOW, LEGACY_SURFACE_HOVER};
#[allow(unused_imports)] // bg_is_light: used by `#[cfg(test)]` submodules via `super::*`
use std_spec::{bg_is_light, with_alpha};

/// Resolve a theme id to its concrete color set.
///
/// `System` has no static palette — at runtime the Slint side follows the OS
/// (std-widgets `Palette`) for the tokens it overrides, exactly as before. This
/// returns the Dark set as a safe fallback for any caller that needs a concrete
/// struct for `System` (it is NOT what paints the System theme; that stays the
/// `is-system` path in `theme.slint`).
pub fn palette(id: ThemeId) -> ThemeColors {
    match id {
        // --- Core (P1 + the standard Light) ---
        ThemeId::Dark => dark(),
        ThemeId::Oled => oled(),
        ThemeId::TokyoNight => tokyo_night(),
        ThemeId::System => dark(),
        ThemeId::Light => light(),
        ThemeId::Sepia => sepia(),
        // --- Dark (branded / community) ---
        ThemeId::Warm => warm(),
        ThemeId::Nord => nord(),
        ThemeId::Dracula => dracula(),
        ThemeId::CatppuccinMocha => catppuccin_mocha(),
        ThemeId::CatppuccinLatte => catppuccin_latte(),
        ThemeId::CatppuccinFrappe => catppuccin_frappe(),
        ThemeId::CatppuccinMacchiato => catppuccin_macchiato(),
        ThemeId::BreezeDark => breeze_dark(),
        ThemeId::AdwaitaDark => adwaita_dark(),
        ThemeId::Aurora => aurora(),
        ThemeId::Ikari => ikari(),
        ThemeId::Ayanami => ayanami(),
        ThemeId::Iscariot => iscariot(),
        ThemeId::Stratego => stratego(),
        ThemeId::Rumi => rumi(),
        ThemeId::Zoey => zoey(),
        ThemeId::Mira => mira(),
        ThemeId::Frost => frost(),
        ThemeId::Langley => langley(),
        // --- Light (branded / community) ---
        ThemeId::Alucard => alucard(),
        ThemeId::RosePineDawn => rose_pine_dawn(),
        ThemeId::BreezeLight => breeze_light(),
        ThemeId::AdwaitaLight => adwaita_light(),
        ThemeId::DuotoneSnow => duotone_snow(),
        ThemeId::SnowStorm => snow_storm(),
        ThemeId::Kurosaki => kurosaki(),
        // --- Accessibility (REDESIGNED in P3): final verified palettes from
        // 99-MIGRATION-PLAN.md Part B (adversarial corrections folded in). ---
        ThemeId::WcagLight => wcag_light(),
        ThemeId::WcagDark => wcag_dark(),
        ThemeId::HighContrast => high_contrast(),
        ThemeId::HighContrastLight => high_contrast_light(),
        ThemeId::Colorblind => colorblind(),
    }
}
