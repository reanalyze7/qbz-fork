use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Create the playlist, then refresh the sidebar + open it.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<CreatePlaylistActions>()
            .on_submit(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                use slint::Model;
                let state = w.global::<CreatePlaylistState>();
                let name = state.get_name().to_string();
                let description = state.get_description().to_string();
                let is_public = state.get_is_public();
                // Resolve the selected folder id ("" = No folder).
                let folder_idx = state.get_folder_index();
                let folder_id = state
                    .get_folder_ids()
                    .row_data(folder_idx.max(0) as usize)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                if name.trim().is_empty() || state.get_creating() {
                    return;
                }
                // D8: offline-only toggle ON — or the app is offline (always
                // local then) — creates a LOCAL playlist in library.db. The
                // online + toggle OFF path below stays byte-unchanged.
                let offline_now = offline_mode::engine().is_offline();
                if state.get_offline_only() || offline_now {
                    // Offline-only when the user opted in; a playlist forced
                    // local by being offline keeps the flag too (it can be
                    // unmarked later in Edit to enable "Upload to Qobuz").
                    state.set_creating(true);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    let image_cache = image_cache.clone();
                    handle.clone().spawn(async move {
                        let nm = name.trim().to_string();
                        let ds = description.trim().to_string();
                        let created = tokio::task::spawn_blocking(move || {
                            let desc = if ds.is_empty() { None } else { Some(ds.as_str()) };
                            local_playlist::create_blocking(&nm, desc, true)
                        })
                        .await
                        .ok()
                        .flatten();
                        let weak2 = weak.clone();
                        let r2 = runtime.clone();
                        let h2 = handle.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            w.global::<CreatePlaylistState>().set_creating(false);
                            match created {
                                Some(new_id) => {
                                    w.global::<CreatePlaylistState>().set_open(false);
                                    load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                    nav::record(nav::NavEntry::Playlist(new_id.clone()));
                                    navigate_playlist(r2, weak2.clone(), &h2, image_cache, new_id);
                                    update_nav_flags(&w);
                                }
                                None => {
                                    log::error!("[qbz-slint] create local playlist failed");
                                }
                            }
                        });
                    });
                    return;
                }
                state.set_creating(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                handle.clone().spawn(async move {
                    let desc = description.trim();
                    let desc_opt = if desc.is_empty() { None } else { Some(desc) };
                    match runtime.core().create_playlist(name.trim(), desc_opt, is_public).await {
                        Ok(playlist) => {
                            let new_id = playlist.id.to_string();
                            // Assign to the chosen folder (local DB) before
                            // the sidebar reloads, mirroring Tauri.
                            if !folder_id.is_empty() {
                                let pid = playlist.id;
                                let fid = folder_id.clone();
                                tokio::task::spawn_blocking(move || {
                                    folders::move_playlist(pid, Some(fid.as_str()));
                                })
                                .await
                                .ok();
                            }
                            let weak2 = weak.clone();
                            let r2 = runtime.clone();
                            let h2 = handle.clone();
                            let ic2 = image_cache.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<CreatePlaylistState>().set_creating(false);
                                w.global::<CreatePlaylistState>().set_open(false);
                                load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                nav::record(nav::NavEntry::Playlist(new_id.clone()));
                                navigate_playlist(r2, weak2.clone(), &h2, ic2, new_id);
                                update_nav_flags(&w);
                            });
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] create playlist failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<CreatePlaylistState>().set_creating(false);
                            });
                        }
                    }
                });
            });
    }
}
