use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch27(
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
    let id = id.to_string();
    match (kind, action) {
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
        _ => {}
    }
}
