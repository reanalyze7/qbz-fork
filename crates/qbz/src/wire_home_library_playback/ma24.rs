use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch24(
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
        _ => {}
    }
}
