use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch20(
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
                ("label", "share") => {
                    let label_id = if id.is_empty() {
                        weak.upgrade()
                            .map(|w| w.global::<LabelState>().get_id().to_string())
                            .unwrap_or_default()
                    } else {
                        id.clone()
                    };
                    if !label_id.is_empty() {
                        share::copy_to_clipboard(share::qobuz_label_url(&label_id));
                        if let Some(w) = weak.upgrade() {
                            crate::toast::success(&w, qbz_i18n::t("Link copied"));
                        }
                    }
                }
                ("label", "add-to-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = label::selected_ids(&w);
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
                ("label", "add-to-mixtape") => {
                    if let Some(w) = weak.upgrade() {
                        let items =
                            mixtape_items_from_qobuz_tracks(&label::selected_play_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            label::clear_selection(&w);
                        }
                    }
                }
                // More-Labels card click -> open that label's landing.
                ("label", "open") => {
                    if let Ok(label_id) = id.parse::<u64>() {
                        let name = weak
                            .upgrade()
                            .map(|w| label::more_label_name(&w, &id))
                            .unwrap_or_default();
                        nav::record(nav::NavEntry::Label {
                            id: label_id,
                            name: name.clone(),
                        });
                        navigate_label(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            label_id,
                            name,
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    }
                }
                // "See all" -> the full releases sub-view for the open label.
        _ => {}
    }
}
