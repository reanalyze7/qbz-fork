use crate::*;

/// Appearance settings persistence: `on_appearance_bool`. The toggles set
/// their AppearanceState property locally, then fire this generic callback
/// so the choice survives restart. Tray keys persist to the shared per-user
/// tray_settings store; unknown keys are logged (other appearance settings
/// are wired as they land).
pub(crate) fn wire_appearance_bool(window: &AppWindow) {
    let appearance = window.global::<AppearanceState>();
    let chrome_weak = window.as_weak();
    appearance.on_appearance_bool(move |key, value| match key.as_str() {
            "use-system-title-bar" => {
                // The toggle only edits the PREF (`use-system-title-bar`);
                // whether it reaches the applied chrome state
                // (`system-title-bar-active`, which no-frame / the header
                // drag+inset read) is decided HERE, per platform.
                let mut prefs = crate::ui_prefs::load();
                prefs.use_system_title_bar = value;
                crate::ui_prefs::save(&prefs);
                // Linux: mirror live (today's hot path — no-frame drives
                // winit set_decorations on the next properties update, and
                // the drawn controls/drag follow).
                #[cfg(not(target_os = "macos"))]
                if let Some(w) = chrome_weak.upgrade() {
                    w.global::<AppearanceState>()
                        .set_system_title_bar_active(value);
                }
                // macOS: persist-only, restart to apply. The overlay
                // attributes (titlebar_transparent / fullsize_content_view)
                // are fixed at window creation; flipping the header bindings
                // live desyncs them from the real chrome (traffic lights
                // overlapped by content, or system bar + overlay inset at
                // once), so `system-title-bar-active` keeps its startup
                // value there.
                crate::toast::info_weak(
                    &chrome_weak,
                    qbz_i18n::t("Title bar mode changed — restart Qoqobuz to apply"),
                );
            }
            "hide-title-bar" => {
                // Live for the controls/drag (bindings read the flag); the
                // frameless state itself already follows use-system-title-bar.
                let mut prefs = crate::ui_prefs::load();
                prefs.hide_title_bar = value;
                crate::ui_prefs::save(&prefs);
            }
            "show-window-controls" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.show_window_controls = value;
                crate::ui_prefs::save(&prefs);
            }
            "album-header-gradient" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.album_header_gradient = value;
                crate::ui_prefs::save(&prefs);
            }
            "intelligent-search" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.intelligent_search = value;
                crate::ui_prefs::save(&prefs);
                // Propagate the toggle to the bound SearchService kill switch
                // (no-op if no session is bound; the next session init re-seeds
                // the flag from the persisted pref above).
                crate::search_service::set_enabled(value);
            }
            "system-notifications" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.system_notifications = value;
                crate::ui_prefs::save(&prefs);
                // Live gate for the poll-thread notify path (no restart).
                playback::NOTIFICATIONS_ENABLED
                    .store(value, std::sync::atomic::Ordering::Relaxed);
            }
            "tray-enable" => tray_settings::set_enable_tray(value),
            "tray-minimize-to-tray" => tray_settings::set_minimize_to_tray(value),
            "tray-close-to-tray" => tray_settings::set_close_to_tray(value),
            "tray-mac-hide-dock" => tray_settings::set_mac_hide_dock(value),
            "window-title-show" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.window_title_show = value;
                crate::ui_prefs::save(&prefs);
            }
            "show-volume-steppers" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.show_volume_steppers = value;
                crate::ui_prefs::save(&prefs);
            }
            "sidebar-playlist-collage" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.sidebar_playlist_collage = value;
                crate::ui_prefs::save(&prefs);
            }
            "local-library-track-artwork" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.local_library_track_artwork = value;
                crate::ui_prefs::save(&prefs);
            }
            "in-app-toasts" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.in_app_toasts = value;
                crate::ui_prefs::save(&prefs);
            }
            other => log::debug!("[qbz-slint] unhandled appearance-bool '{other}'"),
    });
}
