use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part1(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, _tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window.global::<DragActions>().on_move(move |gx, gy| {
            if let Some(w) = weak.upgrade() {
                let ds = w.global::<DragState>();
                ds.set_pointer_x(gx);
                ds.set_pointer_y(gy);
            }
        });
    }
}
