use crate::*;

pub(crate) fn wire_search_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // "Hi-Res only" toggle: pure client-side re-filter of the already-loaded
    // albums/tracks — no re-fetch. The `bool` arg mirrors LabelActions'
    // on_set_hires (state is already flipped by the ToggleButton itself
    // before this fires; see the toolbar in SearchResultsView.slint).
    // Qobuz's search endpoints take no quality parameter
    // (search::recompute_hi_res_filtered has the full rationale), so unlike
    // on_filter_changed above this never spawns a network task.
    {
        let weak = window.as_weak();
        window.global::<SearchActions>().on_hires_only_changed(move |_| {
            if let Some(w) = weak.upgrade() {
                search::recompute_hi_res_filtered(&w);
            }
        });
    }

    // Cortinilla: dismiss (click-outside / Escape).
    {
        let weak = window.as_weak();
        window.global::<SearchActions>().on_cortinilla_dismiss(move || {
            if let Some(w) = weak.upgrade() {
                let st = w.global::<SearchState>();
                st.set_cortinilla_open(false);
                // Clear the keyboard/hover highlight too — a dismissed dropdown
                // has no meaningful selection, and the `changed view` close-hook
                // (AppShell) relies on this to reset the highlight on navigation.
                st.set_selected_index(-1);
            }
        });
    }
}
