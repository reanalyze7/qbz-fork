use crate::*;

use MyQbzDetailActions as Act;

/// Per-row callbacks: PLAY (default), context-menu action, and single-item
/// REMOVE (routed through the same audited bulk remover as the bulk bar).
pub(crate) fn wire_myqbz_detail_rows(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_play_item(move |source_item_id| {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_play::play_item(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                id,
                source_item_id.to_string(),
            );
        });
    }

    // --- Per-row context menu (play / play-next / add-to-queue) ---------
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<Act>()
            .on_item_action(move |source_item_id, action| {
                let Some(w) = weak.upgrade() else { return };
                let id = w.global::<MyQbzDetailState>().get_id().to_string();
                if id.is_empty() {
                    return;
                }
                myqbz_play::item_action(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    source_item_id.to_string(),
                    action.to_string(),
                );
            });
    }

    // --- Per-row REMOVE (single item) -----------------------------------
    // Routes ONE position through the audited bulk remover (remove-highest-
    // first compaction + clear-selection + toast + reload) with a 1-element
    // vec, so single-row remove reuses the exact same code path as the bulk
    // "remove-selected" action — no duplicated removal logic.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_remove_item(move |position| {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_edit::remove_selected(
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                id,
                vec![position],
            );
        });
    }
}
