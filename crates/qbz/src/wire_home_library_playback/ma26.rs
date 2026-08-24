use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch26(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let _id = id.to_string();
    match (kind, action) {
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
        _ => {}
    }
}
