//! Editor entry points: per-token edits, polarity toggle, and
//! "start from current theme".

use slint::{Color, ComponentHandle};

use qbz_theme::{CustomThemeBase, Rgba};

use crate::AppWindow;
use crate::AppearanceState;
use crate::Theme as SlintTheme;

use super::apply::apply_live;
use super::convert::{rgba_of, to_color};
use super::persist::save;
use super::state::{base_from_state, push_base_to_state, set_field, set_one_swatch};

/// Set one base token to `color` (the live ColorPicker drag path). Re-derives and
/// re-pushes the whole palette in real time, persists, and updates the token's
/// own swatch — the inline picker stays open.
pub fn set_token(window: &AppWindow, key: &str, color: Color) {
    let mut base = base_from_state(window);
    set_field(&mut base, key, rgba_of(color).to_hex());
    apply_live(window, &base);
    set_one_swatch(window, key, color);
}

/// Set one base token from a committed HEX string (`#rrggbb`). Malformed input is
/// ignored; a valid value reuses [`set_token`] (which also updates the swatch,
/// reseeding the picker's crosshair via its `value` binding).
pub fn set_token_hex(window: &AppWindow, key: &str, hex: &str) {
    match Rgba::from_hex(hex) {
        Some(c) => set_token(window, key, to_color(Rgba::rgb(c.r, c.g, c.b))),
        None => log::debug!("[qbz-slint] custom theme: ignoring malformed hex '{hex}'"),
    }
}

/// Flip the custom theme polarity (dark/light) and re-derive. The base token
/// colors are unchanged; only `is_dark` and the derived shades/overlays flip.
pub fn toggle_dark(window: &AppWindow, is_dark: bool) {
    let mut base = base_from_state(window);
    base.is_dark = is_dark;
    apply_live(window, &base);
    window.global::<AppearanceState>().set_custom_is_dark(is_dark);
}

/// "Start from current theme": snapshot the LIVE applied palette (whatever is in
/// the `Theme` global — static, auto or custom) into the editable base, then
/// derive/apply/persist and re-seed every editor swatch. `is_dark` is inferred
/// from the surface luminance; `border` prefers the opaque subtle edge, else the
/// strong one (the four legacy P1 themes store a translucent-white hairline in
/// `border_subtle` that would seed as a jarring pure-white edge).
pub fn seed_from_current(window: &AppWindow) {
    let c = window.global::<SlintTheme>().get_c();
    let surface_main = rgba_of(c.surface_main);
    let is_dark = qbz_theme::relative_luminance(Rgba::rgb(
        surface_main.r,
        surface_main.g,
        surface_main.b,
    )) < 0.5;
    let border_subtle = rgba_of(c.border_subtle);
    let border = if border_subtle.a == 255 {
        border_subtle
    } else {
        rgba_of(c.border_strong)
    };
    let base = CustomThemeBase {
        is_dark,
        surface_main: surface_main.to_hex(),
        surface_card: rgba_of(c.surface_card).to_hex(),
        surface_elevated: rgba_of(c.surface_elevated).to_hex(),
        text_primary: rgba_of(c.text_primary).to_hex(),
        text_secondary: rgba_of(c.text_secondary).to_hex(),
        accent: rgba_of(c.accent).to_hex(),
        danger: rgba_of(c.danger).to_hex(),
        warning: rgba_of(c.warning).to_hex(),
        success: rgba_of(c.success).to_hex(),
        border: border.to_hex(),
        favorite: rgba_of(c.favorite).to_hex(),
    };
    let colors = qbz_theme::theme_from_base(&base);
    crate::theme::push_colors(window, &colors, false, false);
    save(&base);
    push_base_to_state(window, &base);
}
