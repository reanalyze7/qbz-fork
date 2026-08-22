//! The fully-materialized per-theme color set.
//!
//! Every theme row in the registry returns one of these with EVERY field
//! populated — there is no CSS cascade on the Slint side, so omissions in
//! `src/app.css` are resolved against `:root` (Dark) at registry-build time.

mod alpha;
#[cfg(test)]
mod tests;

pub use alpha::{alpha_byte, alpha_index, alpha_ramp, ALPHA_COUNT, ALPHA_PERCENTS};

use crate::color::Rgba;

/// The complete, frontend-agnostic color contract for one theme. Field order
/// groups by family (surfaces, text, accent, danger, warning, success, borders,
/// focus, extras, alpha) to match the Slint `ThemeColors` struct and the plan's
/// A.3 token list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    // --- surfaces ---
    pub surface_main: Rgba,
    pub surface_card: Rgba,
    pub surface_elevated: Rgba,
    /// Alpha-based hover overlay (translucent, polarity-correct).
    pub surface_hover: Rgba,
    /// Opaque theme `--bg-hover` hex (distinct from the alpha `surface_hover`).
    pub bg_hover: Rgba,

    // --- text ---
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub text_disabled: Rgba,

    // --- accent ---
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_pressed: Rgba,
    /// Text drawn ON an accent fill (`--btn-primary-text`).
    pub accent_text: Rgba,

    // --- danger family ---
    pub danger: Rgba,
    pub danger_bg: Rgba,
    pub danger_border: Rgba,
    pub danger_hover: Rgba,

    // --- warning family ---
    pub warning: Rgba,
    pub warning_bg: Rgba,
    pub warning_border: Rgba,
    pub warning_hover: Rgba,

    // --- success family (NEW vs Tauri parity) ---
    pub success: Rgba,
    pub success_bg: Rgba,
    pub success_border: Rgba,
    pub success_hover: Rgba,

    // --- borders ---
    /// Theme `--border-subtle` value. The legacy Slint `border-subtle` alias was
    /// a translucent white hairline; for the P1 themes this keeps that value so
    /// the 4 themes stay pixel-identical. Standard/a11y rows (P2/P3) feed the
    /// theme `--border-subtle` hex.
    pub border_subtle: Rgba,
    /// Legacy Slint-only token (no Tauri equivalent): a stronger translucent
    /// edge used by popovers/dropdowns. Kept so existing call sites compile.
    pub border_muted: Rgba,
    pub border_strong: Rgba,

    // --- focus (NEW) ---
    pub focus_ring: Rgba,

    // --- extras ---
    pub favorite: Rgba,
    pub card_shadow: Rgba,

    // --- alpha overlays (polarity baked in: white on dark, black on light) ---
    pub alpha: [Rgba; ALPHA_COUNT],
}

impl ThemeColors {
    /// Look up an alpha overlay by percentage (e.g. `8`, `55`). Falls back to
    /// the nearest standard tier if an exact match is absent.
    pub fn alpha_pct(&self, pct: u8) -> Rgba {
        if let Some(i) = alpha_index(pct) {
            return self.alpha[i];
        }
        // Nearest tier by absolute distance.
        let mut best = 0usize;
        let mut best_d = u8::MAX as i32;
        for (i, &p) in ALPHA_PERCENTS.iter().enumerate() {
            let d = (p as i32 - pct as i32).abs();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        self.alpha[best]
    }
}
