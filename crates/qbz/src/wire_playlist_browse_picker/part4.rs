use crate::*;

pub(crate) fn wire_playlist_browse_picker_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistPickerActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<PlaylistPickerState>();
                    st.set_open(false);
                    // Reset the inline-create + filter affordances so the next
                    // open starts clean.
                    st.set_creating_open(false);
                    st.set_create_name("".into());
                    st.set_creating(false);
                    st.set_filter("".into());
                }
            });
    }
}
