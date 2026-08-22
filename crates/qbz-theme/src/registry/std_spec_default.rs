//! `Default` for [`StdSpec`], split out of `std_spec.rs` for the 130-line
//! budget.

use crate::color::Rgba;

use super::std_spec::StdSpec;

impl Default for StdSpec {
    /// All-black placeholder; every field is overwritten per theme. The default
    /// only exists so theme functions can use struct-update syntax for the tint
    /// fractions without repeating them.
    fn default() -> Self {
        let z = Rgba::rgb(0, 0, 0);
        StdSpec {
            bg_primary: z,
            bg_secondary: z,
            bg_tertiary: z,
            bg_hover: z,
            text_primary: z,
            text_secondary: z,
            text_muted: z,
            text_disabled: z,
            accent: z,
            accent_hover: z,
            accent_pressed: z,
            accent_text: z,
            danger: z,
            warning: z,
            tint_bg: StdSpec::TINT_BG,
            tint_border: StdSpec::TINT_BORDER,
            tint_hover: StdSpec::TINT_HOVER,
            border_subtle: z,
            border_strong: z,
        }
    }
}
