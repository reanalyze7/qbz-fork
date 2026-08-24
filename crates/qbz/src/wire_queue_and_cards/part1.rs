use crate::*;

pub(crate) fn wire_queue_and_cards_part1(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, _tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Album track search — client-side filter, no backend round-trip.
    {
        let weak = window.as_weak();
        window
            .global::<AlbumActions>()
            .on_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    album::filter_tracks(&w, query.as_str());
                }
            });
    }

    // Album multi-select: the toolbar toggle next to the search box.
    {
        let weak = window.as_weak();
        window
            .global::<AlbumActions>()
            .on_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<AlbumState>().get_multi_select();
                    album::set_multi_select(&w, !on);
                }
            });
    }
}
