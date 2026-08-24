use crate::*;

use MyQbzDetailActions as Act;

/// Hero overflow (⋯) menu: open the rename / description / delete-confirm
/// edit modals, custom cover set/remove, and play-mode toggle / convert kind.
pub(crate) fn wire_myqbz_detail_overflow(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = app_runtime;
    {
        let weak = window.as_weak();
        window.global::<Act>().on_open_rename(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<MyQbzDetailState>();
            let es = w.global::<MyQbzEditState>();
            es.set_mode("rename".into());
            es.set_name(ds.get_name());
            es.set_draft_name(ds.get_name());
            es.set_busy(false);
            es.set_open(true);
        });
    }
    {
        let weak = window.as_weak();
        window.global::<Act>().on_open_description(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<MyQbzDetailState>();
            let es = w.global::<MyQbzEditState>();
            es.set_mode("description".into());
            es.set_name(ds.get_name());
            es.set_draft_description(ds.get_description());
            es.set_busy(false);
            es.set_open(true);
        });
    }
    {
        let weak = window.as_weak();
        window.global::<Act>().on_open_delete(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<MyQbzDetailState>();
            let es = w.global::<MyQbzEditState>();
            es.set_mode("delete".into());
            es.set_name(ds.get_name());
            es.set_busy(false);
            es.set_open(true);
        });
    }

    // --- Hero overflow — custom cover (set / remove) --------------------
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_upload_cover(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_cover::upload(weak.clone(), handle.clone(), image_cache.clone(), id);
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_remove_cover(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_cover::remove(weak.clone(), handle.clone(), image_cache.clone(), id);
        });
    }

    // --- Hero overflow — play-mode toggle / convert kind ---------------
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_toggle_play_mode(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<MyQbzDetailState>();
            let id = ds.get_id().to_string();
            let mode = ds.get_play_mode().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_edit::toggle_play_mode(weak.clone(), handle.clone(), image_cache.clone(), id, mode);
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_convert_kind(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<MyQbzDetailState>();
            let id = ds.get_id().to_string();
            let kind = ds.get_kind().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_edit::convert_kind(weak.clone(), handle.clone(), image_cache.clone(), id, kind);
        });
    }
}
