use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_link_and_import_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Appearance settings persistence. The toggles/selects set their
    // AppearanceState property locally, then fire these generic callbacks so
    // the choice survives restart. Tray keys persist to the shared per-user
    // tray_settings store; unknown keys are logged (other appearance settings
    // are wired as they land).
    {
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
                    qbz_i18n::t("Title bar mode changed — restart QBZ to apply"),
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
        let theme_weak = window.as_weak();
        let theme_handle = tokio::runtime::Handle::current();
        appearance.on_appearance_select(move |key, index| match key.as_str() {
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
            "language" => {
                // 0 = Auto, 1 = English, 2 = Español, 3 = Français, 4 = Deutsch,
                // 5 = Português, 6 = Русский, 7 = 日本語, 8 = Nederlands.
                // Persist the RAW user choice ("auto" stays "auto"), but resolve
                // "auto" to a concrete language before switching the live
                // translations.
                let chosen = crate::ui_prefs::language_for_index(index);
                let mut prefs = crate::ui_prefs::load();
                prefs.language = chosen.to_string();
                crate::ui_prefs::save(&prefs);
                let lang = if chosen == "auto" {
                    qbz_i18n::resolve_auto()
                } else {
                    chosen
                };
                qbz_i18n::set_language(lang);
                if let Err(e) = slint::select_bundled_translation(lang) {
                    log::warn!(
                        "[qbz-slint] select_bundled_translation('{lang}') failed: {e:?}"
                    );
                }
                // Reseed the non-reactive option arrays (they live as @tr
                // property DEFAULTS, which do NOT react to a translation
                // switch) so the dropdown contents update live.
                if let Some(w) = theme_weak.upgrade() {
                    reseed_i18n_labels(&w);
                }
            }
            "theme" => {
                // Slug is the source of truth. The appended "Auto (dynamic)"
                // entry (index == theme::auto_index) persists the "auto" slug and
                // generates the palette off-thread; every other index maps to a
                // stable registry id and hot-switches the static palette.
                let mut prefs = crate::ui_prefs::load();
                // The dropdown index is a position in the CURRENTLY filtered list
                // (All/Dark/Light), so map it through the same filter. Auto/Custom
                // only exist in the All view (filtered_*_index is -1 otherwise).
                let filter = prefs.theme_filter;
                if index == crate::theme::filtered_auto_index(filter) {
                    prefs.theme = crate::theme::AUTO_SLUG.to_string();
                    crate::ui_prefs::save(&prefs);
                    if let Some(w) = theme_weak.upgrade() {
                        let st = w.global::<AppearanceState>();
                        st.set_theme_is_auto(true);
                        st.set_theme_is_custom(false);
                        st.set_theme_is_system(false);
                    }
                    crate::auto_theme::regenerate(theme_weak.clone(), theme_handle.clone());
                } else if index == crate::theme::filtered_custom_index(filter) {
                    // "Custom": persist the slug, derive from the persisted (or
                    // freshly seeded) custom base, and apply live. The editor
                    // swatches are seeded from the same base.
                    prefs.theme = crate::theme::CUSTOM_SLUG.to_string();
                    crate::ui_prefs::save(&prefs);
                    if let Some(w) = theme_weak.upgrade() {
                        let st = w.global::<AppearanceState>();
                        st.set_theme_is_auto(false);
                        st.set_theme_is_custom(true);
                        st.set_theme_is_system(false);
                        if crate::custom_theme::exists() {
                            crate::custom_theme::seed_state(&w);
                            crate::custom_theme::apply_startup(&w);
                        } else {
                            // First-ever selection: seed from the palette the
                            // user is looking at RIGHT NOW (the previously
                            // applied theme), not from a hardcoded default —
                            // "customize what I see" is the whole point.
                            crate::custom_theme::seed_from_current(&w);
                        }
                    }
                } else {
                    let id = crate::theme::filtered_id_for_index(index, filter);
                    prefs.theme = id.slug().to_string();
                    crate::ui_prefs::save(&prefs);
                    if let Some(w) = theme_weak.upgrade() {
                        let st = w.global::<AppearanceState>();
                        st.set_theme_is_auto(false);
                        st.set_theme_is_custom(false);
                        st.set_theme_is_system(id == qbz_theme::ThemeId::System);
                        crate::theme::apply_theme(&w, id);
                    }
                }
            }
            "auto-theme-source" => {
                // Auto-theme source dropdown (System / Wallpaper / Custom Image):
                // persist the key and regenerate from the new source.
                crate::auto_theme::set_source(index, theme_weak.clone(), theme_handle.clone());
            }
            "theme-filter" => {
                // Theme-list filter cycle (0 = All, 1 = Dark, 2 = Light): persist
                // the choice, then rebuild the dropdown to show only matching
                // themes and re-derive which row (if any) is the active theme. The
                // applied palette is untouched — this only narrows the list.
                let mut prefs = crate::ui_prefs::load();
                prefs.theme_filter = index;
                crate::ui_prefs::save(&prefs);
                if let Some(w) = theme_weak.upgrade() {
                    let st = w.global::<AppearanceState>();
                    let labels: Vec<slint::SharedString> =
                        crate::theme::filtered_dropdown_labels(index)
                            .into_iter()
                            .map(slint::SharedString::from)
                            .collect();
                    st.set_themes(slint::ModelRc::new(slint::VecModel::from(labels)));
                    st.set_theme_index(crate::theme::filtered_selected_index_for_slug(
                        &prefs.theme,
                        index,
                    ));
                }
            }
            other => log::debug!("[qbz-slint] unhandled appearance-select '{other}'"),
        });

        // Auto-theme actions: image picker + explicit Regenerate button. Both run
        // generation off the event loop and push the palette back on it.
        let action_weak = window.as_weak();
        let action_handle = tokio::runtime::Handle::current();
        appearance.on_appearance_action(move |key| match key.as_str() {
            "auto-theme-select-image" => {
                crate::auto_theme::select_image(action_weak.clone(), action_handle.clone());
            }
            "auto-theme-regenerate" => {
                crate::auto_theme::regenerate(action_weak.clone(), action_handle.clone());
            }
            other => log::debug!("[qbz-slint] unhandled appearance-action '{other}'"),
        });

        // Custom-theme editor callbacks: per-token live edits (drag + hex),
        // polarity toggle, and "start from current theme". Each re-derives the
        // whole palette in Rust and pushes it live (derivation is cheap).
        let ct_weak = window.as_weak();
        appearance.on_custom_set_token(move |key, color| {
            if let Some(w) = ct_weak.upgrade() {
                crate::custom_theme::set_token(&w, key.as_str(), color);
            }
        });
        let ct_hex_weak = window.as_weak();
        appearance.on_custom_set_token_hex(move |key, hex| {
            if let Some(w) = ct_hex_weak.upgrade() {
                crate::custom_theme::set_token_hex(&w, key.as_str(), hex.as_str());
            }
        });
        let ct_dark_weak = window.as_weak();
        appearance.on_custom_toggle_dark(move |is_dark| {
            if let Some(w) = ct_dark_weak.upgrade() {
                crate::custom_theme::toggle_dark(&w, is_dark);
            }
        });
        let ct_seed_weak = window.as_weak();
        appearance.on_custom_seed_from_current(move || {
            if let Some(w) = ct_seed_weak.upgrade() {
                crate::custom_theme::seed_from_current(&w);
            }
        });
    }
}
