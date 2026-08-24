use crate::*;

pub(crate) fn wire_discover_offline_manager_part7(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, _tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Case-insensitive substring test backing the searchable QbzSelect
    // (Slint 1.16 has no `contains` builtin). Pure + stateless, so a single
    // registration at setup serves every searchable list.
    window
        .global::<TextUtil>()
        .on_contains_ci(|haystack: slint::SharedString, needle: slint::SharedString| {
            haystack
                .to_lowercase()
                .contains(needle.to_lowercase().as_str())
        });

    // Genre filter — selection is per context ("discover" / "favorites").
    // Toggling / clearing re-fetches the discover index (discover context)
    // or re-derives the favorites tab (favorites context).
    {
        let weak = window.as_weak();
        window
            .global::<GenreFilterActions>()
            .on_set_context(move |ctx| {
                genre_filter::set_context(ctx.as_str());
                if let Some(w) = weak.upgrade() {
                    genre_filter::apply_state(&w);
                }
            });
    }
}
