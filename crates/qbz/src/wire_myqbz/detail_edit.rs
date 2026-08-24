use crate::*;

/// Edit modal submit (rename / description / delete / close).
pub(crate) fn wire_myqbz_detail_edit(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = app_runtime;
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<MyQbzEditActions>().on_submit_rename(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            let name = w.global::<MyQbzEditState>().get_draft_name().to_string();
            myqbz_edit::rename(weak.clone(), handle.clone(), image_cache.clone(), id, name);
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<MyQbzEditActions>()
            .on_submit_description(move || {
                let Some(w) = weak.upgrade() else { return };
                let id = w.global::<MyQbzDetailState>().get_id().to_string();
                let desc = w.global::<MyQbzEditState>().get_draft_description().to_string();
                myqbz_edit::set_description(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    id,
                    desc,
                );
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<MyQbzEditActions>().on_confirm_delete(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            myqbz_edit::delete(weak.clone(), handle.clone(), id);
        });
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzEditActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                let es = w.global::<MyQbzEditState>();
                es.set_open(false);
                es.set_mode("".into());
                es.set_busy(false);
            }
        });
    }
}
