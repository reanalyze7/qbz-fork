use crate::*;

// `on_track_action` arms: go-to-album/go-to-artist, and the
// unhandled-action log fallback. Called unconditionally alongside
// `local_track_action_a`/`_b` from the single `on_track_action`
// registration (part8.rs).
pub(crate) fn local_track_action_c(
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
                    "go-to-album" | "go-to-artist" => {
                        // Owner improvement over Tauri (which omits both on
                        // local rows): resolve the row (Tracks cache first,
                        // DB fallback for folder-detail rows — same seam as
                        // favorite) and source-route in local_row_goto
                        // (local -> local album view / LocalLibrary
                        // artist by name; qobuz_download -> the REAL Qobuz
                        // pages via its qobuz_track_id).
                        let to_artist = action == "go-to-artist";
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            local_row_goto(runtime.clone(), weak.clone(), &handle, row, to_artist);
                        } else if let Ok(rid) = id.parse::<i64>() {
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
                                match row {
                                    Some(row) => local_row_goto(
                                        runtime, weak2, &handle2, row, to_artist,
                                    ),
                                    None => log::debug!(
                                        "[qbz-slint] go-to: local row {rid} not found"
                                    ),
                                }
                            });
                        }
                    }
                    _ => {
                        log::debug!("[qbz-slint] unhandled local track action: {id} {action}");
                    }
    }
}
