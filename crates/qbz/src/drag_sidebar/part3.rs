use crate::*;

/// Reseed the AppearanceState dropdown OPTION arrays from the Rust-side i18n
/// catalog (`qbz_i18n::t`). These arrays are declared in `ui/state.slint` as
/// `@tr(...)` PROPERTY DEFAULTS, which are evaluated once and do NOT react to a
/// runtime `select_bundled_translation()` switch — so without this reseed the
/// option contents stay in the language that was active at first paint.
/// `QbzSelect` binds `options` live, so writing the arrays here updates the
/// rendered dropdowns immediately.
///
/// Call this (a) at startup right after `select_bundled_translation` (post
/// `AppWindow::new()`) and (b) in the "language" appearance-select arm after the
/// translation switch. Only the string arrays are reseeded; the `*_index`
/// selection properties are intentionally left untouched. Brand names and
/// language endonyms (macOS, Adwaita, English, Español, …) stay as literals.
pub(crate) fn reseed_i18n_labels(window: &AppWindow) {
    use slint::{ModelRc, SharedString, VecModel};
    let t = |s: &str| SharedString::from(qbz_i18n::t(s));
    let state = window.global::<AppearanceState>();

    state.set_auto_theme_sources(ModelRc::new(VecModel::from(vec![
        t("System Colors"),
        t("Wallpaper Sync"),
        t("Custom Image"),
    ])));
    state.set_languages(ModelRc::new(VecModel::from(vec![
        t("Auto"),
        "English".into(),
        "Español".into(),
        "Français".into(),
        "Deutsch".into(),
        "Português".into(),
        "Русский".into(),
        "日本語".into(),
        "Nederlands".into(),
    ])));
    state.set_app_background_modes(ModelRc::new(VecModel::from(vec![
        t("Off"),
        t("Ambient"),
        t("Blurred art"),
    ])));
    // Preferred-GPU labels ("Auto…" + the (discrete)/(integrated) tags) are
    // translated; the device names are not. Rebuild from the cached adapters.
    state.set_gpu_power_modes(ModelRc::new(VecModel::from(gpu_power_options())));
    state.set_wc_positions(ModelRc::new(VecModel::from(vec![t("Left"), t("Right")])));
    state.set_wc_styles(ModelRc::new(VecModel::from(vec![
        t("Rectangular"),
        t("Full-height rounded"),
        t("Circular"),
        t("Square"),
    ])));
    state.set_wc_sizes(ModelRc::new(VecModel::from(vec![
        t("Small"),
        t("Normal"),
        t("Large"),
    ])));
    state.set_wc_color_presets(ModelRc::new(VecModel::from(vec![
        t("Default"),
        "macOS".into(),
        "Adwaita".into(),
        t("Monochrome"),
        t("Custom"),
    ])));
    state.set_startup_pages(ModelRc::new(VecModel::from(vec![
        t("Home"),
        t("Where you left off"),
    ])));
    state.set_tray_icon_themes(ModelRc::new(VecModel::from(vec![
        t("Auto"),
        t("Mono light"),
        t("Mono dark"),
        t("Color"),
    ])));
}

