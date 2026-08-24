use crate::*;

/// Open the full "See all releases" sub-view for `label_id`. Fetches the
/// label header + first album page, then the header image.
pub(crate) fn navigate_label_releases(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    label_id: u64,
    name: String,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            label::reset_label(&w);
            w.global::<NavState>().set_view(ContentView::LabelReleases);
        });
        match label::load_label(&runtime, label_id, &name).await {
            Ok(data) => {
                let jobs = label::artwork_jobs(&data);
                let image_url = data.image_url.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    label::apply_label(&w, data);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                if !image_url.is_empty() {
                    if let Some((pixels, width, height)) =
                        artwork::fetch_and_decode(&image_url, &image_cache, 240).await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            label::apply_image(&w, &pixels, width, height);
                        });
                    }
                }
            }
            Err(e) => {
                log::error!("[qbz-slint] label releases load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<LabelState>().set_loading(false);
                });
            }
        }
    });
}

/// Open the dedicated discography page for one release bucket of an artist.
/// Reuses `artist::load_release_page` (get_releases_grid) for the first page.
pub(crate) fn navigate_artist_releases(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    artist_id: String,
    name: String,
    release_type: String,
) {
    handle.spawn(async move {
        let title = artist::release_type_title(&release_type);
        let aid = artist_id.clone();
        let nm = name.clone();
        let rt = release_type.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            artist_releases::reset(&w, &aid, &nm, &rt, &title);
            w.global::<NavState>().set_view(ContentView::ArtistReleases);
        });
        match artist::load_release_page(&runtime, &artist_id, &release_type, 0).await {
            Ok((cards, has_more)) => {
                let image_cache = image_cache.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let jobs = artist_releases::apply_page(&w, cards, has_more, true);
                    artwork::spawn_loads(jobs, w.as_weak(), image_cache);
                });
            }
            Err(e) => {
                log::error!("[qbz-slint] artist releases load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    let st = w.global::<ArtistReleasesState>();
                    st.set_loading(false);
                    st.set_load_error(true);
                });
            }
        }
    });
}

/// Load the immersive Suggestions split panel for the current track. Reads the
/// artist-id + track-id + title off NowPlayingState, resets the panel, and
/// spawns the live artist load (mirror of navigate_award). An empty artist-id
/// or track-id resets to the no-track empty state. Refreshed on track change
/// while the panel is open via the mount's `changed live-track-id` -> the
/// SuggestionsActions.load callback (which calls this).
pub(crate) fn navigate_suggestions(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    artist_id: String,
    track_id: String,
    track_name: String,
) {
    // No track / no artist -> reset to the empty state and stop. apply with an
    // empty payload clears cards/tracks and leaves artist-id "" (the empty
    // branch in the panel).
    let (Ok(aid), Ok(tid)) = (artist_id.parse::<u64>(), track_id.parse::<u64>()) else {
        let _ = weak.upgrade_in_event_loop(|w| {
            suggestions::apply_suggestions(&w, suggestions::empty_payload());
        });
        return;
    };
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            suggestions::reset_suggestions(&w);
        });
        let payload = suggestions::load_suggestions(&runtime, aid, tid, track_name).await;
        let jobs = suggestions::suggestions_artwork_jobs(&payload);
        let _ = weak.upgrade_in_event_loop(move |w| {
            suggestions::apply_suggestions(&w, payload);
        });
        artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
    });
}

