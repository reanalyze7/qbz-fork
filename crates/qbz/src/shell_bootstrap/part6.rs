use crate::*;

/// Fetch the discover index (honoring the shared genre selection),
/// apply all three tab section sets, show the requested tab, and fan
/// out artwork. Shared by the initial shell load and genre-filter /
/// tab re-fetches.
pub(crate) async fn reload_home(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    image_cache: &artwork::ImageCache,
    active_tab: String,
) {
    // The raw genre selection (parent or sub-genre ids) goes straight to
    // /discover/index — Qobuz facets sub-genre ids server-side (Tauri parity).
    let genre_ids = current_genre_filter();

    match home::load_home(runtime, genre_ids).await {
        Ok(data) => {
            // Album-carousel covers are now fired by select_tab below: the
            // prefs-driven render loop draws Home/Editor album sections from the
            // DiscoverState descriptor lists, so their artwork is descriptor-
            // targeted (DiscoverSectionAlbum) and returned by select_tab once the
            // lists are built. Here we only prebuild the artwork for the models
            // that still bind HomeState fields (slim grids, recent albums,
            // playlists), which select_tab does not rebuild.
            let mut jobs: Vec<artwork::ArtworkJob> = Vec::new();
            // Home-only slim grids (their models are populated regardless
            // of the visible tab; harmless to prefetch).
            jobs.extend(data.popular.iter().enumerate().filter_map(|(idx, slim)| {
                (!slim.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::Popular { idx },
                    url: slim.artwork_url.clone(),
                })
            }));
            jobs.extend(data.recent.iter().enumerate().filter_map(|(idx, slim)| {
                (!slim.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::Recent { idx },
                    url: slim.artwork_url.clone(),
                })
            }));
            // Recently-played album covers: Qobuz covers use the plain loader;
            // local covers need the source-aware funnel (local file read),
            // else they never resolve.
            let mut local_album_jobs: Vec<artwork::ArtworkJob> = Vec::new();
            for (idx, card) in data.recent_albums.iter().enumerate() {
                if card.artwork_url.is_empty() {
                    continue;
                }
                let job = artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::RecentAlbum { idx },
                    url: card.artwork_url.clone(),
                };
                if card.source == "local" {
                    local_album_jobs.push(job);
                } else {
                    jobs.push(job);
                }
            }

            // #566 ported-rail covers — Library Albums / Release Watch are
            // Qobuz catalog albums and Top Artists are Qobuz artist images,
            // so the plain loader applies (same as their For You twins).
            // Prefetched regardless of the pref state, like the slim grids
            // above: the models are populated either way and the configurator
            // re-render is cache-only, so enabling a section must find its
            // covers ready. (qobuzMixes is static tiles — no artwork.)
            jobs.extend(data.favorite_albums.iter().enumerate().filter_map(|(idx, card)| {
                (!card.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::HomeFavoriteAlbum { idx },
                    url: card.artwork_url.clone(),
                })
            }));
            jobs.extend(data.most_played_albums.iter().enumerate().filter_map(|(idx, card)| {
                (!card.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::HomeMostPlayedAlbum { idx },
                    url: card.artwork_url.clone(),
                })
            }));
            jobs.extend(data.release_watch.iter().enumerate().filter_map(|(idx, card)| {
                (!card.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::HomeReleaseWatchAlbum { idx },
                    url: card.artwork_url.clone(),
                })
            }));
            jobs.extend(data.top_artists.iter().enumerate().filter_map(|(idx, artist)| {
                (!artist.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::HomeTopArtist { idx },
                    url: artist.artwork_url.clone(),
                })
            }));

            // Qobuz Playlists row covers for the active tab (single-cover,
            // Qobuz CDN URLs → the plain loader, never the local funnel).
            let empty_playlists: Vec<home::PlaylistCardData> = Vec::new();
            let active_playlists = match active_tab.as_str() {
                "editorPicks" => &data.editor_playlists,
                "forYou" => &empty_playlists,
                _ => &data.playlists,
            };
            jobs.extend(home::playlist_artwork_jobs(active_playlists));

            let weak_for_artwork = weak.clone();
            let weak_for_local = weak.clone();
            let image_cache_local = image_cache.clone();
            let image_cache_sections = image_cache.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                home::apply_home(&w, data);
                // apply_home caches the section sets + pushes the descriptor
                // lists; select_tab renders the requested tab from them and
                // returns the descriptor-targeted album-section artwork jobs
                // (DiscoverSectionAlbum) — spawn them here, on the UI thread.
                let section_jobs = home::select_tab(&w, &active_tab);
                artwork::spawn_loads(section_jobs, w.as_weak(), image_cache_sections.clone());
                w.global::<HomeState>().set_loading(false);
            });
            artwork::spawn_loads(jobs, weak_for_artwork, image_cache.clone());
            if !local_album_jobs.is_empty() {
                artwork::spawn_local_loads(local_album_jobs, weak_for_local, image_cache_local);
            }
        }
        Err(e) => {
            log::error!("[qbz-slint] discover load failed: {e}");
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<HomeState>().set_loading(false);
            });
        }
    }
}

