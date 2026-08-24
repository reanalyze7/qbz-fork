use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Step B: execute the import (re-fetches the URL internally —
        // Tauri behavior, kept) with live sink progress.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistImportActions>()
            .on_execute(move || {
                let Some(w) = weak.upgrade() else { return; };
                let Some(args) = playlist_import::begin_execute(&w) else {
                    return;
                };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                handle.clone().spawn(async move {
                    // Tauri's RequiresUserSession gate: execute needs a
                    // logged-in client (the preview does not).
                    let client = runtime.core().client().read().await.clone();
                    let Some(client) = client else {
                        let g = args.generation;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            if g == playlist_import::current_generation() {
                                playlist_import::apply_execute_err(
                                    &w,
                                    "Not logged in to Qobuz",
                                );
                            }
                            toast::show(&w, "Playlist import failed", ToastKind::Error);
                        });
                        return;
                    };
                    let sink: Arc<dyn qbz_playlist_import::ImportProgressSink> = Arc::new(
                        playlist_import::SlintSink::new(weak.clone(), args.generation),
                    );
                    let res = qbz_playlist_import::import_public_playlist(
                        &args.url,
                        &client,
                        args.name_override.as_deref(),
                        false, // is_public — Tauri hardcodes false, no toggle
                        sink,
                    )
                    .await;
                    match res {
                        Ok(summary) => {
                            // reco: NOT logged. The importer is a bulk external
                            // import (Spotify/Apple/...), not a per-track taste
                            // action, and Tauri never logged it — left unlogged
                            // for parity. (Re-evaluate if the owner wants it.)
                            // Assign every created part to the chosen folder
                            // (local DB) BEFORE the sidebar reload — mirrors
                            // the create-playlist precedent above.
                            if !args.folder_id.is_empty() {
                                for pid in &summary.qobuz_playlist_ids {
                                    let (pid, fid) = (*pid, args.folder_id.clone());
                                    tokio::task::spawn_blocking(move || {
                                        folders::move_playlist(pid, Some(fid.as_str()));
                                    })
                                    .await
                                    .ok();
                                }
                            }
                            let g = args.generation;
                            let weak2 = weak.clone();
                            let r2 = runtime.clone();
                            let h2 = handle.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                // Toast + sidebar refresh fire even after a
                                // close mid-import (§1.8); the generation
                                // guard keeps a stale run's writes off a
                                // reopened modal's fresh state.
                                if g == playlist_import::current_generation() {
                                    playlist_import::apply_execute_ok(&w, &summary);
                                }
                                if summary.matched_tracks > 0 {
                                    toast::show(&w, "Playlist imported", ToastKind::Success);
                                }
                                load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                if let Some(first) = summary.qobuz_playlist_ids.first() {
                                    // Navigate only while the modal is still
                                    // open AND this run is current (§1.8).
                                    if g == playlist_import::current_generation()
                                        && w.global::<PlaylistImportState>().get_open()
                                    {
                                        nav::record(nav::NavEntry::Playlist(first.to_string()));
                                        navigate_playlist(
                                            r2,
                                            weak2,
                                            &h2,
                                            image_cache,
                                            first.to_string(),
                                        );
                                        update_nav_flags(&w);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            let g = args.generation;
                            let msg = e.to_string();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                if g == playlist_import::current_generation() {
                                    playlist_import::apply_execute_err(&w, &msg);
                                }
                                toast::show(&w, "Playlist import failed", ToastKind::Error);
                            });
                        }
                    }
                });
            });
    }
}
