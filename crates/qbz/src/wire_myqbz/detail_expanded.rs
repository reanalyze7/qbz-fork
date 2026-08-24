use crate::*;

use MyQbzDetailActions as Act;

/// Expanded view-mode (§8): fetch every expandable item's inline tracks, and
/// wire their row actions (play / play-next / play-later / go-to-album).
pub(crate) fn wire_myqbz_detail_expanded(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = image_cache;
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_ensure_expanded(move || {
            myqbz_detail::ensure_expanded(runtime.clone(), weak.clone(), handle.clone());
        });
    }
    // Inline-track row actions (play / play-next / play-later / go-to-album).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<Act>()
            .on_inline_track_action(move |item_source_item_id, track_id, action| {
                let Some(w) = weak.upgrade() else { return };
                // go-to-album routes through the existing open-item path (Qobuz
                // album view vs local-album by id), keeping nav in one place.
                // It must open the PARENT item (spec 12 §8) — so route with the
                // parent's REAL item_type (album/playlist), not a hardcoded
                // "album": a playlist parent must reach the playlist view, not
                // be mis-routed to the album view. The parent's type is read off
                // the rendered row carrying this source-item-id.
                if action == "go-to-album" {
                    let parent_type = {
                        let model = w.global::<MyQbzDetailState>().get_items();
                        (0..model.row_count())
                            .filter_map(|i| model.row_data(i))
                            .find(|it| it.source_item_id == item_source_item_id)
                            .map(|it| it.item_type.to_string())
                            .unwrap_or_else(|| "album".to_string())
                    };
                    w.global::<Act>().invoke_open_item(
                        "".into(),
                        parent_type.into(),
                        item_source_item_id,
                    );
                    return;
                }
                let id = w.global::<MyQbzDetailState>().get_id().to_string();
                if id.is_empty() {
                    return;
                }
                myqbz_play::play_inline_track(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    item_source_item_id.to_string(),
                    track_id.to_string(),
                    action.to_string(),
                );
            });
    }
}
