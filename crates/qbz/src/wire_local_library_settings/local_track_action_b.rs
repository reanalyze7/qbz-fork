use crate::*;

// `on_track_action` arms: add-to-mixtape, favorite. Called unconditionally
// alongside `local_track_action_a`/`_c` from the single `on_track_action`
// registration (part8.rs).
pub(crate) fn local_track_action_b(
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let id = id.to_string();
    match action {
                    "add-to-mixtape" => {
                        // Single-row Add to Mixtape/Collection (Tracks tab +
                        // folder-detail rows; spec §3.1). Same resolution as
                        // play-next: loaded cache first, DB fallback
                        // off-thread for folder rows.
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            let items = myqbz_add::track_items_from_local(&[row]);
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                if let Some(row) = row {
                                    let items = myqbz_add::track_items_from_local(&[row]);
                                    open_add_to_mixtape(weak2, handle2, items);
                                }
                            });
                        }
                    }
                    "favorite" => {
                        // Library-surface favorite: the menu only shows the
                        // entry on qobuz_download rows (TrackRow gates on
                        // source == "qobuz"), and the toggle uses the row's
                        // REAL qobuz_track_id — never the local row id, which
                        // is what Tauri sends (spec §3.2 latent bug; we port
                        // the intent, not the bug).
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            match row.qobuz_track_id {
                                Some(qid) => toggle_track_favorite(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    qid.to_string(),
                                ),
                                None => log::debug!(
                                    "[qbz-slint] favorite: local row {id} has no qobuz_track_id"
                                ),
                            }
                        } else if let Ok(rid) = id.parse::<i64>() {
                            // Folder rows aren't in the Tracks cache: resolve
                            // off-thread, then hop back to the UI thread (the
                            // toggle reads/writes UI models).
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                let Some(qid) = row.and_then(|r| r.qobuz_track_id) else {
                                    log::debug!(
                                        "[qbz-slint] favorite: row {rid} has no qobuz_track_id"
                                    );
                                    return;
                                };
                                let weak3 = weak2.clone();
                                let _ = weak2.upgrade_in_event_loop(move |_w| {
                                    toggle_track_favorite(
                                        runtime,
                                        weak3,
                                        handle2,
                                        qid.to_string(),
                                    );
                                });
                            });
                        }
                    }
        _ => {}
    }
}
