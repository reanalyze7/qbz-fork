// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for. (A real
// split, like wire_playlist_manager got, is recommended on a follow-up PR
// with compiler feedback available.)
use crate::*;

/// Wire the My QBZ collection-DETAIL view (Phase-2 Slice 3, read-only). The
/// toolbar callbacks (search / sort / type-filter / source-filter / view-mode /
/// select / reset) drive `crate::myqbz_detail` re-derives + re-issue row
/// artwork; `open-item` / `open-artist` route to the existing
/// album/playlist/artist navigators (reusing the top-level open-album /
/// open-artist callbacks so local-vs-qobuz routing + history stay in one
/// place). Every hero CTA + per-row context action is a logging STUB — the
/// read-only boundary for this slice.
pub(crate) fn wire_myqbz_detail(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    use MyQbzDetailActions as Act;

    // Stash the runtime for the mutation-reload paths (cover/edit) that re-run
    // `myqbz_detail::navigate` (whose resolveItems pass needs it) without
    // threading it through every entry point.
    myqbz_detail::set_runtime(app_runtime.clone());
    // Blacklist Manager album-cover resolution needs the shared image cache.
    blacklist_manager::set_image_cache(image_cache.clone());

    // After a toolbar re-derive the rendered model changed, so the visible
    // rows need their thumbnails reloaded — through the SOURCE-SPLIT dispatch
    // (Qobuz CDN urls via HTTP; local paths via the source-aware decoder).
    fn refresh_row_covers(window: &AppWindow, image_cache: &artwork::ImageCache) {
        let split = myqbz_detail::artwork_jobs(window);
        myqbz_detail::dispatch_artwork(split, window.as_weak(), image_cache.clone());
    }

    // A toolbar re-derive rebuilds the rendered model with fresh rows
    // (tracks_loaded reset to false). While in expanded view-mode the new
    // visible rows must (re-)fetch their inline tracks (spec §8 auto-fetch).
    fn ensure_expanded_if_active(
        window: &AppWindow,
        runtime: &Arc<AppRuntime<SlintAdapter>>,
        handle: &tokio::runtime::Handle,
    ) {
        if window.global::<MyQbzDetailState>().get_view_mode() == "expanded" {
            myqbz_detail::ensure_expanded(runtime.clone(), window.as_weak(), handle.clone());
        }
    }

    // --- Toolbar (client-side re-derive) --------------------------------
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_search_changed(move |q| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::search(&w, q.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_set_sort(move |field| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::set_sort(&w, field.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_set_type_filter(move |value| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::set_type_filter(&w, value.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_toggle_source_filter(move |kind| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::toggle_source_filter(&w, kind.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_set_view_mode(move |mode| {
            if let Some(w) = weak.upgrade() {
                // Sets view-mode + persists the per-collection prefs (spec §18).
                myqbz_detail::set_view_mode(&w, mode.as_str());
                // Entering expanded mode: fetch every expandable item's tracks
                // (spec §8 — tracks render directly under each row).
                if mode == "expanded" {
                    myqbz_detail::ensure_expanded(runtime.clone(), weak.clone(), handle.clone());
                }
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<Act>().on_toggle_select_mode(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::toggle_select_mode(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<Act>().on_toggle_item_select(move |position| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::toggle_item_select(&w, position);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_reset_filters(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::reset_filters(&w);
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }

    // --- Open an item -> album / local-album / playlist -----------------
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<Act>()
            .on_open_item(move |_source, item_type, source_item_id| {
                let Some(w) = weak.upgrade() else { return };
                let id = source_item_id.to_string();
                match item_type.as_str() {
                    // Album / track items both open an album view; the top-level
                    // open-album callback handles Qobuz-vs-local routing + history.
                    "album" | "track" => {
                        w.invoke_open_album(id.into());
                    }
                    "playlist" => {
                        nav::record(nav::NavEntry::Playlist(id.clone()));
                        navigate_playlist(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    other => {
                        log::warn!("[qbz-slint] myqbz_detail open-item: unknown type {other}");
                    }
                }
            });
    }

    // --- Open an item's artist (route by SOURCE) -------------------------
    {
        let weak = window.as_weak();
        window
            .global::<Act>()
            .on_open_artist(move |source, artist_name, artist_id| {
                let Some(w) = weak.upgrade() else { return };
                // The top-level open-artist callback routes a numeric id to
                // the Qobuz artist page (with nav history — the same path
                // AlbumView's artist button uses) and a name to the
                // LocalLibrary Artists tab. Stored items only carry the
                // artist NAME, so Qobuz rows route by the numeric artist id
                // the resolveItems pass derived from their first track.
                if source == "qobuz" {
                    if !artist_id.trim().is_empty() {
                        w.invoke_open_artist(artist_id);
                    } else {
                        // Resolve still pending (or failed) — do NOT fall
                        // back to the name: that opens the WRONG page (the
                        // LocalLibrary artist) for a Qobuz item.
                        log::warn!(
                            "[qbz-slint] myqbz_detail open-artist: qobuz item '{artist_name}' \
                             has no resolved artist id yet — ignoring click"
                        );
                    }
                } else if !artist_name.trim().is_empty() {
                    // local -> the LocalLibrary Artists tab by NAME.
                    w.invoke_open_artist(artist_name);
                }
            });
    }

    // --- Hero PLAY / SHUFFLE (Slice 5: detail playback) -----------------
    // Resolve the collection's items through the qbz-mixtape ENQUEUE resolver
    // and drive the queue (replace + auto-play). DJ-mix / edit / delete / sync
    // stay logging stubs (later slices).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_play_all(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_play::play_all(runtime.clone(), weak.clone(), handle.clone(), id);
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_shuffle(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_play::shuffle(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                id,
            );
        });
    }

    // --- Hero DJ-mix CTA — open the "Random queue" sampler modal --------
    // Resolves the collection in-order + counts unique tracks (the slider max),
    // then the modal samples + replace-plays on confirm (myqbz_mix).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_dj_mix(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_mix::open(runtime.clone(), weak.clone(), handle.clone(), id);
        });
    }

    // --- STILL-STUBBED hero CTA: discography sync -----------------------
    // Sync: artist_discography has NO sync impl (spec §8) — no-op stub (the
    // hero button is shown only for artist_collection for Tauri parity).
    {
        let weak = window.as_weak();
        window.global::<Act>().on_sync(move || {
            let id = weak
                .upgrade()
                .map(|w| w.global::<MyQbzDetailState>().get_id().to_string())
                .unwrap_or_default();
            log::info!("[qbz-slint] myqbz_detail sync({id}) — no discography sync impl (spec §8)");
        });
    }

    // --- DJ-mix modal actions (slider / cancel / confirm) ---------------
    {
        let weak = window.as_weak();
        window.global::<MyQbzMixActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_mix::close(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzMixActions>().on_set_index(move |index| {
            if let Some(w) = weak.upgrade() {
                myqbz_mix::apply_index(&w, index);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<MyQbzMixActions>().on_shuffle(move || {
            let Some(w) = weak.upgrade() else { return };
            let ms = w.global::<MyQbzMixState>();
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            let size = ms.get_selected_size();
            if id.is_empty() || size <= 0 {
                return;
            }
            myqbz_mix::shuffle(runtime.clone(), weak.clone(), handle.clone(), id, size);
        });
    }

    // --- Bulk action bar (select-mode, spec 12 §13.1) ------------------
    // The full §13.1 group set:
    //  - "add-to-queue" / "play-next": resolve the selected items via the shared
    //    enqueue resolver + append / insert-next (no replace, no queue-source
    //    stamp — mirrors the per-row contract).
    //  - "add-to-playlist": resolve the selected items to their Qobuz track ids
    //    and open the existing playlist picker (Qobuz mode) with them.
    //  - "add-to-mixtape": open the global AddToMixtapeModal with the payloads.
    //  - "remove-selected": remove each selected position (highest-first) then
    //    reload the detail + clear selection.
    //  - "clear": clear the selection (exit-select / uncheck all).
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

    // --- Hero overflow (⋯) menu — open the edit modals (spec 12 §10/§11) ---
    // Rename / Edit description / Delete-confirm open the shared MyQbzEditModal
    // with the right mode + prefill; the mutations + reload run on submit.
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

    // --- Edit modals — submit (rename / description / delete) ----------
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

    // --- Per-row PLAY (default) -----------------------------------------
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

    // --- Expanded view-mode: inline tracks under every album/playlist (§8) -
    // Fired when the expanded view-mode becomes active; fetches each
    // expandable item's tracks (skipping already-cached rows).
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

