use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch11(
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
    let _image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
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
        _ => {}
    }
}
