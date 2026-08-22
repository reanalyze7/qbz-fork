//! Assemble a full [`ThemeColors`] from an extracted palette or a DE color
//! scheme.
//!
//! Logic is a 1:1 port of the Tauri `auto_theme::generator` (`generate_theme` /
//! `generate_theme_from_scheme`): same text-contrast enforcement, same accent
//! triplet, same status hues, same border shifts. The DIFFERENCE is the output
//! type — instead of writing `rgba()`/hex strings into a CSS-var map we populate
//! the registry [`ThemeColors`] struct, deriving the tokens the CSS map never
//! had (`success`, `focus_ring`, `favorite`, `border_muted`, the polarity alpha
//! ramp) exactly as `registry::StdSpec::build` does so a generated theme
//! composites identically to a static one.
//!
//! Tokens with no `ThemeColors` field are intentionally dropped: the Tauri
//! generator also emitted `--btn-danger-text` / `--btn-warning-text`, but the
//! frontend-agnostic contract has no per-status text field (only `accent_text`),
//! so those are not carried over.

mod assemble;
mod contrast;
mod from_palette;
mod from_scheme;
#[cfg(test)]
mod tests;

pub(crate) use assemble::tint;
pub(crate) use contrast::{ensure_text_contrast_target, pick_btn_text_for_accent_set};
pub use from_palette::theme_from_palette;
pub use from_scheme::theme_from_scheme;
