use crate::*;

/// Load an artist page and show the artist view, then fetch the portrait.
/// Shared by the `open-artist` callback and by history back/forward. Split
/// into `artist_library_jobs`, `spawn_artist_mb_enrichment`, and
/// `apply_artist_portrait` (this dir's `artist_*.rs`) to stay under the
/// 130-line file cap.
pub(crate) fn navigate_artist(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    artist_id: String,
) {
    let artist_id_for_state = artist_id.clone();
    handle.spawn(async move {
        let id_for_apply = artist_id_for_state.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            artist::reset_artist(&w);
            artist::reset_network_sidebar(&w);
            // Reflect whether THIS artist is blacklisted so the overflow
            // menu shows Show/Blacklist correctly and the hidden banner
            // appears. is_blacklisted auto-gates on the enabled flag (reads
            // false when the feature is disabled, which is acceptable here).
            let is_bl = crate::artist_blacklist::is_blacklisted_id_str(&id_for_apply);
            let st = w.global::<ArtistState>();
            // Seed the pin state from the pinned store (Home "Pinned"
            // section) — before set_id, which moves id_for_apply.
            st.set_pinned(crate::pinned::is_pinned("artist", &id_for_apply));
            st.set_id(id_for_apply.into());
            st.set_is_blacklisted(is_bl);
            w.global::<NavState>().set_view(ContentView::Artist);
        });
        match artist::load_artist(&runtime, &artist_id).await {
            Ok(data) => {
                let artwork_url = data.artwork_url.clone();
                let jobs = artist::artwork_jobs(&data);
                let artist_name = data.name.clone();
                // Resolve a user-set custom portrait (keyed by artist name)
                // up front — it persists across navigation and is the ONLY
                // image source for artists with no Qobuz portrait (Vicky).
                let custom_image_path = crate::custom_artwork::artist_image(&artist_name);
                let similar_names_for_discovery: Vec<String> = data
                    .similar_artists
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();
                let (lib, lib_jobs) = artist_library_jobs(&artist_id);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    artist::apply_artist(&w, data);
                    if let Some(lib) = lib.as_ref() {
                        let ast = w.global::<ArtistState>();
                        ast.set_library_count(lib.count() as i32);
                        ast.set_library_tracks(crate::library_by_artist::track_items(&lib.tracks));
                        ast.set_library_albums(crate::library_by_artist::album_items(&lib.albums));
                    }
                    w.global::<ArtistState>().set_loading(false);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                artwork::spawn_loads(lib_jobs, weak.clone(), image_cache.clone());

                // Seed the follow heart: pull the user's followed artists (also
                // refreshes the in-memory cache the toggle reads), then reflect
                // whether THIS artist is followed.
                {
                    let runtime_fav = runtime.clone();
                    let weak_fav = weak.clone();
                    let aid_fav = artist_id.clone();
                    tokio::spawn(async move {
                        if let Ok(ids) = runtime_fav.core().favorite_artist_ids().await {
                            let is_fav = aid_fav
                                .parse::<u64>()
                                .map(|a| ids.contains(&a))
                                .unwrap_or(false);
                            crate::fav_cache::set_all_artists(ids);
                            let _ = weak_fav.upgrade_in_event_loop(move |w| {
                                let ast = w.global::<ArtistState>();
                                if ast.get_id().as_str() == aid_fav.as_str() {
                                    ast.set_is_following(is_fav);
                                }
                            });
                        }
                    });
                }

                // Magazine / Stories — fetch the editorial teasers in
                // parallel; the sidebar section stays hidden if there are none.
                {
                    let runtime_story = runtime.clone();
                    let weak_story = weak.clone();
                    let artist_id_story = artist_id.clone();
                    let image_cache_story = image_cache.clone();
                    tokio::spawn(async move {
                        let stories = artist::load_stories(&runtime_story, &artist_id_story).await;
                        let _ = weak_story.upgrade_in_event_loop(move |w| {
                            let jobs = artist::apply_stories(&w, stories);
                            artwork::spawn_loads(jobs, w.as_weak(), image_cache_story);
                        });
                    });
                }

                spawn_artist_mb_enrichment(
                    runtime.clone(),
                    weak.clone(),
                    artist_name,
                    similar_names_for_discovery,
                );

                apply_artist_portrait(weak.clone(), image_cache.clone(), custom_image_path, artwork_url)
                    .await;
            }
            Err(e) => {
                log::error!("[qbz-slint] artist load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<ArtistState>().set_loading(false);
                });
            }
        }
    });
}
