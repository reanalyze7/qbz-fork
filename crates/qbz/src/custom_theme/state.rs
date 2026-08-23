//! Read/write of the editor swatch properties on `AppearanceState`.

use slint::ComponentHandle;

use crate::AppWindow;
use crate::AppearanceState;
use qbz_theme::CustomThemeBase;
use slint::Color;

use super::convert::{hex_to_color, rgba_of};

/// Read the current editable base straight from the editor swatch properties on
/// `AppearanceState` — the in-memory source of truth while the editor is open, so
/// per-drag edits never hit the disk to reconstruct the base.
pub(super) fn base_from_state(window: &AppWindow) -> CustomThemeBase {
    let st = window.global::<AppearanceState>();
    CustomThemeBase {
        is_dark: st.get_custom_is_dark(),
        surface_main: rgba_of(st.get_custom_surface_main()).to_hex(),
        surface_card: rgba_of(st.get_custom_surface_card()).to_hex(),
        surface_elevated: rgba_of(st.get_custom_surface_elevated()).to_hex(),
        text_primary: rgba_of(st.get_custom_text_primary()).to_hex(),
        text_secondary: rgba_of(st.get_custom_text_secondary()).to_hex(),
        accent: rgba_of(st.get_custom_accent()).to_hex(),
        danger: rgba_of(st.get_custom_danger()).to_hex(),
        warning: rgba_of(st.get_custom_warning()).to_hex(),
        success: rgba_of(st.get_custom_success()).to_hex(),
        border: rgba_of(st.get_custom_border()).to_hex(),
        favorite: rgba_of(st.get_custom_favorite()).to_hex(),
    }
}

/// Assign one base-token field by its stable key (the `token-key` strings the
/// Slint editor rows use). Unknown keys are ignored.
pub(super) fn set_field(base: &mut CustomThemeBase, key: &str, hex: String) {
    match key {
        "surface-main" => base.surface_main = hex,
        "surface-card" => base.surface_card = hex,
        "surface-elevated" => base.surface_elevated = hex,
        "text-primary" => base.text_primary = hex,
        "text-secondary" => base.text_secondary = hex,
        "accent" => base.accent = hex,
        "danger" => base.danger = hex,
        "warning" => base.warning = hex,
        "success" => base.success = hex,
        "border" => base.border = hex,
        "favorite" => base.favorite = hex,
        other => log::debug!("[qbz-slint] custom theme: unknown token key '{other}'"),
    }
}

/// Reflect a single edited token back into its editor swatch (so the swatch
/// preview and the ColorPicker's `value` binding update), WITHOUT touching the
/// open-token state (the inline picker must stay open through the edit).
pub(super) fn set_one_swatch(window: &AppWindow, key: &str, color: Color) {
    let st = window.global::<AppearanceState>();
    match key {
        "surface-main" => st.set_custom_surface_main(color),
        "surface-card" => st.set_custom_surface_card(color),
        "surface-elevated" => st.set_custom_surface_elevated(color),
        "text-primary" => st.set_custom_text_primary(color),
        "text-secondary" => st.set_custom_text_secondary(color),
        "accent" => st.set_custom_accent(color),
        "danger" => st.set_custom_danger(color),
        "warning" => st.set_custom_warning(color),
        "success" => st.set_custom_success(color),
        "border" => st.set_custom_border(color),
        "favorite" => st.set_custom_favorite(color),
        _ => {}
    }
}

/// Push a [`CustomThemeBase`] into the editor swatch properties on
/// `AppearanceState` so the swatches reflect the current base. Collapses any open
/// inline picker.
pub(super) fn push_base_to_state(window: &AppWindow, base: &CustomThemeBase) {
    let st = window.global::<AppearanceState>();
    st.set_custom_surface_main(hex_to_color(&base.surface_main));
    st.set_custom_surface_card(hex_to_color(&base.surface_card));
    st.set_custom_surface_elevated(hex_to_color(&base.surface_elevated));
    st.set_custom_text_primary(hex_to_color(&base.text_primary));
    st.set_custom_text_secondary(hex_to_color(&base.text_secondary));
    st.set_custom_accent(hex_to_color(&base.accent));
    st.set_custom_danger(hex_to_color(&base.danger));
    st.set_custom_warning(hex_to_color(&base.warning));
    st.set_custom_success(hex_to_color(&base.success));
    st.set_custom_border(hex_to_color(&base.border));
    st.set_custom_favorite(hex_to_color(&base.favorite));
    st.set_custom_is_dark(base.is_dark);
    st.set_custom_open_token("".into());
}
