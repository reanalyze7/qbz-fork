use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Rename the playlist, then refresh the detail header + sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EditPlaylistActions>()
            .on_save(move || {
                let Some(w) = weak.upgrade() else { return; };
                let es = w.global::<EditPlaylistState>();
                let name = es.get_name().to_string();
                let description = es.get_description().to_string();
                let id = es.get_id().to_string();
                if name.trim().is_empty() || es.get_busy() {
                    return;
                }
                // LOCAL playlist (id "local:<uuid>") — rename/description/
                // offline-only write the library.db repo; no Qobuz call.
                if local_playlist::is_local_id(&id) {
                    let offline_only = es.get_offline_only();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let nm = name.trim().to_string();
                        let ds = description.trim().to_string();
                        let lid = id.clone();
                        let (nm2, ds2) = (nm.clone(), ds.clone());
                        let ok = tokio::task::spawn_blocking(move || {
                            let desc = if ds2.is_empty() { None } else { Some(ds2.as_str()) };
                            local_playlist::update_blocking(&lid, &nm2, desc, offline_only)
                        })
                        .await
                        .unwrap_or(false);
                        if !ok {
                            log::error!("[qbz-slint] update local playlist failed");
                            return;
                        }
                        let r2 = runtime.clone();
                        let w2 = weak.clone();
                        let h2 = handle.clone();
                        let rid = id.clone();
                        let rnm = nm.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            // Optimistic sidebar patch FIRST (the reload
                            // alone can show the pre-rename name — see
                            // sidebar::rename_entry), then reconcile.
                            sidebar::rename_entry(&w, &id, &nm);
                            let ps = w.global::<PlaylistState>();
                            // Only refresh the open detail header if this IS
                            // the open playlist.
                            if ps.get_id().as_str() == id {
                                ps.set_name(nm.into());
                                ps.set_description(ds.into());
                                ps.set_offline_only(offline_only);
                            }
                            w.global::<EditPlaylistState>().set_open(false);
                        });
                        // Hold the new name until the data source agrees
                        // (first pass for local: the DB read is already fresh).
                        reconcile_sidebar_after_rename(r2, w2, &h2, rid, rnm);
                    });
                    return;
                }
                let (Ok(pid),) = (id.parse::<u64>(),) else { return; };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let desc_opt = Some(description.trim());
                    match runtime
                        .core()
                        .update_playlist(pid, Some(name.trim()), desc_opt, None)
                        .await
                    {
                        Ok(_) => {
                            let nm = name.trim().to_string();
                            let ds = description.trim().to_string();
                            let r2 = runtime.clone();
                            let w2 = weak.clone();
                            let h2 = handle.clone();
                            let rid = id.clone();
                            let rnm = nm.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                // Optimistic sidebar patch FIRST — Qobuz's
                                // playlist/list can lag read-after-write, so
                                // the reload alone may show the old name (see
                                // sidebar::rename_entry).
                                sidebar::rename_entry(&w, &id, &nm);
                                w.global::<PlaylistState>().set_name(nm.into());
                                w.global::<PlaylistState>().set_description(ds.into());
                                w.global::<EditPlaylistState>().set_open(false);
                            });
                            // Hold the optimistic name until Qobuz's list
                            // agrees (bounded retries); replaces the plain
                            // reload that overwrote it with the stale name.
                            reconcile_sidebar_after_rename(r2, w2, &h2, rid, rnm);
                        }
                        Err(e) => log::error!("[qbz-slint] update playlist failed: {e}"),
                    }
                });
            });
    }
}
