//! Shared builder for the standard (non-accessibility) themes.

use crate::color::{relative_luminance, Rgba};
use crate::colors::{alpha_ramp, ThemeColors};

use super::LEGACY_CARD_SHADOW;

/// The Tauri token set for one standard theme, as read from doc 01. Only the
/// named hues are carried here; the derived families (success/focus/favorite),
/// the polarity-driven alpha ramp + translucent edges, and the status tints are
/// materialized by [`StdSpec::build`].
#[derive(Clone, Copy)]
pub(super) struct StdSpec {
    // surfaces (--bg-*)
    pub(super) bg_primary: Rgba,
    pub(super) bg_secondary: Rgba,
    pub(super) bg_tertiary: Rgba,
    pub(super) bg_hover: Rgba,
    // text (--text-*)
    pub(super) text_primary: Rgba,
    pub(super) text_secondary: Rgba,
    pub(super) text_muted: Rgba,
    pub(super) text_disabled: Rgba,
    // accent (--accent-* + --btn-primary-text)
    pub(super) accent: Rgba,
    pub(super) accent_hover: Rgba,
    pub(super) accent_pressed: Rgba,
    pub(super) accent_text: Rgba,
    // status hues (--danger / --warning); families derived as rgba() tints
    pub(super) danger: Rgba,
    pub(super) warning: Rgba,
    /// Tint fractions for the danger/warning bg/border/hover families.
    /// Standard themes use (0.1, 0.3, 0.2); dracula uses (0.15, 0.4, 0.25).
    pub(super) tint_bg: f32,
    pub(super) tint_border: f32,
    pub(super) tint_hover: f32,
    // borders (--border-*)
    pub(super) border_subtle: Rgba,
    pub(super) border_strong: Rgba,
}

impl StdSpec {
    /// Default status-tint fractions (every theme except dracula).
    pub(super) const TINT_BG: f32 = 0.1;
    pub(super) const TINT_BORDER: f32 = 0.3;
    pub(super) const TINT_HOVER: f32 = 0.2;

    /// Materialize a complete [`ThemeColors`] row. `is_light` is the corrected
    /// (luminance-derived) polarity — it drives the alpha ramp base (black on
    /// light, white on dark), the translucent edge/hover bases, and the derived
    /// `success` hue. NOTE: do NOT trust the Tauri `type` flag for this; pass the
    /// real luminance (Frost/Langley are registered light but are dark canvases).
    pub(super) fn build(self, is_light: bool) -> ThemeColors {
        // success: NEW token, no Tauri parity. Theme-appropriate green that
        // clears >=3:1 on the theme surface; darker on light themes. Polished P4.
        let success = if is_light {
            Rgba::rgb(0x1f, 0x8a, 0x4c)
        } else {
            Rgba::rgb(0x3f, 0xae, 0x6a)
        };

        // Polarity-aware translucent edges (legacy Slint-only tokens). On light
        // themes a white hairline is invisible, so flip the base to black.
        let (eh, eg, eb) = if is_light { (0, 0, 0) } else { (255, 255, 255) };
        let surface_hover = Rgba::rgba(eh, eg, eb, 0x10); // ~6%
        let border_muted = Rgba::rgba(eh, eg, eb, 0x38); // ~22%

        ThemeColors {
            surface_main: self.bg_primary,
            surface_card: self.bg_secondary,
            surface_elevated: self.bg_tertiary,
            surface_hover,
            bg_hover: self.bg_hover,

            text_primary: self.text_primary,
            text_secondary: self.text_secondary,
            text_muted: self.text_muted,
            text_disabled: self.text_disabled,

            accent: self.accent,
            accent_hover: self.accent_hover,
            accent_pressed: self.accent_pressed,
            accent_text: self.accent_text,

            danger: self.danger,
            danger_bg: with_alpha(self.danger, self.tint_bg),
            danger_border: with_alpha(self.danger, self.tint_border),
            danger_hover: with_alpha(self.danger, self.tint_hover),

            warning: self.warning,
            warning_bg: with_alpha(self.warning, self.tint_bg),
            warning_border: with_alpha(self.warning, self.tint_border),
            warning_hover: with_alpha(self.warning, self.tint_hover),

            success,
            success_bg: with_alpha(success, self.tint_bg),
            success_border: with_alpha(success, self.tint_border),
            success_hover: with_alpha(success, self.tint_hover),

            // Standard rows feed the theme `--border-subtle` hex (NOT the legacy
            // translucent hairline the 4 P1 rows kept).
            border_subtle: self.border_subtle,
            border_muted,
            border_strong: self.border_strong,

            focus_ring: self.accent, // = accent (no Tauri token; new)

            favorite: self.danger, // loved-heart uses danger red
            card_shadow: LEGACY_CARD_SHADOW,

            alpha: alpha_ramp(is_light),
        }
    }
}

/// True when a `bg-primary` reads as light (luminance >= 0.5). Drives polarity
/// for the standard rows. Matches `lib::is_light` (which calls through
/// `palette()`), but used internally to avoid a recursive `palette()` call.
pub(super) fn bg_is_light(bg_primary: Rgba) -> bool {
    relative_luminance(bg_primary) >= 0.5
}

/// Straight-alpha overlay of an opaque hue at `frac` opacity (0.0..=1.0).
/// Used to reproduce Tauri's `rgba(hue, frac)` danger/warning/success tints.
pub(super) const fn with_alpha(c: Rgba, frac: f32) -> Rgba {
    let a = (frac * 255.0 + 0.5) as u8;
    Rgba::rgba(c.r, c.g, c.b, a)
}
