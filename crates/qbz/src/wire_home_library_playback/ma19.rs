use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch19(
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
                ("label", "follow") => {
                    // Toggle the label favorite, optimistically flipping the
                    // header + any matching More-Labels card.
                    if let Some(w) = weak.upgrade() {
                        let make = !label::label_following_state(&w, &id);
                        label::mark_label_followed(&w, &id, make);
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let label_id = id.clone();
                        handle.spawn(async move {
                            let res = if make {
                                runtime.core().add_favorite("label", &label_id).await
                            } else {
                                runtime.core().remove_favorite("label", &label_id).await
                            };
                            if let Err(e) = res {
                                log::error!("[qbz-slint] toggle label favorite failed: {e}");
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    label::mark_label_followed(&w, &label_id, !make);
                                });
                            }
                        });
                    }
                }
                ("label", "play-top") => {
                    // Popular tracks are cached on the UI thread by
                    // apply_label_page; read them here (UI thread) + queue.
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                }
                // Label Popular Tracks multi-select: mode toggle + bulk bar.
                ("label", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<LabelState>().get_multi_select();
                        label::set_multi_select(&w, !on);
                    }
                }
                ("label", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        label::select_all(&w);
                    }
                }
                ("label", "clear") => {
                    if let Some(w) = weak.upgrade() {
                        label::clear_selection(&w);
                    }
                }
                ("label", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = label::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                ("label", "play-next") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = label::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                // Popular Tracks section menu + header overflow: ALL of the
                // label's popular tracks play-next / add-to-queue (the cached
                // list — same source as "play-top").
                ("label", "top-play-next") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                ("label", "top-queue") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                // Header shuffle: all popular tracks, xorshift-shuffled.
                ("label", "shuffle") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::play_label_top_shuffled(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            id.clone(),
                        );
                    }
                }
                // Header overflow Share — Qobuz web-player label link (no
                // Song.link/Album.link equivalent exists for labels).
        _ => {}
    }
}
