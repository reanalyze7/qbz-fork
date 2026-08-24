use crate::*;

/// `on_appearance_select` arms: language, theme, auto-theme-source,
/// theme-filter. Called unconditionally alongside
/// `handle_appearance_select_a` from the single `on_appearance_select`
/// registration (wire_appearance_select.rs).
pub(crate) fn handle_appearance_select_b(
    key: &str,
    index: i32,
    theme_weak: &slint::Weak<AppWindow>,
    theme_handle: &tokio::runtime::Handle,
) {
    match key {
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

    }
}
