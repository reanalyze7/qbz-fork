use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_home_library_playback_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Context-menu / overlay media actions — route play / queue actions
    // into the playback controller; favorite / download stay logged.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_media_action(move |kind, id, action| {
            let kind = kind.to_string();
            let id = id.to_string();
            let action = action.to_string();
            log::info!("[qbz-slint] media-action: kind={kind} id={id} action={action}");
            // Local Library album detail reuses AlbumPageView. Route its play
            // actions to local playback — guarded to the album view + is-local
            // so Qobuz album/track play is untouched.
            if action == "play" && (kind == "album" || kind == "track") {
                if let Some(w) = weak.upgrade() {
                    let album_state = w.global::<AlbumState>();
                    if matches!(w.global::<NavState>().get_view(), ContentView::Album)
                        && album_state.get_is_local()
                    {
                        let album_id = album_state.get_id().to_string();
                        let start = if kind == "track" {
                            id.parse::<i64>().ok()
                        } else {
                            None
                        };
                        playback::play_local_album(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            album_id,
                            start,
                        );
                        return;
                    }
                }
            }

            // === Capa B feedback (intelligent search) ====================
            // Feed the ranking model from RESULTS-PAGE clicks, gated to the
            // Search view inside `record_search_interaction` so the same global
            // media-action handler fired from other views never mis-attributes.
            // Only QOBUZ result clicks are recorded; the search results page
            // never carries local rows (D1/D2), so no source check is needed.
            //   - track play              -> Play
            //   - album play              -> Play (an album-card play is still a
            //                                play interaction with the entity)
            //   - album favorite (toggle) -> Favorite ONLY when transitioning to
            //                                favorited (the card heart arm is a
            //                                toggle since 2026-07; Favorite
            //                                weight must only ADD)
            //   - artist follow (add)     -> Favorite (search artist cards show
            //                                "Follow" only when NOT following, so
            //                                this action is always an add)
            //   - track favorite (toggle) -> Favorite ONLY when transitioning to
            //                                favorited (Favorite weight must only
            //                                ADD — never record on un-favorite)
            if let Some(w) = weak.upgrade() {
                use crate::search_service::InteractionAction;
                match (kind.as_str(), action.as_str()) {
                    ("track", "play") | ("album", "play") => {
                        record_search_interaction(&w, &kind, &id, InteractionAction::Play);
                    }
                    ("album", "favorite") => {
                        // Toggle: record ONLY when this click ADDS the favorite
                        // (mirrors the track arm below; the album card arm flips
                        // off the same `fav_cache::is_album_favorite`).
                        if !crate::fav_cache::is_album_favorite(&id) {
                            record_search_interaction(&w, &kind, &id, InteractionAction::Favorite);
                        }
                    }
                    ("artist", "follow") => {
                        // Add-only on a search card ("Follow" shows only when
                        // NOT following).
                        record_search_interaction(&w, &kind, &id, InteractionAction::Favorite);
                    }
                    ("track", "favorite") => {
                        // Toggle: record ONLY when this click ADDS the favorite
                        // (the current cached state is "not favorite"). Reading
                        // the pre-toggle state here matches `toggle_track_favorite`,
                        // which flips off the same `fav_cache::is_favorite`.
                        if !crate::fav_cache::is_favorite(&id) {
                            record_search_interaction(&w, &kind, &id, InteractionAction::Favorite);
                        }
                    }
                    _ => {}
                }
            }

            match (kind.as_str(), action.as_str()) {
                // Large dock: visualizer on/off toggle (the cover's eye button).
                // Routed through Rust so the choice persists in ui_prefs; the
                // AppShell viz-should-run handler idles the FFT tap when off.
                ("npb-large", "viz-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let shell = w.global::<ShellState>();
                        let on = !shell.get_large_visualizer_on();
                        shell.set_large_visualizer_on(on);
                        let mut prefs = crate::ui_prefs::load();
                        prefs.large_visualizer = on;
                        crate::ui_prefs::save(&prefs);
                    }
                }
                // Large dock: cycle the spectrum visualization (Bars -> Waveform
                // -> Energy), persisted in ui_prefs.
                ("npb-large", "spectrum-cycle") => {
                    if let Some(w) = weak.upgrade() {
                        let shell = w.global::<ShellState>();
                        let next = (shell.get_large_spectrum_mode() + 1).rem_euclid(3);
                        shell.set_large_spectrum_mode(next);
                        let mut prefs = crate::ui_prefs::load();
                        prefs.large_spectrum_mode =
                            crate::ui_prefs::large_spectrum_mode_key(next).to_string();
                        crate::ui_prefs::save(&prefs);
                    }
                }
                // Track Info modal — opened from the NPB (i) button, the
                // song-card title, or a TrackRow context menu. Qobuz tracks
                // only (the id must be a real catalog u64).
                ("track", "track-info") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        info_modals::open_track_info(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                // "Reveal in file explorer" — local tracks only (the row's
                // id is a library row id, not a catalog id; TrackContextMenu
                // gates the menu entry itself on source == "local").
                // Try the in-memory Tracks-tab cache first (no DB hit);
                // folder-detail rows that aren't in it fall back to an
                // off-thread DB resolve, mirroring the play-next/queue arm
                // just above this match's local block.
                ("track", "reveal-in-explorer") => {
                    if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                        reveal_in_file_manager(&row.file_path);
                    } else if let Ok(rid) = id.parse::<i64>() {
                        handle.spawn(async move {
                            let row = tokio::task::spawn_blocking(move || {
                                crate::library_db::with_db(|db| db.get_track(rid)).flatten()
                            })
                            .await
                            .ok()
                            .flatten();
                            if let Some(row) = row {
                                reveal_in_file_manager(&row.file_path);
                            }
                        });
                    }
                }
                // Album Info (Credits/Review) modal — opened from the album
                // header (i) button. Qobuz albums only (skip local keys).
                ("album", "info") => {
                    if !is_local_album_key(&id) {
                        info_modals::open_album_credits(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
                // Album booklet (digital liner-notes PDF) — the album-header
                // booklet button DOWNLOADS the goody PDF (stashed by
                // album::apply_album) to a user-chosen location. No-op when the
                // album bundles no booklet (empty stashed URL).
                ("album", "booklet") => {
                    crate::booklet::download_booklet(weak.clone(), handle.clone());
                }
                // "From the same artist" carousel "View all" — open the artist's
                // full Albums discography page. `id` is the artist id; reuse the
                // dedicated releases page (release_type "album").
                ("artist", "releases") => {
                    if !id.is_empty() {
                        let name = weak
                            .upgrade()
                            .map(|w| w.global::<AlbumState>().get_artist().to_string())
                            .unwrap_or_default();
                        nav::record(nav::NavEntry::ArtistReleases {
                            id: id.clone(),
                            name: name.clone(),
                            release_type: "album".to_string(),
                        });
                        navigate_artist_releases(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id.clone(),
                            name,
                            "album".to_string(),
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    }
                }
                ("album", "play") => {
                    // A local id is a metadata group key, not a Qobuz id —
                    // play it from the local cache (Home "Recently played",
                    // etc.) instead of trying to fetch a Qobuz album.
                    if is_local_album_key(&id) {
                        playback::play_local_album(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            None,
                        );
                    } else {
                        playback::play_album(runtime.clone(), weak.clone(), handle.clone(), id, 0);
                    }
                }
                ("track", "play") => {
                    // Universal per-row play: queue the current view's VISIBLE
                    // tracklist starting at the clicked track (see
                    // playback::play_track_in_context). Every tracklist surface
                    // routes here — album, playlist, favorites, label, mix,
                    // artist, search.
                    if let Some(w) = weak.upgrade() {
                        playback::play_track_in_context(
                            &w,
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            &id,
                        );
                    }
                }
                ("album", "queue") => playback::enqueue_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("track", "queue") => {
                    // SOURCE-TYPED routing first (spec §3.2, mirrors the
                    // add-to-playlist arm): on a snapshot-backed playlist
                    // detail a local row's id is a library row id — the
                    // catalog path below would mis-resolve it (wrong-track
                    // hazard / silent failure). The merged snapshot carries
                    // the ready, source-aware QueueTrack; enqueue it directly.
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    playback::enqueue_queue_tracks(
                                        runtime.clone(),
                                        weak.clone(),
                                        handle.clone(),
                                        vec![qt],
                                        false,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    // Qobuz rows (incl. offline copies with real catalog
                    // ids): the existing path — single-track
                    // admission + fresh fetch.
                    if let Ok(track_id) = id.parse::<u64>() {
                        playback::enqueue_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("album", "play-next") => playback::enqueue_album_next(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "shuffle") => playback::play_album_shuffled(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "edit") => {
                    // Open the local-album tag editor (group_key == directory_path
                    // for folder-grouped local albums).
                    tag_editor::open_tag_editor(weak.clone(), handle.clone(), id.clone(), id);
                }
                ("album", "add-to-mixtape") => {
                    // The cassette button on the album header. Local albums
                    // build the payload
                    // from AlbumState + the loaded tracks; Qobuz albums resolve
                    // via get_album (the proven fail-safe resolver).
                    let Some(w) = weak.upgrade() else { return };
                    let st = w.global::<AlbumState>();
                    if st.get_is_local() {
                        let item = myqbz_add::AddItem {
                            item_type: "album".into(),
                            source: "local".into(),
                            source_item_id: st.get_id().to_string(),
                            title: st.get_title().to_string(),
                            subtitle: {
                                let a = st.get_artist().to_string();
                                (!a.is_empty()).then_some(a)
                            },
                            artwork_url: None, // local albums omit artwork_url (1:1 PSD)
                            year: None,
                            track_count: {
                                use slint::Model;
                                let n = st.get_tracks().row_count();
                                (n > 0).then_some(n as i32)
                            },
                        };
                        open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
                    } else {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle2 = handle.clone();
                        let album_id = id.clone();
                        handle.spawn(async move {
                            let item = match runtime.core().get_album(&album_id).await {
                                Ok(album) => {
                                    let artwork_url = album
                                        .image
                                        .thumbnail
                                        .clone()
                                        .or_else(|| album.image.small.clone());
                                    let year = album
                                        .release_date_original
                                        .as_deref()
                                        .and_then(|d| d.get(0..4))
                                        .and_then(|y| y.parse::<i32>().ok());
                                    let track_count = album
                                        .tracks_count
                                        .or(album.track_count)
                                        .map(|n| n as i32);
                                    myqbz_add::AddItem {
                                        item_type: "album".into(),
                                        source: "qobuz".into(),
                                        source_item_id: album.id.clone(),
                                        title: album.title.clone(),
                                        subtitle: {
                                            let a = album.artist.name.clone();
                                            (!a.is_empty()).then_some(a)
                                        },
                                        artwork_url,
                                        year,
                                        track_count,
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "[qbz-slint] add-to-mixtape: get_album {album_id} failed: {e}"
                                    );
                                    return;
                                }
                            };
                            open_add_to_mixtape(weak, handle2, vec![item]);
                        });
                    }
                }
                ("album", "favorite") => {
                    // Album-card heart + "…" menu entry: a TRUE TOGGLE keyed
                    // off the favorite-album cache (filled heart → remove,
                    // empty → add), mirroring the header "favorite-toggle"
                    // arm below. Was add-only while the cards couldn't show
                    // favorite state; now that they do, re-adding from a
                    // filled heart would lie. Optimistic: flip the heart on
                    // every visible card right away (mirrors the track
                    // rows); rolled back on failure. NOTE: the Favorites
                    // albums tab never reaches this arm — FavoritesView
                    // intercepts "favorite" to unfavorite-album (fade-out +
                    // row removal).
                    let was_fav = crate::fav_cache::is_album_favorite(&id);
                    let new_state = !was_fav;
                    if let Some(w) = weak.upgrade() {
                        set_album_row_favorite(&w, &id, new_state);
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let album_id = id.clone();
                    handle.spawn(async move {
                        let res = if new_state {
                            runtime.core().add_favorite("album", &album_id).await
                        } else {
                            runtime.core().remove_favorite("album", &album_id).await
                        };
                        match res {
                            Ok(()) => {
                                // Keep the favorite-album cache in sync so the
                                // album-header heart reflects a card toggle.
                                crate::fav_cache::set_album(&album_id, new_state);
                                crate::toast::success_weak(
                                    &weak,
                                    if new_state {
                                        "Added to favorites"
                                    } else {
                                        "Removed from favorites"
                                    },
                                );
                                // reco: log the album favorite ADD on success
                                // only — Capa B scores adds, never removals.
                                if new_state {
                                    let aid = album_id.clone();
                                    tokio::task::spawn_blocking(move || {
                                        crate::reco::log_favorite_album(aid, None)
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "[qbz-slint] toggle favorite album {album_id} failed: {e}"
                                );
                                crate::toast::error_weak(&weak, "Couldn't update favorites");
                                // Roll the optimistic hearts back to the
                                // pre-click state.
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    set_album_row_favorite(&w, &album_id, was_fav);
                                });
                            }
                        }
                    });
                }
                ("album", "favorite-toggle") => {
                    // The album-header heart: a TRUE toggle that reflects the
                    // favorite-album cache (the card "favorite" arm above is
                    // the same toggle, minus the AlbumState header sync).
                    // Optimistic on the open header, reconciled on the server
                    // result.
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    let was_fav = crate::fav_cache::is_album_favorite(&id);
                    let new_state = !was_fav;
                    let st = w.global::<AlbumState>();
                    let is_open = st.get_id() == id.as_str();
                    if is_open {
                        st.set_is_favorite(new_state);
                        st.set_favorite_loading(true);
                    }
                    // Optimistic on every visible album card too (artist
                    // discography, carousels, search, favorites) — reconciled
                    // with the server result below, like the header heart.
                    set_album_row_favorite(&w, &id, new_state);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let album_id = id.clone();
                    handle.spawn(async move {
                        let res = if new_state {
                            runtime.core().add_favorite("album", &album_id).await
                        } else {
                            runtime.core().remove_favorite("album", &album_id).await
                        };
                        let ok = res.is_ok();
                        if let Err(e) = &res {
                            log::error!(
                                "[qbz-slint] toggle favorite album {album_id} failed: {e}"
                            );
                        }
                        // reco: log the album favorite ADD on success (skip the
                        // un-favorite). Blocking SQLite off the async path.
                        if ok && new_state {
                            let aid = album_id.clone();
                            tokio::task::spawn_blocking(move || {
                                crate::reco::log_favorite_album(aid, None)
                            });
                        }
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let st = w.global::<AlbumState>();
                            let open_now = st.get_id() == album_id.as_str();
                            if ok {
                                crate::fav_cache::set_album(&album_id, new_state);
                                if open_now {
                                    st.set_favorite_loading(false);
                                    st.set_is_favorite(new_state);
                                }
                                crate::toast::success(
                                    &w,
                                    if new_state {
                                        "Added to favorites"
                                    } else {
                                        "Removed from favorites"
                                    },
                                );
                            } else {
                                if open_now {
                                    st.set_favorite_loading(false);
                                    st.set_is_favorite(was_fav);
                                }
                                // Roll the optimistic card hearts back too.
                                set_album_row_favorite(&w, &album_id, was_fav);
                                crate::toast::error(&w, "Couldn't update favorites");
                            }
                        });
                    });
                }
                ("album", "cache") => offline_cache::cache_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "recache") => offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    // Refresh the WHOLE album (Tauri's "Refresh offline copy"
                    // re-downloads every track, not only the failed ones).
                    false,
                ),
                ("album", "add-to-playlist") => {
                    // Resolve the album's loaded tracks to their Qobuz catalog
                    // ids and open the playlist picker for the whole set
                    // (mirrors Tauri's album → Add to playlist). Local
                    // albums carry no catalog ids, so the entry no-ops there
                    // (the header menu is a Qobuz surface).
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    let ids: Vec<String> = {
                        use slint::Model;
                        w.global::<AlbumState>()
                            .get_tracks()
                            .iter()
                            .map(|t| t.id.to_string())
                            .filter(|s| s.parse::<u64>().is_ok())
                            .collect()
                    };
                    if ids.is_empty() {
                        toast::error(&w, "No tracks to add");
                        return;
                    }
                    playlist_picker::open_multi(&w, &ids, false);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                ("album", "share-qobuz") => {
                    share::copy_to_clipboard(share::qobuz_album_url(&id));
                    log::info!("[qbz-slint] copied Qobuz link for album {id}");
                }
                ("album", "share-songlink") => {
                    // Tauri-parity resolution (#514): fetch the album to get
                    // its UPC, then UPC -> Deezer -> album.link. The old
                    // URL-only Odesli call never worked for Qobuz input
                    // (could_not_resolve_entity) — see share.rs.
                    let album = id.clone();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    crate::toast::info_weak(&weak, qbz_i18n::t("Fetching Album.link..."));
                    handle.spawn(async move {
                        let upc = runtime
                            .core()
                            .get_album(&album)
                            .await
                            .ok()
                            .and_then(|a| a.upc);
                        match share::albumlink_for_album(&album, upc.as_deref()).await {
                            Some(url) => {
                                share::copy_to_clipboard(url);
                                log::info!("[qbz-slint] copied Album.link for album {album}");
                                crate::toast::success_weak(&weak, qbz_i18n::t("Link copied"));
                            }
                            None => {
                                log::warn!("[qbz-slint] Album.link resolution failed for {album}");
                                crate::toast::error_weak(
                                    &weak,
                                    qbz_i18n::t("Failed to copy link"),
                                );
                            }
                        }
                    });
                }
                ("track", "play-next") => {
                    // Source-typed routing — see the ("track","queue") arm
                    // (same seam, insert-next instead of append).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    playback::enqueue_queue_tracks(
                                        runtime.clone(),
                                        weak.clone(),
                                        handle.clone(),
                                        vec![qt],
                                        true,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    if let Ok(track_id) = id.parse::<u64>() {
                        playback::play_track_next(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "favorite") => {
                    // Offline guard + optimistic toggle + network flip with
                    // rollback — shared with the library-surface favorite
                    // (see toggle_track_favorite).
                    toggle_track_favorite(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.to_string(),
                    );
                }
                // Offline cache: "download"/"cache" make a track available
                // offline; "uncache" removes the local copy. The row affordance
                // and the context menu both bubble these.
                ("track", "cache") | ("track", "download") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::cache_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "uncache") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::remove_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "recache") => {
                    // "Refresh offline copy" (cached-state menu entry, spec
                    // §3.5): remove + re-download, sequenced.
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::refresh_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "remove-from-playlist") => {
                    // Per-row removal on the playlist detail (spec §3.1).
                    // Ownership-gated in the UI (PlaylistState.is-owner —
                    // DELIBERATE: Tauri's available branch renders it
                    // un-gated on followed playlists where the owner-only
                    // API rejects, §1.6.1; we port the intent, not the
                    // hole). One-row ride on the same namespace-split seam
                    // as the bulk removal; the reload re-merges the sidecar.
                    let Some(w) = weak.upgrade() else { return };
                    if w.global::<NavState>().get_view() != ContentView::Playlist {
                        return;
                    }
                    if w.global::<PlaylistState>().get_is_local() {
                        local_playlist::remove_rows_by_ids(
                            &w,
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            vec![id.to_string()],
                        );
                        return;
                    }
                    let pid = w.global::<PlaylistState>().get_id().to_string();
                    let Some(row) = playlist::row_for_id(&id) else {
                        log::warn!("[qbz-slint] remove-from-playlist: row {id} not loaded");
                        return;
                    };
                    if let Ok(pid) = pid.parse::<u64>() {
                        playlist_remove_rows(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            pid,
                            vec![row],
                        );
                    }
                }
                // External-reco Weekly rows (P7): the title-adjacent buttons.
                // `id` carries the section key ("weekly-exploration"/"weekly-jams").
                ("ext-reco-list", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = external_reco::list_track_ids(&w, &id);
                        playback::enqueue_track_ids(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            ids,
                            false,
                        );
                    }
                }
                ("ext-reco-list", "create-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = external_reco::list_track_ids(&w, &id);
                        if !ids.is_empty() {
                            let ids_str: Vec<String> =
                                ids.iter().map(|i| i.to_string()).collect();
                            playlist_picker::open_for_ids(
                                &w,
                                runtime.clone(),
                                &handle,
                                ids_str,
                                false,
                            );
                        }
                    }
                }
                ("track", "add-to-playlist") => {
                    // Open the global picker for this track + load the
                    // user's playlists. SOURCE-TYPED routing first: this
                    // shared arm also fires for local rows (local
                    // playlist detail, now-playing), whose ids are NOT
                    // Qobuz catalog ids. Type the ref, or refuse.
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    // Only consult the local-playlist queue snapshot while
                    // its detail is the OPEN view — a stale snapshot row id
                    // could collide with a genuine catalog id from a Qobuz
                    // surface (both are small integers). The ONLINE mixed
                    // Qobuz detail shares the snapshot (E11), so its
                    // local rows type their refs the same way.
                    let in_local_detail = snapshot_detail_open(&w);
                    let local_ref: Option<String> = if in_local_detail {
                        // Open local-playlist detail row: the queue snapshot
                        // knows its source ("<row id>"; None for Qobuz rows
                        // = catalog flow below).
                        local_playlist::local_picker_ref_for_row(id.as_str())
                    } else {
                        None
                    };
                    if let Some(track_ref) = local_ref {
                        playlist_picker::open_multi(&w, &[track_ref], true);
                    } else if id
                        .parse::<u64>()
                        .is_ok_and(|n| n >= local_library::LEGACY_SYNTHETIC_ID_FLOOR)
                    {
                        // A synthetic (ephemeral) id with no resolvable
                        // ref — refuse rather than store a fake Qobuz id.
                        log::warn!(
                            "[qbz-slint] add-to-playlist: unresolvable non-catalog id {id} — refused"
                        );
                        toast::error(&w, "Couldn't resolve this track");
                        return;
                    } else {
                        playlist_picker::open(&w, &id);
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                ("track", "add-to-mixtape") => {
                    // The menu only carries the track id; resolve the Qobuz
                    // track (this entry is gated to Qobuz/offline in the menu)
                    // to build the AddToMixtape payload, then open the picker.
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle2 = handle.clone();
                        handle.spawn(async move {
                            let item = match runtime.core().get_track(track_id).await {
                                Ok(track) => {
                                    let artist = track
                                        .performer
                                        .as_ref()
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default();
                                    let album = track
                                        .album
                                        .as_ref()
                                        .map(|a| a.title.clone())
                                        .unwrap_or_default();
                                    let subtitle = [artist, album]
                                        .into_iter()
                                        .filter(|s| !s.is_empty())
                                        .collect::<Vec<_>>()
                                        .join(" · ");
                                    let artwork_url = track.album.as_ref().and_then(|a| {
                                        a.image
                                            .thumbnail
                                            .clone()
                                            .or_else(|| a.image.small.clone())
                                    });
                                    myqbz_add::AddItem {
                                        item_type: "track".into(),
                                        source: "qobuz".into(),
                                        source_item_id: track_id.to_string(),
                                        title: track.title.clone(),
                                        subtitle: (!subtitle.is_empty()).then_some(subtitle),
                                        artwork_url,
                                        year: None,
                                        track_count: None,
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "[qbz-slint] add-to-mixtape: get_track {track_id} failed: {e}"
                                    );
                                    return;
                                }
                            };
                            open_add_to_mixtape(weak, handle2, vec![item]);
                        });
                    }
                }
                ("track", "share-qobuz") => {
                    share::copy_to_clipboard(share::qobuz_track_url(&id));
                    log::info!("[qbz-slint] copied Qobuz link for track {id}");
                }
                ("track", "share-songlink") => {
                    // Tauri-parity resolution (#514): fetch the track to get
                    // its ISRC, then ISRC -> Deezer -> song.link. The old
                    // URL-only Odesli call never worked for Qobuz input
                    // (could_not_resolve_entity) — see share.rs.
                    let track = id.clone();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    crate::toast::info_weak(&weak, qbz_i18n::t("Fetching Song.link..."));
                    handle.spawn(async move {
                        let isrc = match track.parse::<u64>() {
                            Ok(tid) => runtime
                                .core()
                                .get_track(tid)
                                .await
                                .ok()
                                .and_then(|t| t.isrc),
                            Err(_) => None,
                        };
                        match share::songlink_for_track(&track, isrc.as_deref()).await {
                            Some(url) => {
                                share::copy_to_clipboard(url);
                                log::info!("[qbz-slint] copied Song.link for track {track}");
                                crate::toast::success_weak(&weak, qbz_i18n::t("Link copied"));
                            }
                            None => {
                                log::warn!("[qbz-slint] Song.link resolution failed for {track}");
                                crate::toast::error_weak(
                                    &weak,
                                    qbz_i18n::t("Failed to copy link"),
                                );
                            }
                        }
                    });
                }
                ("track", "go-to-album") => {
                    // Playlist-detail local sidecar rows first (owner
                    // improvement — Tauri omits the entries there): their
                    // snapshot ids are library row ids, NOT catalog ids, and
                    // the snapshot QueueTrack's album_id already carries the
                    // LOCAL navigation key (the same one the now-playing bar
                    // navigates by — group key). Qobuz + offline-copy rows fall
                    // through to the catalog resolve below (an offline copy's
                    // row id IS its Qobuz id).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    match qt.album_id.filter(|k| !k.is_empty()) {
                                        Some(key) => w.invoke_open_album(key.into()),
                                        None => log::debug!(
                                            "[qbz-slint] go-to-album: playlist row {id} has no album key"
                                        ),
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    // The menu only carries the track id — resolve the
                    // track to find its album, then open it.
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            if let Ok(track) = runtime.core().get_track(track_id).await {
                                if let Some(album_id) =
                                    track.album.as_ref().map(|a| a.id.clone())
                                {
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        w.invoke_open_album(album_id.into());
                                    });
                                }
                            }
                        });
                    }
                }
                ("track", "go-to-artist") => {
                    // Same local diversion as go-to-album: local
                    // artists have no id, so route by NAME to the LocalLibrary
                    // Artists tab (the open-artist callback's split).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    if qt.artist.trim().is_empty() {
                                        log::debug!(
                                            "[qbz-slint] go-to-artist: playlist row {id} has no artist name"
                                        );
                                    } else {
                                        w.invoke_open_artist(qt.artist.into());
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            if let Ok(track) = runtime.core().get_track(track_id).await {
                                if let Some(artist_id) =
                                    track.performer.as_ref().map(|p| p.id)
                                {
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        w.invoke_open_artist(artist_id.to_string().into());
                                    });
                                }
                            }
                        });
                    }
                }
                // Clickable artist name (album cards) -> artist page.
                ("artist", "open") => {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_open_artist(id.clone().into());
                    }
                }
                // Clickable album name (track rows) -> album page.
                ("album", "open") => {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_open_album(id.clone().into());
                    }
                }
                // Now-playing context (song-card layers button) -> playlist page.
                ("playlist", "open") => {
                    nav::record(nav::NavEntry::Playlist(id.clone()));
                    navigate_playlist(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        id.clone(),
                    );
                }
                // Blacklist / Show toggle from the ArtistView overflow
                // menu (and the hidden-artist banner). Resolves the id
                // from the passed value, falling back to ArtistState.id
                // Reads the name from
                // ArtistState for storage. Optimistic with rollback: flip
                // ArtistState.is-blacklisted immediately, perform the
                // mutation, revert + error-toast on failure. Synchronous
                // on the event-loop thread, so there is no re-entrancy
                // window (a second click can't interleave mid-toggle).
                ("artist", "share") => {
                    let artist_id = if id.is_empty() {
                        weak.upgrade()
                            .map(|w| w.global::<ArtistState>().get_id().to_string())
                            .unwrap_or_default()
                    } else {
                        id.clone()
                    };
                    if !artist_id.is_empty() {
                        share::copy_to_clipboard(share::qobuz_artist_url(&artist_id));
                        if let Some(w) = weak.upgrade() {
                            crate::toast::success(&w, qbz_i18n::t("Link copied"));
                        }
                    }
                }
                ("artist", "blacklist-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let st = w.global::<ArtistState>();
                        let artist_id = if id.is_empty() {
                            st.get_id().to_string()
                        } else {
                            id.clone()
                        };
                        let name = st.get_name().to_string();
                        if let Ok(id_num) = artist_id.parse::<u64>() {
                            let was_blacklisted =
                                crate::artist_blacklist::is_blacklisted(id_num);
                            // Optimistic flip.
                            st.set_is_blacklisted(!was_blacklisted);
                            let res = if was_blacklisted {
                                crate::artist_blacklist::remove(id_num)
                            } else {
                                crate::artist_blacklist::add(
                                    id_num,
                                    &name,
                                    None,
                                )
                            };
                            match res {
                                Ok(()) => {
                                    // Live refresh for the artist page is the
                                    // optimistic ArtistState.is-blacklisted
                                    // flip above (drives the banner + the
                                    // menu Show/Blacklist label). ArtistView
                                    // popular-tracks rows are deliberately
                                    // NOT per-row greyed (T6 scoping — the
                                    // banner is the artist-page surface);
                                    // other open views (search, album,
                                    // favorites) re-stamp on next navigation
                                    // (no global observer).
                                    let msg = if was_blacklisted {
                                        format!("{name} is now visible")
                                    } else {
                                        format!("{name} is now hidden")
                                    };
                                    crate::toast::success_weak(&weak, msg);
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] blacklist toggle failed: {e}"
                                    );
                                    // Rollback the optimistic flip.
                                    st.set_is_blacklisted(was_blacklisted);
                                    crate::toast::error_weak(
                                        &weak,
                                        "Failed to update artist visibility",
                                    );
                                }
                            }
                        }
                    }
                }
                ("album", "block") | ("album", "unblock") => {
                    if let Some(w) = weak.upgrade() {
                        let st = w.global::<AlbumState>();
                        // Header menu: the open album is AlbumState, so resolve
                        // the display fields (title/artist/cover) from it.
                        let album_id = if id.is_empty() {
                            st.get_id().to_string()
                        } else {
                            id.clone()
                        };
                        if !album_id.is_empty() {
                            let was_blocked =
                                crate::artist_blacklist::is_album_blacklisted(&album_id);
                            // Optimistic flip on the header toggle.
                            st.set_is_album_blocked(!was_blocked);
                            let title = st.get_title().to_string();
                            let artist = st.get_artist().to_string();
                            let cover = st.get_artwork_url().to_string();
                            let res = if was_blocked {
                                crate::artist_blacklist::remove_album(&album_id)
                            } else {
                                crate::artist_blacklist::add_album(
                                    &album_id, &title, &artist, &cover, None,
                                )
                            };
                            match res {
                                Ok(()) => {
                                    seed_blacklist_status(&w);
                                    let msg = if was_blocked {
                                        qbz_i18n::t_args("Album \"{}\" unblocked", &[&title])
                                    } else {
                                        qbz_i18n::t_args("Album \"{}\" blocked", &[&title])
                                    };
                                    crate::toast::success_weak(&weak, msg);
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] album block toggle failed: {e}"
                                    );
                                    st.set_is_album_blocked(was_blocked);
                                    let emsg = if was_blocked {
                                        qbz_i18n::t("Failed to unblock album")
                                    } else {
                                        qbz_i18n::t("Failed to block album")
                                    };
                                    crate::toast::error_weak(&weak, emsg);
                                }
                            }
                        }
                    }
                }
                // Artist card / grid overlay play button: Popular tracks, with
                // a studio-discography fallback when the artist has none (see
                // playback::play_artist).
                ("artist", "play") => playback::play_artist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id.clone(),
                ),
                ("artist", "play-top") => playback::play_artist_top_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id.clone(),
                ),
                ("artist", "follow") => {
                    // Toggle the artist follow (= Qobuz artist favorite). State
                    // source = the in-memory artist fav cache (seeded by search +
                    // the artist page). Optimistic flip on the cache + every
                    // visible surface (search cards + the ArtistView heart),
                    // revert on network failure.
                    if let (Some(w), Ok(aid)) = (weak.upgrade(), id.parse::<u64>()) {
                        let following = crate::fav_cache::is_artist_favorite(aid);
                        let make = !following;
                        crate::fav_cache::set_artist(aid, make);
                        search::mark_artist_followed(&w, &id, make);
                        let ast = w.global::<ArtistState>();
                        if ast.get_id().as_str() == id.as_str() {
                            ast.set_is_following(make);
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let artist_id = id.clone();
                        handle.spawn(async move {
                            let res = if make {
                                runtime.core().add_favorite("artist", &artist_id).await
                            } else {
                                runtime.core().remove_favorite("artist", &artist_id).await
                            };
                            match res {
                                Ok(()) => {
                                    // reco: log the favorite only on ADD.
                                    if make {
                                        tokio::task::spawn_blocking(move || {
                                            crate::reco::log_favorite_artist(aid)
                                        });
                                    }
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] toggle follow artist failed: {e}"
                                    );
                                    crate::fav_cache::set_artist(aid, following);
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        search::mark_artist_followed(&w, &artist_id, following);
                                        let ast = w.global::<ArtistState>();
                                        if ast.get_id().as_str() == artist_id.as_str() {
                                            ast.set_is_following(following);
                                        }
                                    });
                                }
                            }
                        });
                    }
                }
                // "Not interested" (reco-scoped dismissal — NOT the app-wide
                // blacklist): persist the dismissal, drop the card from the
                // Recommendations rails live, and backfill the freed slot from
                // the retained overflow. The artist stays visible everywhere
                // else (search/home/label pages); future paints exclude it via
                // the §B filter.
                ("artist", "not-interested") => {
                    if let Some(w) = weak.upgrade() {
                        let snapshot =
                            crate::external_reco::apply_artist_dismissal(&w, &image_cache, &id);
                        match snapshot {
                            Some((name, image)) => {
                                if let Ok(aid) = id.parse::<u64>() {
                                    crate::reco_dismiss::dismiss(aid, &name, &image);
                                }
                                crate::toast::info_weak(
                                    &weak,
                                    qbz_i18n::t_args(
                                        "{} won't appear in Recommendations anymore",
                                        &[&name],
                                    ),
                                );
                            }
                            None => {
                                // Dismissed from a non-reco surface (search /
                                // home / pinned card): nothing to remove live
                                // — resolve the display name, then persist.
                                let runtime = runtime.clone();
                                let weak = weak.clone();
                                let artist_id = id.clone();
                                handle.spawn(async move {
                                    let Ok(aid) = artist_id.parse::<u64>() else {
                                        return;
                                    };
                                    let (name, image) = runtime
                                        .core()
                                        .get_artist(aid)
                                        .await
                                        .map(|a| {
                                            (
                                                a.name,
                                                a.image
                                                    .and_then(|i| i.best().cloned())
                                                    .unwrap_or_default(),
                                            )
                                        })
                                        .unwrap_or_default();
                                    crate::reco_dismiss::dismiss(aid, &name, &image);
                                    let msg = if name.is_empty() {
                                        qbz_i18n::t("Artist dismissed from Recommendations")
                                    } else {
                                        qbz_i18n::t_args(
                                            "{} won't appear in Recommendations anymore",
                                            &[&name],
                                        )
                                    };
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        crate::toast::info(&w, msg);
                                    });
                                });
                            }
                        }
                    }
                }
                // === Label landing actions ===============================
                ("label", "follow") => {
                    // Toggle the label favorite, optimistically flipping the
                    // header + any matching More-Labels card.
                    if let Some(w) = weak.upgrade() {
                        let make = !label::label_following_state(&w, &id);
                        label::mark_label_followed(&w, &id, make);
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let label_id = id.clone();
                        handle.spawn(async move {
                            let res = if make {
                                runtime.core().add_favorite("label", &label_id).await
                            } else {
                                runtime.core().remove_favorite("label", &label_id).await
                            };
                            if let Err(e) = res {
                                log::error!("[qbz-slint] toggle label favorite failed: {e}");
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    label::mark_label_followed(&w, &label_id, !make);
                                });
                            }
                        });
                    }
                }
                ("label", "play-top") => {
                    // Popular tracks are cached on the UI thread by
                    // apply_label_page; read them here (UI thread) + queue.
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                }
                // Label Popular Tracks multi-select: mode toggle + bulk bar.
                ("label", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<LabelState>().get_multi_select();
                        label::set_multi_select(&w, !on);
                    }
                }
                ("label", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        label::select_all(&w);
                    }
                }
                ("label", "clear") => {
                    if let Some(w) = weak.upgrade() {
                        label::clear_selection(&w);
                    }
                }
                ("label", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = label::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                ("label", "play-next") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = label::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                // Popular Tracks section menu + header overflow: ALL of the
                // label's popular tracks play-next / add-to-queue (the cached
                // list — same source as "play-top").
                ("label", "top-play-next") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                ("label", "top-queue") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                // Header shuffle: all popular tracks, xorshift-shuffled.
                ("label", "shuffle") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::play_label_top_shuffled(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            id.clone(),
                        );
                    }
                }
                // Header overflow Share — Qobuz web-player label link (no
                // Song.link/Album.link equivalent exists for labels).
                ("label", "share") => {
                    let label_id = if id.is_empty() {
                        weak.upgrade()
                            .map(|w| w.global::<LabelState>().get_id().to_string())
                            .unwrap_or_default()
                    } else {
                        id.clone()
                    };
                    if !label_id.is_empty() {
                        share::copy_to_clipboard(share::qobuz_label_url(&label_id));
                        if let Some(w) = weak.upgrade() {
                            crate::toast::success(&w, qbz_i18n::t("Link copied"));
                        }
                    }
                }
                ("label", "add-to-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = label::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                }
                ("label", "add-to-mixtape") => {
                    if let Some(w) = weak.upgrade() {
                        let items =
                            mixtape_items_from_qobuz_tracks(&label::selected_play_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            label::clear_selection(&w);
                        }
                    }
                }
                // More-Labels card click -> open that label's landing.
                ("label", "open") => {
                    if let Ok(label_id) = id.parse::<u64>() {
                        let name = weak
                            .upgrade()
                            .map(|w| label::more_label_name(&w, &id))
                            .unwrap_or_default();
                        nav::record(nav::NavEntry::Label {
                            id: label_id,
                            name: name.clone(),
                        });
                        navigate_label(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            label_id,
                            name,
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    }
                }
                // "See all" -> the full releases sub-view for the open label.
                ("label", "see-all-releases") => {
                    if let (Some(w), Ok(label_id)) = (weak.upgrade(), id.parse::<u64>()) {
                        let name = w.global::<LabelState>().get_name().to_string();
                        nav::record(nav::NavEntry::LabelReleases {
                            id: label_id,
                            name: name.clone(),
                        });
                        navigate_label_releases(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            label_id,
                            name,
                        );
                        update_nav_flags(&w);
                    }
                }
                ("track", "toggle-select") => {
                    // Plain / Ctrl+Click = single per-row toggle; Shift+Click =
                    // additive range from the per-surface anchor to the clicked
                    // row (1:1 with Tauri applyShiftRange — only ever adds). The
                    // anchor moves to the clicked row after either gesture. The
                    // surface id keys the anchor so a range never leaks across
                    // views; the model `match` mirrors the surface `match`.
                    if let Some(w) = weak.upgrade() {
                        let view = w.global::<NavState>().get_view();
                        let (model, surface) = match view {
                            ContentView::Album => {
                                (w.global::<AlbumState>().get_tracks(), selection::SURFACE_ALBUM)
                            }
                            ContentView::Playlist => (
                                w.global::<PlaylistState>().get_tracks(),
                                selection::SURFACE_PLAYLIST,
                            ),
                            ContentView::Label => (
                                w.global::<LabelState>().get_top_tracks(),
                                selection::SURFACE_LABEL,
                            ),
                            ContentView::Favorites => (
                                w.global::<FavoritesState>().get_tracks_visible(),
                                selection::SURFACE_FAVORITES,
                            ),
                            ContentView::Mix => (
                                w.global::<MixState>().get_tracks(),
                                selection::SURFACE_MIX,
                            ),
                            _ => (
                                w.global::<ArtistState>().get_top_tracks(),
                                selection::SURFACE_ARTIST,
                            ),
                        };
                        if let Some(vm) = model
                            .as_any()
                            .downcast_ref::<slint::VecModel<TrackItem>>()
                        {
                            let clicked = (0..vm.row_count()).find(|&i| {
                                vm.row_data(i)
                                    .map(|t| t.id.as_str() == id.as_str())
                                    .unwrap_or(false)
                            });
                            if let Some(clicked) = clicked {
                                let shift = keybindings::mods().2;
                                let anchor = if shift {
                                    selection::resolve_anchor(surface, vm, |t| t.id.to_string())
                                } else {
                                    None
                                };
                                match anchor {
                                    Some(anchor) => selection::apply_shift_range(
                                        vm,
                                        anchor,
                                        clicked,
                                        |t, v| t.selected = v,
                                    ),
                                    None => {
                                        if let Some(mut item) = vm.row_data(clicked) {
                                            item.selected = !item.selected;
                                            vm.set_row_data(clicked, item);
                                        }
                                    }
                                }
                                selection::set_anchor(surface, clicked, id.as_str());
                            }
                        }
                        match view {
                            ContentView::Album => album::recount_selected(&w),
                            ContentView::Artist => artist::recount_selected(&w),
                            ContentView::Playlist => playlist::recount_selected(&w),
                            ContentView::Favorites => favorites::recount_selected(&w),
                            ContentView::Mix => mix::recount_selected(&w),
                            ContentView::Label => label::recount_selected(&w),
                            _ => {}
                        }
                    }
                }
                // The mix tile sends id = mix kind, action = "open".
                ("mix", "open") => {
                    nav::record(nav::NavEntry::Mix { kind: id.clone() });
                    navigate_mix(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        id.clone(),
                    );
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                }
                ("mix", "play-all") => {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = mix::current_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("mix", "shuffle") => {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = mix::shuffled_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("mix", "refresh") => {
                    // Re-load the current mix (re-fetch its tracks).
                    if let Some(w) = weak.upgrade() {
                        let kind = w.global::<MixState>().get_kind().to_string();
                        if !kind.is_empty() {
                            navigate_mix(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                kind,
                            );
                        }
                    }
                }
                // Mix multi-select: mode toggle + bulk bar (select-all toggles
                // all/none; Ctrl+A select-all-only goes through the key handler).
                ("mix", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<MixState>().get_multi_select();
                        mix::set_multi_select(&w, !on);
                    }
                }
                ("mix", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        mix::select_all(&w);
                    }
                }
                ("mix", "clear") => {
                    if let Some(w) = weak.upgrade() {
                        mix::clear_selection(&w);
                    }
                }
                ("mix", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = mix::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                ("mix", "play-next") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = mix::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                ("mix", "add-to-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = mix::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                }
                ("mix", "add-to-mixtape") => {
                    if let Some(w) = weak.upgrade() {
                        let items =
                            mixtape_items_from_qobuz_tracks(&mix::selected_play_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            mix::clear_selection(&w);
                        }
                    }
                }
                ("playlist", "cache") => {
                    if let Ok(pid) = id.parse::<u64>() {
                        offline_cache::cache_playlist(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                        );
                    }
                }
                ("playlist", "play") => {
                    // Play a playlist by id NOW (replace the queue), from any
                    // playlist CARD overlay / context menu (Discover qobuzPlaylists,
                    // Search, Label) where no PlaylistView is open. The `play-all`
                    // arm below reads the open detail's PlaylistState, so it cannot
                    // serve a cold card play — this fetches the playlist by id.
                    playback::play_playlist(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.clone(),
                    );
                }
                ("playlist", "play-all") => {
                    // LOCAL playlist detail — its own queue snapshot +
                    // offline-only stamp (D8); the offline sidecar view of
                    // a MIXED playlist (D11.a) AND the ONLINE mixed detail
                    // (Seam B: source-aware merged queue) share that
                    // snapshot; the pure-Qobuz path is unchanged below.
                    if let Some(w) = weak.upgrade() {
                        let ps = w.global::<PlaylistState>();
                        if ps.get_is_local()
                            || ps.get_offline_subset()
                            || playlist::is_mixed()
                        {
                            local_playlist::play_all(
                                &w,
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                false,
                            );
                            return;
                        }
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = playlist::current_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("playlist", "shuffle") => {
                    // Mixed pool shuffles as ONE list, local rows as
                    // equals (E9); the context stays the playlist id.
                    if let Some(w) = weak.upgrade() {
                        let ps = w.global::<PlaylistState>();
                        if ps.get_is_local()
                            || ps.get_offline_subset()
                            || playlist::is_mixed()
                        {
                            local_playlist::play_all(
                                &w,
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                true,
                            );
                            return;
                        }
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = playlist::shuffled_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("playlist", "queue") => {
                    if local_playlist::is_local_id(&id) {
                        local_playlist::enqueue_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            false,
                        );
                        return;
                    }
                    playback::enqueue_playlist(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id,
                        false,
                    )
                }
                ("playlist", "play-next") => {
                    if local_playlist::is_local_id(&id) {
                        local_playlist::enqueue_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            true,
                        );
                        return;
                    }
                    playback::enqueue_playlist(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id,
                        true,
                    )
                }
                ("playlist", "upload-to-qobuz") => {
                    // D8: convert a non-offline-only LOCAL playlist into a
                    // real Qobuz playlist (explicit user action, confirmed
                    // in the detail view — nothing ever auto-syncs).
                    if local_playlist::is_local_id(&id) {
                        local_playlist::upload_to_qobuz(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            id,
                        );
                    }
                }
                ("playlist", "favorite") => {
                    // Internal qbz library flag (Qobuz /favorite/create rejects
                    // playlist_ids). id-scoped: a CARD toggles ITS playlist, not
                    // the open one; the DB read picks the direction. `is_open`
                    // keeps the detail's optimistic heart in sync.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_toggle_favorite_by_id(handle.clone(), weak.clone(), pid, is_open);
                    }
                }
                ("playlist", "copy") => {
                    // Copy a Qobuz playlist into the user's own playlists
                    // (create + add every track). id-scoped so a card copies ITS
                    // playlist; the detail passes its own id, so behavior is
                    // unchanged there (is_open flips PlaylistState.is-copied).
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_copy_by_id(runtime.clone(), weak.clone(), handle.clone(), pid, is_open);
                    }
                }
                ("playlist", "follow") => {
                    // Follow on Qobuz (subscribe). The DETAIL button emits
                    // "follow" as a toggle (id == open → flip current state); a
                    // CARD carries its follow-state and emits follow/unfollow
                    // explicitly, so a card "follow" always subscribes.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        let follow = if is_open {
                            !w.global::<PlaylistState>().get_is_following()
                        } else {
                            true
                        };
                        playlist_set_follow_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                            follow,
                            is_open,
                        );
                    }
                }
                ("playlist", "unfollow") => {
                    // Card-only: unfollow (unsubscribe) the given playlist.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_set_follow_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                            false,
                            is_open,
                        );
                    }
                }
                ("playlist", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<PlaylistState>().get_multi_select_mode();
                        playlist::set_multi_select(&w, !on);
                    }
                }
                ("playlist", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        playlist::select_all(&w);
                    }
                }
                ("playlist", "play-next-selected") | ("playlist", "queue-selected") => {
                    // Bulk Play next / Add to queue over the selection
                    // (Tauri's BulkActionBar split-button, spec §1.5) —
                    // source-aware: rows resolve through the merged queue
                    // snapshot (local/cached keep their source — the
                    // T2 fix-forward) or the pure-Qobuz Track cache.
                    if let Some(w) = weak.upgrade() {
                        let next = action == "play-next-selected";
                        let tracks = playlist::selected_queue_tracks(&w);
                        if tracks.is_empty() {
                            toast::error(&w, "Nothing playable in the selection");
                            return;
                        }
                        // Selection clears, mode stays on (LL precedent).
                        playlist::clear_selection(&w);
                        playback::enqueue_queue_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            next,
                        );
                    }
                }
                ("playlist", "add-selected-to-playlist") => {
                    // Bulk Add to playlist (spec §1.5). The picker is
                    // single-mode (catalog ids XOR local-mode refs), so:
                    // Qobuz rows ride the catalog flow; a selection with NO
                    // Qobuz rows rides the local-mode flow (library row ids
                    // — per-row parity for sidecar rows); a MIXED selection
                    // follows Tauri (Qobuz rows only, sidecar rows skipped +
                    // logged).
                    let Some(w) = weak.upgrade() else { return };
                    let rows = playlist::selected_rows(&w);
                    if rows.is_empty() {
                        return;
                    }
                    let mut qobuz_ids: Vec<String> = Vec::new();
                    let mut local_refs: Vec<String> = Vec::new();
                    for row in &rows {
                        match row.source.as_str() {
                            "local" => local_refs.push(row.id.clone()),
                            _ => {
                                if row.id.parse::<u64>().is_ok() {
                                    qobuz_ids.push(row.id.clone());
                                }
                            }
                        }
                    }
                    if !qobuz_ids.is_empty() {
                        if !local_refs.is_empty() {
                            log::info!(
                                "[qbz-slint] bulk add-to-playlist: mixed selection — {} sidecar row(s) skipped (single-mode picker; Tauri §1.5 behavior)",
                                local_refs.len()
                            );
                        }
                        playlist_picker::open_multi(&w, &qobuz_ids, false);
                    } else if !local_refs.is_empty() {
                        playlist_picker::open_multi(&w, &local_refs, true);
                    } else {
                        return;
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                ("playlist", "remove-selected") => {
                    if let Some(w) = weak.upgrade() {
                        // LOCAL playlist — remove the selected rows from the
                        // library.db repo by stored position.
                        if w.global::<PlaylistState>().get_is_local() {
                            local_playlist::remove_selected(
                                &w,
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                image_cache.clone(),
                            );
                            return;
                        }
                        // QOBUZ detail (pure or mixed): split by row
                        // namespace — qobuz rows resolve to ptids, local
                        // rows to the local sidecar delete (Seam D).
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        let rows = playlist::selected_rows(&w);
                        if let (Ok(pid), false) = (pid.parse::<u64>(), rows.is_empty()) {
                            playlist_remove_rows(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                image_cache.clone(),
                                pid,
                                rows,
                            );
                        }
                    }
                }
                ("playlist", "set-artwork") => {
                    // Pick an image, copy it into the artwork cache, store
                    // it as the playlist's custom cover, then reload.
                    if let Some(w) = weak.upgrade() {
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        // LOCAL playlist — same flow, repo-backed.
                        if local_playlist::is_local_id(&pid) {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                let Some(file) = rfd::AsyncFileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                    .await
                                else {
                                    return;
                                };
                                let src = file.path().to_string_lossy().into_owned();
                                let lid = pid.clone();
                                let ok = tokio::task::spawn_blocking(move || {
                                    local_playlist::set_custom_artwork_blocking(&lid, &src)
                                        .is_some()
                                })
                                .await
                                .unwrap_or(false);
                                if ok {
                                    local_playlist::navigate(
                                        runtime, weak, &handle, image_cache, pid,
                                    );
                                }
                            });
                            return;
                        }
                        if let Ok(pid) = pid.parse::<u64>() {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                let Some(file) = rfd::AsyncFileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                    .await
                                else {
                                    return;
                                };
                                let src = file.path().to_string_lossy().into_owned();
                                let ok = tokio::task::spawn_blocking(move || {
                                    playlist::set_custom_artwork(pid, &src).is_some()
                                })
                                .await
                                .unwrap_or(false);
                                if ok {
                                    navigate_playlist(
                                        runtime, weak, &handle, image_cache, pid.to_string(),
                                    );
                                }
                            });
                        }
                    }
                }
                ("playlist", "clear-artwork") => {
                    if let Some(w) = weak.upgrade() {
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        // LOCAL playlist — clear the repo column + reload.
                        if local_playlist::is_local_id(&pid) {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                let lid = pid.clone();
                                tokio::task::spawn_blocking(move || {
                                    local_playlist::clear_custom_artwork_blocking(&lid);
                                })
                                .await
                                .ok();
                                local_playlist::navigate(
                                    runtime, weak, &handle, image_cache, pid,
                                );
                            });
                            return;
                        }
                        if let Ok(pid) = pid.parse::<u64>() {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                tokio::task::spawn_blocking(move || {
                                    playlist::clear_custom_artwork(pid);
                                })
                                .await
                                .ok();
                                navigate_playlist(
                                    runtime, weak, &handle, image_cache, pid.to_string(),
                                );
                            });
                        }
                    }
                }
                ("playlist", "edit") => {
                    // Open the edit modal, prefilled from the open playlist.
                    if let Some(w) = weak.upgrade() {
                        let ps = w.global::<PlaylistState>();
                        let pid = ps.get_id();
                        let name = ps.get_name();
                        let desc = ps.get_description();
                        let is_local = ps.get_is_local();
                        let offline_only = ps.get_offline_only();
                        let es = w.global::<EditPlaylistState>();
                        es.set_id(pid);
                        es.set_name(name);
                        es.set_description(desc);
                        es.set_is_local(is_local);
                        es.set_offline_only(offline_only);
                        es.set_open(true);
                    }
                }
                ("track", "move-up") | ("track", "move-down") => {
                    // Custom-order reorder (playlist view). Optimistic UI
                    // move, then persist the full order off-thread.
                    if let Some(w) = weak.upgrade() {
                        let up = action == "move-up";
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        // LOCAL playlist (B2): the move writes the repo's
                        // position order directly (no custom-order sidecar).
                        if local_playlist::is_local_id(&pid) {
                            local_playlist::move_row(&w, &handle, id.as_str(), up);
                        } else {
                            let orders = playlist::move_track(&w, id.as_str(), up);
                            if !orders.is_empty() {
                                if let Ok(pid) = pid.parse::<u64>() {
                                    handle.spawn(async move {
                                        tokio::task::spawn_blocking(move || {
                                            playlist::persist_custom(pid, orders);
                                        })
                                        .await
                                        .ok();
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }
}
