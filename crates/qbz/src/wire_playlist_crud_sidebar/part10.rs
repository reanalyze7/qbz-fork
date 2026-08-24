use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part10(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<CreatePlaylistActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<CreatePlaylistState>().set_open(false);
                }
            });
    }
}
