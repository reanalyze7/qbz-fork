use crate::*;

/// Seed the tray settings UI from the persisted per-user store.
pub(crate) fn seed_tray_appearance(w: &AppWindow, tray: &tray_settings::TraySettings) {
    let appearance = w.global::<AppearanceState>();
    appearance.set_tray_enable(tray.enable_tray);
    appearance.set_tray_minimize_to_tray(tray.minimize_to_tray);
    appearance.set_tray_close_to_tray(tray.close_to_tray);
    appearance.set_tray_mac_hide_dock(tray.mac_hide_dock);
    appearance.set_tray_icon_theme_index(tray_settings::icon_theme_index(&tray.tray_icon_theme));
    // Renderer row (Linux-only: on macOS the renderer is always Skia, so the
    // row stays hidden). Piggybacks this appearance-seed choke point so the
    // dropdown always reflects the persisted value when Settings opens.
    appearance.set_renderer_setting_visible(cfg!(target_os = "linux"));
    appearance.set_renderer_index(crate::ui_prefs::renderer_index(
        &crate::ui_prefs::load().renderer,
    ));
    // Preferred GPU row (Linux only, shares the renderer-row visibility) —
    // options built from the real detected GPUs; index resolved from the
    // persisted device name / legacy key.
    appearance
        .set_gpu_power_modes(slint::ModelRc::new(slint::VecModel::from(gpu_power_options())));
    appearance.set_gpu_power_index(gpu_power_seed_index(
        &crate::ui_prefs::load().gpu_power,
    ));
    // Interface-size row (all platforms). Same choke point as the renderer
    // row so the dropdown always reflects the persisted value.
    appearance.set_ui_scale_index(crate::ui_prefs::ui_scale_index(
        &crate::ui_prefs::load().ui_scale,
    ));
}

/// Refresh the blacklist count + enabled flag on `BlacklistState` (T10).
/// The Settings content-filtering row binds to these, and Settings is reached
/// independently of the Manager (which seeds them on its own load), so we
/// re-read the wrapper whenever the Settings view is shown. Fail-open: with no
/// session the wrapper returns count 0 / enabled true.
pub(crate) fn seed_blacklist_status(w: &AppWindow) {
    let st = w.global::<BlacklistState>();
    st.set_count(crate::artist_blacklist::count() as i32);
    st.set_album_count(crate::artist_blacklist::album_count() as i32);
    st.set_enabled(crate::artist_blacklist::is_enabled());
}

