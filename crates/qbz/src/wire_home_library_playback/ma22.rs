use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch22(
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
                ("mix", "open") => {
                    nav::record(nav::NavEntry::Mix { kind: id.clone() });
                    navigate_mix(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        id.clone(),
                    );
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                }
                ("mix", "play-all") => {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = mix::current_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("mix", "shuffle") => {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = mix::shuffled_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("mix", "refresh") => {
                    // Re-load the current mix (re-fetch its tracks).
                    if let Some(w) = weak.upgrade() {
                        let kind = w.global::<MixState>().get_kind().to_string();
                        if !kind.is_empty() {
                            navigate_mix(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                kind,
                            );
                        }
                    }
                }
                // Mix multi-select: mode toggle + bulk bar (select-all toggles
                // all/none; Ctrl+A select-all-only goes through the key handler).
                ("mix", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<MixState>().get_multi_select();
                        mix::set_multi_select(&w, !on);
                    }
                }
                ("mix", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        mix::select_all(&w);
                    }
                }
                ("mix", "clear") => {
                    if let Some(w) = weak.upgrade() {
                        mix::clear_selection(&w);
                    }
                }
                ("mix", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = mix::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                ("mix", "play-next") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = mix::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                ("mix", "add-to-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = mix::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                }
                ("mix", "add-to-mixtape") => {
                    if let Some(w) = weak.upgrade() {
                        let items =
                            mixtape_items_from_qobuz_tracks(&mix::selected_play_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            mix::clear_selection(&w);
                        }
                    }
                }
        _ => {}
    }
}
