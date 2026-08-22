//! Reduce a fully-materialized [`ThemeColors`] back to editable base tokens —
//! the reverse of [`super::derive::theme_from_base`].

use crate::colors::ThemeColors;

use super::CustomThemeBase;

/// Reduce an existing fully-materialized [`ThemeColors`] to its editable base
/// tokens — the "Start from current theme" seed. Reads the tokens that
/// `theme_from_base` treats as inputs, so `theme_from_base(base_from_theme(c))`
/// reproduces every base token exactly for any theme this module authored.
///
/// `border` reads `border_subtle` when it is opaque (custom themes always are),
/// else falls back to the opaque `border_strong` — the four legacy P1 themes
/// store a translucent-white hairline in `border_subtle`, which would seed as a
/// jarring pure-white edge.
pub fn base_from_theme(colors: &ThemeColors, is_dark: bool) -> CustomThemeBase {
    let border = if colors.border_subtle.a == 255 {
        colors.border_subtle
    } else {
        colors.border_strong
    };
    CustomThemeBase {
        is_dark,
        surface_main: colors.surface_main.to_hex(),
        surface_card: colors.surface_card.to_hex(),
        surface_elevated: colors.surface_elevated.to_hex(),
        text_primary: colors.text_primary.to_hex(),
        text_secondary: colors.text_secondary.to_hex(),
        accent: colors.accent.to_hex(),
        danger: colors.danger.to_hex(),
        warning: colors.warning.to_hex(),
        success: colors.success.to_hex(),
        border: border.to_hex(),
        favorite: colors.favorite.to_hex(),
    }
}
