use crate::*;

/// `on_appearance_select` arms: tray-icon-theme, wc-position,
/// app-background, startup-page, renderer, gpu-power, ui-scale. Called
/// unconditionally alongside `handle_appearance_select_b` from the single
/// `on_appearance_select` registration (wire_appearance_select.rs) — safe
/// since each key matches at most one of the two, the other falls through
/// its own `_ => {}`.
pub(crate) fn handle_appearance_select_a(
    key: &str,
    index: i32,
    theme_weak: &slint::Weak<AppWindow>,
) {
    match key {
            "tray-icon-theme" => {
                tray_settings::set_icon_theme_index(index);
                // Re-theme the running tray icon live (no restart).
                if let Some(t) = tray::handle() {
                    t.set_icon_theme(tray_settings::theme_for_index(index));
                }
            }
            "wc-position" => {
                // 0 = Left, 1 = Right. Live — HeaderBar re-anchors the drawn
                // controls from `wc-position-index`; persist only.
                let mut prefs = crate::ui_prefs::load();
                prefs.wc_position = if index == 0 { "left" } else { "right" }.to_string();
                crate::ui_prefs::save(&prefs);
            }
            "app-background" => {
                // 0 = Off, 1 = Ambient (GPU shader), 2 = Blurred art. The Slint
                // side already flipped app-background-mode-index; app-shader-mode
                // and app-background-active derive from it, and the AppShell
                // viz-should-run recompute starts/stops the drain reactively.
                let mut prefs = crate::ui_prefs::load();
                prefs.app_background =
                    crate::ui_prefs::app_background_for_index(index).to_string();
                crate::ui_prefs::save(&prefs);
            }
            "startup-page" => {
                // 0 = Home, 1 = Where you left off (restore last_view).
                let mut prefs = crate::ui_prefs::load();
                prefs.startup_page = crate::ui_prefs::startup_page_for_index(index).to_string();
                crate::ui_prefs::save(&prefs);
            }
            "renderer" => {
                // 0 = Auto, 1 = GPU (wgpu), 2 = GPU compatibility (femtovg GL),
                // 3 = Software. Startup-time choice — select_slint_backend()
                // reads it before the window exists, so it applies on the next
                // launch (a non-auto pick is protected by the auto-revert
                // sentinel there).
                let mut prefs = crate::ui_prefs::load();
                prefs.renderer = crate::ui_prefs::renderer_for_index(index).to_string();
                // A manual pick is the USER's choice — drop the auto-degrade
                // and alt-adapter markers so the ladder starts clean if they
                // ever return to "auto".
                prefs.renderer_auto_degraded.clear();
                prefs.renderer_wgpu_alt.clear();
                crate::ui_prefs::save(&prefs);
                crate::toast::info_weak(
                    &theme_weak,
                    qbz_i18n::t("Renderer changed — restart QBZ to apply"),
                );
            }
            "gpu-power" => {
                // 0 = Auto, i>0 = a specific detected GPU (by name). Applied at
                // startup by gpu_power_from_prefs (wgpu adapter power preference),
                // so it takes effect on the next launch.
                let mut prefs = crate::ui_prefs::load();
                prefs.gpu_power = gpu_power_value_for_index(index);
                crate::ui_prefs::save(&prefs);
                crate::toast::info_weak(
                    &theme_weak,
                    qbz_i18n::t("Preferred GPU changed — restart QBZ to apply"),
                );
            }
            "ui-scale" => {
                // 0 = Extra small, 1 = Small, 2 = Default, 3 = Large,
                // 4 = Extra large. Startup-time choice — SLINT_SCALE_FACTOR is
                // set at the very top of main() before the backend exists, so
                // it applies on the next launch.
                let mut prefs = crate::ui_prefs::load();
                prefs.ui_scale = crate::ui_prefs::ui_scale_for_index(index).to_string();
                crate::ui_prefs::save(&prefs);
                crate::toast::info_weak(
                    &theme_weak,
                    qbz_i18n::t("Interface size changed — restart QBZ to apply"),
                );
            }
        _ => {}
    }
}
