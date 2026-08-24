use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part10(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, _tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
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
