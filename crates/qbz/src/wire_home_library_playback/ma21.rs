use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch21(
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
                ("label", "see-all-releases") => {
                    if let (Some(w), Ok(label_id)) = (weak.upgrade(), id.parse::<u64>()) {
                        let name = w.global::<LabelState>().get_name().to_string();
                        nav::record(nav::NavEntry::LabelReleases {
                            id: label_id,
                            name: name.clone(),
                        });
                        navigate_label_releases(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            label_id,
                            name,
                        );
                        update_nav_flags(&w);
                    }
                }
                ("track", "toggle-select") => {
                    // Plain / Ctrl+Click = single per-row toggle; Shift+Click =
                    // additive range from the per-surface anchor to the clicked
                    // row (1:1 with Tauri applyShiftRange — only ever adds). The
                    // anchor moves to the clicked row after either gesture. The
                    // surface id keys the anchor so a range never leaks across
                    // views; the model `match` mirrors the surface `match`.
                    if let Some(w) = weak.upgrade() {
                        let view = w.global::<NavState>().get_view();
                        let (model, surface) = match view {
                            ContentView::Album => {
                                (w.global::<AlbumState>().get_tracks(), selection::SURFACE_ALBUM)
                            }
                            ContentView::Playlist => (
                                w.global::<PlaylistState>().get_tracks(),
                                selection::SURFACE_PLAYLIST,
                            ),
                            ContentView::Label => (
                                w.global::<LabelState>().get_top_tracks(),
                                selection::SURFACE_LABEL,
                            ),
                            ContentView::Favorites => (
                                w.global::<FavoritesState>().get_tracks_visible(),
                                selection::SURFACE_FAVORITES,
                            ),
                            ContentView::Mix => (
                                w.global::<MixState>().get_tracks(),
                                selection::SURFACE_MIX,
                            ),
                            _ => (
                                w.global::<ArtistState>().get_top_tracks(),
                                selection::SURFACE_ARTIST,
                            ),
                        };
                        if let Some(vm) = model
                            .as_any()
                            .downcast_ref::<slint::VecModel<TrackItem>>()
                        {
                            let clicked = (0..vm.row_count()).find(|&i| {
                                vm.row_data(i)
                                    .map(|t| t.id.as_str() == id.as_str())
                                    .unwrap_or(false)
                            });
                            if let Some(clicked) = clicked {
                                let shift = keybindings::mods().2;
                                let anchor = if shift {
                                    selection::resolve_anchor(surface, vm, |t| t.id.to_string())
                                } else {
                                    None
                                };
                                match anchor {
                                    Some(anchor) => selection::apply_shift_range(
                                        vm,
                                        anchor,
                                        clicked,
                                        |t, v| t.selected = v,
                                    ),
                                    None => {
                                        if let Some(mut item) = vm.row_data(clicked) {
                                            item.selected = !item.selected;
                                            vm.set_row_data(clicked, item);
                                        }
                                    }
                                }
                                selection::set_anchor(surface, clicked, id.as_str());
                            }
                        }
                        match view {
                            ContentView::Album => album::recount_selected(&w),
                            ContentView::Artist => artist::recount_selected(&w),
                            ContentView::Playlist => playlist::recount_selected(&w),
                            ContentView::Favorites => favorites::recount_selected(&w),
                            ContentView::Mix => mix::recount_selected(&w),
                            ContentView::Label => label::recount_selected(&w),
                            _ => {}
                        }
                    }
                }
                // The mix tile sends id = mix kind, action = "open".
        _ => {}
    }
}
