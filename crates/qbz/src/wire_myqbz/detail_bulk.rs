use crate::*;

use MyQbzDetailActions as Act;

/// Bulk action bar (select-mode, spec 12 §13.1): add-to-queue / play-next /
/// add-to-playlist / add-to-mixtape / remove-selected / clear.
pub(crate) fn wire_myqbz_detail_bulk(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        window.global::<Act>().on_bulk_action(move |id| {
            let Some(w) = weak.upgrade() else { return };
            match id.as_str() {
                "add-to-queue" | "play-next" => {
                    let selected = myqbz_detail::selected_full_items(&w);
                    if selected.is_empty() {
                        return;
                    }
                    myqbz_play::bulk_enqueue(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        selected,
                        id.as_str() == "play-next",
                    );
                }
                "add-to-playlist" => {
                    let selected = myqbz_detail::selected_full_items(&w);
                    if selected.is_empty() {
                        return;
                    }
                    // Resolve to Qobuz track ids on a worker, then open the
                    // global picker (Qobuz mode) + load the user's playlists.
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let ids =
                            myqbz_play::resolve_bulk_qobuz_track_ids(&runtime, &selected).await;
                        if ids.is_empty() {
                            crate::toast::error_weak(
                                &weak,
                                "No Qobuz tracks in the selection to add to a playlist",
                            );
                            return;
                        }
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::open_multi(&w, &ids, false);
                        });
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                "add-to-mixtape" => {
                    let selected = myqbz_detail::selected_full_items(&w);
                    let items: Vec<myqbz_add::AddItem> = selected
                        .iter()
                        .map(|it| myqbz_add::AddItem {
                            item_type: myqbz_detail::item_type_str(it.item_type).to_string(),
                            source: myqbz_detail::source_str(it.source).to_string(),
                            source_item_id: it.source_item_id.clone(),
                            title: it.title.clone(),
                            subtitle: it.subtitle.clone(),
                            artwork_url: it.artwork_url.clone(),
                            year: it.year,
                            track_count: it.track_count,
                        })
                        .collect();
                    open_add_to_mixtape(weak.clone(), handle.clone(), items);
                }
                "remove-selected" => {
                    let cid = w.global::<MyQbzDetailState>().get_id().to_string();
                    let positions = myqbz_detail::selected_positions(&w);
                    myqbz_edit::remove_selected(
                        weak.clone(),
                        handle.clone(),
                        image_cache.clone(),
                        cid,
                        positions,
                    );
                }
                "clear" => {
                    // Clear-X: uncheck every row + zero the count, staying in
                    // select-mode (spec §13.1 clear control).
                    myqbz_detail::clear_selection(&w);
                }
                other => {
                    log::warn!("[qbz-slint] myqbz_detail bulk-action: unknown id {other}");
                }
            }
        });
    }
}
