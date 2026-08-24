use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch23(
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
        _ => {}
    }
}
